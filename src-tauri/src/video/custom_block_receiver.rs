use crate::video::custom_block_reassembler::{CustomBlockOutput, H264Reassembler};
use crate::video::source::{CustomBlockParserMode, VideoPipelineConfig, VideoSource};
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::Emitter;
use tokio::sync::{mpsc, Mutex};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomBlockStatsPayload {
  pub custom_block_packets_received: u64,
  pub custom_block_bytes_received: u64,
  pub custom_block_ready_frames: u64,
  pub custom_block_invalid_packets: u64,
  pub custom_block_parser_mode: CustomBlockParserMode,
  pub custom_block_mock_active: bool,
}

#[derive(Debug, Clone)]
pub struct CustomBlockStats {
  pub packets_received: u64,
  pub bytes_received: u64,
  pub ready_frames: u64,
  pub invalid_packets: u64,
  pub last_packet_at: Option<Instant>,
  pub last_frame_at: Option<String>,
  pub mock_active: bool,
}

impl Default for CustomBlockStats {
  fn default() -> Self {
    Self {
      packets_received: 0,
      bytes_received: 0,
      ready_frames: 0,
      invalid_packets: 0,
      last_packet_at: None,
      last_frame_at: None,
      mock_active: false,
    }
  }
}

impl CustomBlockStats {
  pub fn payload(&self, parser_mode: CustomBlockParserMode) -> CustomBlockStatsPayload {
    CustomBlockStatsPayload {
      custom_block_packets_received: self.packets_received,
      custom_block_bytes_received: self.bytes_received,
      custom_block_ready_frames: self.ready_frames,
      custom_block_invalid_packets: self.invalid_packets,
      custom_block_parser_mode: parser_mode,
      custom_block_mock_active: self.mock_active,
    }
  }

  pub fn stream_alive(&self) -> bool {
    self
      .last_packet_at
      .map(|ts| ts.elapsed() <= Duration::from_secs(2))
      .unwrap_or(false)
  }
}

pub async fn handle_custom_block_data(
  data: &[u8],
  decoder_input_tx: Arc<Mutex<Option<mpsc::Sender<Vec<u8>>>>>,
  config: Arc<Mutex<VideoPipelineConfig>>,
  stats: Arc<Mutex<CustomBlockStats>>,
  reassembler: Arc<Mutex<H264Reassembler>>,
) {
  let pipeline_config = config.lock().await.clone();

  {
    let mut stats_guard = stats.lock().await;
    stats_guard.packets_received += 1;
    stats_guard.bytes_received += data.len() as u64;
    stats_guard.last_packet_at = Some(Instant::now());
  }

  if pipeline_config.current_video_source != VideoSource::CustomByteBlockH264 {
    return;
  }

  let output = {
    let mut reassembler_guard = reassembler.lock().await;
    match pipeline_config.custom_block_parser_mode {
      CustomBlockParserMode::RawAnnexBStream => reassembler_guard.push_raw_annexb_stream(data),
      CustomBlockParserMode::PacketizedFrame => reassembler_guard.push_packetized_frame(data),
    }
  };

  match output {
    CustomBlockOutput::Bytes(bytes) => {
      {
        let mut stats_guard = stats.lock().await;
        stats_guard.ready_frames += 1;
        stats_guard.last_frame_at = Some(now_iso_like_string());
      }

      if let Some(tx) = decoder_input_tx.lock().await.as_ref().cloned() {
        let _ = tx.try_send(bytes);
      }
    }
    CustomBlockOutput::Waiting => {}
    CustomBlockOutput::InvalidPacket => {
      stats.lock().await.invalid_packets += 1;
    }
  }
}

pub fn decode_custom_byte_block_payload(payload: &[u8]) -> Option<Vec<u8>> {
  let mut index = 0usize;
  while index < payload.len() {
    let key = read_varint(payload, &mut index)?;
    let field_number = key >> 3;
    let wire_type = key & 0x07;

    match (field_number, wire_type) {
      (1, 2) => {
        let len = read_varint(payload, &mut index)? as usize;
        let end = index.checked_add(len)?;
        if end > payload.len() {
          return None;
        }
        return Some(payload[index..end].to_vec());
      }
      (_, 0) => {
        let _ = read_varint(payload, &mut index)?;
      }
      (_, 2) => {
        let len = read_varint(payload, &mut index)? as usize;
        index = index.checked_add(len)?;
        if index > payload.len() {
          return None;
        }
      }
      _ => return None,
    }
  }

  None
}

pub fn emit_custom_block_error(app: &tauri::AppHandle, message: String) {
  if let Err(error) = app.emit("custom_block_error", message) {
    log::error!("emit custom_block_error failed: {error:?}");
  }
}

fn read_varint(bytes: &[u8], index: &mut usize) -> Option<u64> {
  let mut result = 0u64;
  for shift in (0..64).step_by(7) {
    let byte = *bytes.get(*index)?;
    *index += 1;
    result |= ((byte & 0x7f) as u64) << shift;
    if byte & 0x80 == 0 {
      return Some(result);
    }
  }
  None
}

fn now_iso_like_string() -> String {
  let duration = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or(Duration::from_secs(0));
  format!("{}.{:03}Z", duration.as_secs(), duration.subsec_millis())
}
