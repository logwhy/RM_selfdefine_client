use crate::video::custom_block_reassembler::{CustomBlockOutput, H264ParserStats, H264Reassembler};
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
  pub custom_block_packets_per_second: f64,
  pub custom_block_bytes_per_second: f64,
  pub custom_block_bitrate_kbps: f64,
  pub custom_block_dropped_blocks: u64,
  pub custom_block_buffered_bytes: usize,
  pub custom_block_last_receive_at: Option<String>,
  pub custom_block_no_data_duration_ms: Option<u128>,
  pub custom_block_parser_mode: CustomBlockParserMode,
  pub custom_block_mock_active: bool,
  #[serde(flatten)]
  pub h264_parser_stats: H264ParserStats,
}

#[derive(Debug, Clone)]
pub struct CustomBlockStats {
  pub packets_received: u64,
  pub bytes_received: u64,
  pub ready_frames: u64,
  pub invalid_packets: u64,
  pub dropped_blocks: u64,
  pub first_packet_at: Option<Instant>,
  pub last_packet_at: Option<Instant>,
  pub last_frame_at: Option<String>,
  pub last_packet_at_string: Option<String>,
  pub rate_window_start: Option<Instant>,
  pub window_packets: u64,
  pub window_bytes: u64,
  pub packets_per_second: f64,
  pub bytes_per_second: f64,
  pub mock_active: bool,
  pub h264_parser_stats: H264ParserStats,
}

impl Default for CustomBlockStats {
  fn default() -> Self {
    Self {
      packets_received: 0,
      bytes_received: 0,
      ready_frames: 0,
      invalid_packets: 0,
      dropped_blocks: 0,
      first_packet_at: None,
      last_packet_at: None,
      last_frame_at: None,
      last_packet_at_string: None,
      rate_window_start: None,
      window_packets: 0,
      window_bytes: 0,
      packets_per_second: 0.0,
      bytes_per_second: 0.0,
      mock_active: false,
      h264_parser_stats: H264ParserStats::default(),
    }
  }
}

impl CustomBlockStats {
  pub fn payload(&self, parser_mode: CustomBlockParserMode) -> CustomBlockStatsPayload {
    let no_data_duration_ms = self.last_packet_at.map(|ts| ts.elapsed().as_millis());
    let stream_recent = no_data_duration_ms
      .map(|duration| duration <= 2_000)
      .unwrap_or(false);
    let packets_per_second = if stream_recent { self.packets_per_second } else { 0.0 };
    let bytes_per_second = if stream_recent { self.bytes_per_second } else { 0.0 };
    CustomBlockStatsPayload {
      custom_block_packets_received: self.packets_received,
      custom_block_bytes_received: self.bytes_received,
      custom_block_ready_frames: self.ready_frames,
      custom_block_invalid_packets: self.invalid_packets,
      custom_block_packets_per_second: packets_per_second,
      custom_block_bytes_per_second: bytes_per_second,
      custom_block_bitrate_kbps: (bytes_per_second * 8.0) / 1000.0,
      custom_block_dropped_blocks: self.dropped_blocks,
      custom_block_buffered_bytes: self.h264_parser_stats.h264_buffered_bytes,
      custom_block_last_receive_at: self.last_packet_at_string.clone(),
      custom_block_no_data_duration_ms: no_data_duration_ms,
      custom_block_parser_mode: parser_mode,
      custom_block_mock_active: self.mock_active,
      h264_parser_stats: self.h264_parser_stats.clone(),
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
    let now = Instant::now();
    if stats_guard.first_packet_at.is_none() {
      stats_guard.first_packet_at = Some(now);
    }
    if stats_guard.rate_window_start.is_none() {
      stats_guard.rate_window_start = Some(now);
    }
    stats_guard.packets_received += 1;
    stats_guard.bytes_received += data.len() as u64;
    stats_guard.window_packets += 1;
    stats_guard.window_bytes += data.len() as u64;
    if let Some(window_start) = stats_guard.rate_window_start {
      let elapsed = window_start.elapsed();
      if elapsed >= Duration::from_secs(1) {
        let elapsed_secs = elapsed.as_secs_f64().max(0.001);
        stats_guard.packets_per_second = stats_guard.window_packets as f64 / elapsed_secs;
        stats_guard.bytes_per_second = stats_guard.window_bytes as f64 / elapsed_secs;
        stats_guard.window_packets = 0;
        stats_guard.window_bytes = 0;
        stats_guard.rate_window_start = Some(now);
      }
    }
    stats_guard.last_packet_at = Some(now);
    stats_guard.last_packet_at_string = Some(now_iso_like_string());
  }

  if pipeline_config.current_video_source != VideoSource::CustomByteBlockH264 {
    return;
  }

  let output = {
    let mut reassembler_guard = reassembler.lock().await;
    let output = match pipeline_config.custom_block_parser_mode {
      CustomBlockParserMode::RawAnnexBStream => reassembler_guard.push_raw_annexb_stream(data),
      CustomBlockParserMode::PacketizedFrame => reassembler_guard.push_packetized_frame(data),
    };
    stats.lock().await.h264_parser_stats = reassembler_guard.stats();
    output
  };

  match output {
    CustomBlockOutput::Bytes(bytes) => {
      {
        let mut stats_guard = stats.lock().await;
        stats_guard.ready_frames += 1;
        stats_guard.last_frame_at = Some(now_iso_like_string());
      }

      if let Some(tx) = decoder_input_tx.lock().await.as_ref().cloned() {
        match tx.try_send(bytes) {
          Ok(()) => {}
          Err(mpsc::error::TrySendError::Full(_)) => {
            stats.lock().await.dropped_blocks += 1;
          }
          Err(mpsc::error::TrySendError::Closed(_)) => {}
        }
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
