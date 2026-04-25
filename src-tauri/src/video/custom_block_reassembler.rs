use crate::video::reassembler::FrameReassembler;
use serde::Serialize;
use std::collections::VecDeque;
use std::time::Duration;

pub enum CustomBlockOutput {
  Bytes(Vec<u8>),
  Waiting,
  InvalidPacket,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct H264ParserStats {
  pub h264_seen_sps: bool,
  pub h264_seen_pps: bool,
  pub h264_seen_idr: bool,
  pub h264_last_nal_type: Option<u8>,
  pub h264_buffered_bytes: usize,
  pub h264_nal_units_parsed: u64,
  pub h264_frames_submitted_to_decoder: u64,
  pub h264_frames_dropped: u64,
}

pub struct H264Reassembler {
  raw_buffer: Vec<u8>,
  ready_outputs: VecDeque<Vec<u8>>,
  current_access_unit: Vec<Vec<u8>>,
  current_has_vcl: bool,
  sps: Option<Vec<u8>>,
  pps: Option<Vec<u8>>,
  seen_idr: bool,
  last_nal_type: Option<u8>,
  nal_units_parsed: u64,
  frames_submitted_to_decoder: u64,
  frames_dropped: u64,
  frame_reassembler: FrameReassembler,
}

impl H264Reassembler {
  pub fn new() -> Self {
    Self {
      raw_buffer: Vec::with_capacity(4096),
      ready_outputs: VecDeque::with_capacity(2),
      current_access_unit: Vec::new(),
      current_has_vcl: false,
      sps: None,
      pps: None,
      seen_idr: false,
      last_nal_type: None,
      nal_units_parsed: 0,
      frames_submitted_to_decoder: 0,
      frames_dropped: 0,
      frame_reassembler: FrameReassembler::new(Duration::from_millis(1200)),
    }
  }

  pub fn push_raw_annexb_stream(&mut self, data: &[u8]) -> CustomBlockOutput {
    if data.is_empty() {
      return CustomBlockOutput::InvalidPacket;
    }
    self.raw_buffer.extend_from_slice(data);
    if self.raw_buffer.len() > 64 * 1024 {
      self.raw_buffer.clear();
      self.frames_dropped += 1;
      return CustomBlockOutput::InvalidPacket;
    }

    let Some(first_start) = find_start_code(&self.raw_buffer, 0) else {
      if self.raw_buffer.len() > 4 {
        let keep_from = self.raw_buffer.len().saturating_sub(4);
        self.raw_buffer.drain(0..keep_from);
      }
      return CustomBlockOutput::Waiting;
    };

    if first_start > 0 {
      self.raw_buffer.drain(0..first_start);
    }

    while let Some(next_start) = find_start_code(&self.raw_buffer, 4) {
      let nal = self.raw_buffer.drain(0..next_start).collect::<Vec<u8>>();
      match self.push_h264_nal(nal) {
        CustomBlockOutput::Bytes(bytes) => self.push_ready_output(bytes),
        CustomBlockOutput::InvalidPacket => return CustomBlockOutput::InvalidPacket,
        CustomBlockOutput::Waiting => {}
      }
    }

    self.pop_ready_output()
  }

  pub fn push_packetized_frame(&mut self, data: &[u8]) -> CustomBlockOutput {
    if data.len() < 8 {
      return CustomBlockOutput::InvalidPacket;
    }

    let frame_id = u16::from_be_bytes([data[0], data[1]]);
    let fragment_index = u16::from_be_bytes([data[2], data[3]]);
    let frame_total_bytes = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let payload = &data[8..];

    match self
      .frame_reassembler
      .push_fragment(frame_id, fragment_index, frame_total_bytes, payload)
    {
      Some(frame) => {
        self.frames_submitted_to_decoder += 1;
        CustomBlockOutput::Bytes(frame)
      }
      None => CustomBlockOutput::Waiting,
    }
  }

  pub fn stats(&self) -> H264ParserStats {
    H264ParserStats {
      h264_seen_sps: self.sps.is_some(),
      h264_seen_pps: self.pps.is_some(),
      h264_seen_idr: self.seen_idr,
      h264_last_nal_type: self.last_nal_type,
      h264_buffered_bytes: self.raw_buffer.len(),
      h264_nal_units_parsed: self.nal_units_parsed,
      h264_frames_submitted_to_decoder: self.frames_submitted_to_decoder,
      h264_frames_dropped: self.frames_dropped,
    }
  }

  fn push_ready_output(&mut self, bytes: Vec<u8>) {
    if self.ready_outputs.len() >= 1 {
      self.ready_outputs.pop_front();
      self.frames_dropped += 1;
    }
    self.ready_outputs.push_back(bytes);
  }

  fn pop_ready_output(&mut self) -> CustomBlockOutput {
    if let Some(bytes) = self.ready_outputs.pop_back() {
      self.ready_outputs.clear();
      self.frames_submitted_to_decoder += 1;
      CustomBlockOutput::Bytes(bytes)
    } else {
      CustomBlockOutput::Waiting
    }
  }

  fn push_h264_nal(&mut self, nal: Vec<u8>) -> CustomBlockOutput {
    let Some(nal_type) = h264_nal_type(&nal) else {
      return CustomBlockOutput::InvalidPacket;
    };
    self.last_nal_type = Some(nal_type);
    self.nal_units_parsed += 1;

    match nal_type {
      7 => {
        self.sps = Some(nal.clone());
        self.current_access_unit.push(nal);
        CustomBlockOutput::Waiting
      }
      8 => {
        self.pps = Some(nal.clone());
        self.current_access_unit.push(nal);
        CustomBlockOutput::Waiting
      }
      9 => {
        if self.current_has_vcl {
          let output = self.finish_access_unit();
          self.current_access_unit.push(nal);
          output
        } else {
          self.current_access_unit.push(nal);
          CustomBlockOutput::Waiting
        }
      }
      1 | 5 => {
        if nal_type == 5 {
          self.seen_idr = true;
        }
        if self.current_has_vcl && h264_first_mb_in_slice_is_zero(&nal) {
          let output = self.finish_access_unit();
          self.begin_access_unit_with_cached_parameter_sets();
          self.current_access_unit.push(nal);
          self.current_has_vcl = true;
          return output;
        }

        if self.current_access_unit.is_empty() {
          self.begin_access_unit_with_cached_parameter_sets();
        }
        self.current_access_unit.push(nal);
        self.current_has_vcl = true;
        CustomBlockOutput::Waiting
      }
      _ => {
        self.current_access_unit.push(nal);
        CustomBlockOutput::Waiting
      }
    }
  }

  fn begin_access_unit_with_cached_parameter_sets(&mut self) {
    if let Some(sps) = self.sps.clone() {
      self.current_access_unit.push(sps);
    }
    if let Some(pps) = self.pps.clone() {
      self.current_access_unit.push(pps);
    }
  }

  fn finish_access_unit(&mut self) -> CustomBlockOutput {
    if !self.current_has_vcl || self.current_access_unit.is_empty() {
      self.current_access_unit.clear();
      self.current_has_vcl = false;
      return CustomBlockOutput::Waiting;
    }

    let total_len = self.current_access_unit.iter().map(Vec::len).sum();
    let mut bytes = Vec::with_capacity(total_len);
    for nal in self.current_access_unit.drain(..) {
      bytes.extend_from_slice(&nal);
    }
    self.current_has_vcl = false;
    CustomBlockOutput::Bytes(bytes)
  }
}

fn find_start_code(data: &[u8], from: usize) -> Option<usize> {
  let mut index = from;
  while index + 3 <= data.len() {
    if index + 3 <= data.len() && data[index..].starts_with(&[0, 0, 1]) {
      return Some(index);
    }
    if index + 4 <= data.len() && data[index..].starts_with(&[0, 0, 0, 1]) {
      return Some(index);
    }
    index += 1;
  }
  None
}

fn start_code_len(data: &[u8]) -> Option<usize> {
  if data.starts_with(&[0, 0, 0, 1]) {
    Some(4)
  } else if data.starts_with(&[0, 0, 1]) {
    Some(3)
  } else {
    None
  }
}

fn h264_nal_type(nal: &[u8]) -> Option<u8> {
  let offset = start_code_len(nal)?;
  Some(*nal.get(offset)? & 0x1f)
}

fn h264_first_mb_in_slice_is_zero(nal: &[u8]) -> bool {
  let Some(offset) = start_code_len(nal) else {
    return false;
  };
  let rbsp = remove_emulation_prevention_bytes(nal.get(offset + 1..).unwrap_or_default());
  read_unsigned_exp_golomb_is_zero(&rbsp)
}

fn remove_emulation_prevention_bytes(bytes: &[u8]) -> Vec<u8> {
  let mut out = Vec::with_capacity(bytes.len());
  let mut zero_count = 0usize;
  for &byte in bytes {
    if zero_count >= 2 && byte == 0x03 {
      zero_count = 0;
      continue;
    }
    out.push(byte);
    if byte == 0 {
      zero_count += 1;
    } else {
      zero_count = 0;
    }
  }
  out
}

fn read_unsigned_exp_golomb_is_zero(bytes: &[u8]) -> bool {
  for (byte_index, byte) in bytes.iter().enumerate() {
    for bit_index in 0..8 {
      let bit = (byte >> (7 - bit_index)) & 1;
      if bit == 1 {
        return byte_index == 0 && bit_index == 0;
      }
    }
  }
  false
}
