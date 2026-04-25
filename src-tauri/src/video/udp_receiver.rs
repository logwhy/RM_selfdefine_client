use crate::video::custom_block_receiver::CustomBlockStats;
use crate::video::decoder::{MOCK_DECODER_ENABLED, REAL_DECODER_ENABLED};
use crate::video::frame_hub::LatestFrameHub;
use crate::video::reassembler::FrameReassembler;
use crate::video::source::{ClientMode, CodecMode, CustomBlockParserMode, VideoPipelineConfig, VideoSource};
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::Emitter;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tokio::time::MissedTickBehavior;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoStatsPayload {
  pub stream_alive: bool,
  pub packet_loss_count: u64,
  pub last_frame_at: Option<String>,
  pub fps: f64,
  pub packets_received: u64,
  pub active_frames: usize,
  pub timeout_dropped_frames: u64,
  pub incomplete_frames: u64,
  pub ready_frames: u64,
  pub decoder_reset_count: u64,
  pub last_decode_cost_ms: f64,
  pub latest_frame_age_ms: Option<u128>,
  pub is_rendering_real_frame: bool,
  pub real_decoder_enabled: bool,
  pub stub_decoder_enabled: bool,
  pub current_mode: ClientMode,
  pub current_video_source: VideoSource,
  pub current_codec_mode: CodecMode,
  pub current_decoder_name: String,
  pub decoder_init_success: bool,
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
  pub h264_seen_sps: bool,
  pub h264_seen_pps: bool,
  pub h264_seen_idr: bool,
  pub h264_last_nal_type: Option<u8>,
  pub h264_buffered_bytes: usize,
  pub h264_nal_units_parsed: u64,
  pub h264_frames_submitted_to_decoder: u64,
  pub h264_frames_decoded: u64,
  pub h264_frames_dropped: u64,
  pub h264_decoder_errors: u64,
  pub h264_consecutive_decode_errors: u64,
  pub dropped_old_frames: u64,
  pub dropped_by_backpressure: u64,
  pub decode_input_queue_len: usize,
  pub frame_render_queue_len: usize,
  pub avg_decode_cost_ms: f64,
  pub max_decode_cost_ms: f64,
  pub last_render_cost_ms: f64,
  pub avg_end_to_end_latency_ms: f64,
}

pub struct VideoRuntime {
  stop_tx: Option<oneshot::Sender<()>>,
  join_handle: tokio::task::JoinHandle<()>,
}

impl VideoRuntime {
  pub fn new(stop_tx: oneshot::Sender<()>, join_handle: tokio::task::JoinHandle<()>) -> Self {
    Self {
      stop_tx: Some(stop_tx),
      join_handle,
    }
  }

  pub async fn stop(mut self) {
    if let Some(stop_tx) = self.stop_tx.take() {
      let _ = stop_tx.send(());
    }
    let _ = self.join_handle.await;
  }
}

pub fn spawn_video_receiver_with_decoder(
  app: tauri::AppHandle,
  port: u16,
  decoder_input_tx: mpsc::Sender<Vec<u8>>,
  frame_hub: Arc<LatestFrameHub>,
  video_config: Arc<Mutex<VideoPipelineConfig>>,
  custom_block_stats: Arc<Mutex<CustomBlockStats>>,
) -> VideoRuntime {
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();

  let join_handle = tokio::spawn(async move {
    let bind_addr = format!("0.0.0.0:{port}");
    let socket = match UdpSocket::bind(&bind_addr).await {
      Ok(s) => s,
      Err(error) => {
        log::error!("bind udp receiver failed on {bind_addr}: {error:?}");
        emit_video_stats(
          &app,
          VideoStatsPayload {
            stream_alive: false,
            packet_loss_count: 0,
            last_frame_at: None,
            fps: 0.0,
            packets_received: 0,
            active_frames: 0,
            timeout_dropped_frames: 0,
            incomplete_frames: 0,
            ready_frames: 0,
            decoder_reset_count: 0,
            last_decode_cost_ms: 0.0,
            latest_frame_age_ms: None,
            is_rendering_real_frame: false,
            real_decoder_enabled: REAL_DECODER_ENABLED,
            stub_decoder_enabled: MOCK_DECODER_ENABLED,
            current_mode: ClientMode::Normal,
            current_video_source: VideoSource::UdpHevc,
            current_codec_mode: CodecMode::Hevc,
            current_decoder_name: CodecMode::Hevc.decoder_name(REAL_DECODER_ENABLED),
            decoder_init_success: false,
            custom_block_packets_received: 0,
            custom_block_bytes_received: 0,
            custom_block_ready_frames: 0,
            custom_block_invalid_packets: 0,
            custom_block_packets_per_second: 0.0,
            custom_block_bytes_per_second: 0.0,
            custom_block_bitrate_kbps: 0.0,
            custom_block_dropped_blocks: 0,
            custom_block_buffered_bytes: 0,
            custom_block_last_receive_at: None,
            custom_block_no_data_duration_ms: None,
            custom_block_parser_mode: CustomBlockParserMode::RawAnnexBStream,
            custom_block_mock_active: false,
            h264_seen_sps: false,
            h264_seen_pps: false,
            h264_seen_idr: false,
            h264_last_nal_type: None,
            h264_buffered_bytes: 0,
            h264_nal_units_parsed: 0,
            h264_frames_submitted_to_decoder: 0,
            h264_frames_decoded: 0,
            h264_frames_dropped: 0,
            h264_decoder_errors: 0,
            h264_consecutive_decode_errors: 0,
            dropped_old_frames: 0,
            dropped_by_backpressure: 0,
            decode_input_queue_len: 0,
            frame_render_queue_len: 0,
            avg_decode_cost_ms: 0.0,
            max_decode_cost_ms: 0.0,
            last_render_cost_ms: 0.0,
            avg_end_to_end_latency_ms: 0.0,
          },
        );
        return;
      }
    };

    let mut buf = vec![0u8; 65_535];
    let mut ticker = tokio::time::interval(Duration::from_millis(250));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let mut reassembler = FrameReassembler::new(Duration::from_millis(1200));
    let mut packets_received: u64 = 0;
    let mut packet_loss_count: u64 = 0;
    let mut last_frame_at: Option<String> = None;
    let mut last_packet_instant: Option<Instant> = None;
    let mut last_frame_id: Option<u16> = None;

    loop {
      tokio::select! {
        _ = &mut stop_rx => {
          break;
        }
        _ = ticker.tick() => {
          reassembler.sweep_timeouts();
          let payload = build_stats_payload(
            &frame_hub,
            &reassembler,
            packets_received,
            packet_loss_count,
            last_packet_instant,
            last_frame_at.clone(),
            &video_config,
            &custom_block_stats,
          ).await;
          emit_video_stats(&app, payload);
        }
        recv = socket.recv_from(&mut buf) => {
          match recv {
            Ok((size, _peer)) => {
              if size < 8 {
                continue;
              }
              packets_received += 1;
              last_packet_instant = Some(Instant::now());

              let frame_id = u16::from_be_bytes([buf[0], buf[1]]);
              let fragment_index = u16::from_be_bytes([buf[2], buf[3]]);
              let frame_total_bytes = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
              let payload = &buf[8..size];

              if let Some(prev) = last_frame_id {
                if frame_id > prev + 1 {
                  packet_loss_count += (frame_id - prev - 1) as u64;
                }
              }
              last_frame_id = Some(frame_id);

              if let Some(ready_frame) = reassembler.push_fragment(frame_id, fragment_index, frame_total_bytes, payload) {
                let _ = decoder_input_tx.try_send(ready_frame);
                last_frame_at = Some(now_iso_like_string());
              }
            }
            Err(error) => {
              log::warn!("recv udp packet failed: {error:?}");
            }
          }
        }
      }
    }

    reassembler.sweep_timeouts();
    let payload = build_stats_payload(
      &frame_hub,
      &reassembler,
      packets_received,
      packet_loss_count,
      None,
      last_frame_at,
      &video_config,
      &custom_block_stats,
    ).await;
    emit_video_stats(&app, payload);
  });

  VideoRuntime::new(stop_tx, join_handle)
}

async fn build_stats_payload(
  frame_hub: &Arc<LatestFrameHub>,
  reassembler: &FrameReassembler,
  packets_received: u64,
  packet_loss_count: u64,
  last_packet_instant: Option<Instant>,
  last_frame_at: Option<String>,
  video_config: &Arc<Mutex<VideoPipelineConfig>>,
  custom_block_stats: &Arc<Mutex<CustomBlockStats>>,
) -> VideoStatsPayload {
  let stats = reassembler.stats();
  let decoder_stats = frame_hub.decoder_stats().await;
  let frame_age = frame_hub.latest_frame_age_ms().await;
  let pipeline_config = video_config.lock().await.clone();
  let custom_stats = custom_block_stats
    .lock()
    .await
    .payload(pipeline_config.custom_block_parser_mode);
  let stream_alive = last_packet_instant
    .map(|ts| ts.elapsed() <= Duration::from_secs(2))
    .unwrap_or(false);

  VideoStatsPayload {
    stream_alive,
    packet_loss_count,
    last_frame_at,
    fps: reassembler.fps_estimate(),
    packets_received,
    active_frames: stats.active_frames,
    timeout_dropped_frames: stats.timeout_dropped_frames,
    incomplete_frames: stats.incomplete_frames,
    ready_frames: stats.ready_frames,
    decoder_reset_count: decoder_stats.decoder_reset_count,
    last_decode_cost_ms: decoder_stats.last_decode_cost_ms,
    latest_frame_age_ms: frame_age,
    is_rendering_real_frame: frame_age.is_some(),
    real_decoder_enabled: REAL_DECODER_ENABLED,
    stub_decoder_enabled: MOCK_DECODER_ENABLED,
    current_mode: pipeline_config.current_mode,
    current_video_source: pipeline_config.current_video_source,
    current_codec_mode: pipeline_config.current_codec_mode,
    current_decoder_name: decoder_name_or_default(
      decoder_stats.current_decoder_name,
      pipeline_config.current_codec_mode,
    ),
    decoder_init_success: decoder_stats.decoder_init_success,
    custom_block_packets_received: custom_stats.custom_block_packets_received,
    custom_block_bytes_received: custom_stats.custom_block_bytes_received,
    custom_block_ready_frames: custom_stats.custom_block_ready_frames,
    custom_block_invalid_packets: custom_stats.custom_block_invalid_packets,
    custom_block_packets_per_second: custom_stats.custom_block_packets_per_second,
    custom_block_bytes_per_second: custom_stats.custom_block_bytes_per_second,
    custom_block_bitrate_kbps: custom_stats.custom_block_bitrate_kbps,
    custom_block_dropped_blocks: custom_stats.custom_block_dropped_blocks,
    custom_block_buffered_bytes: custom_stats.custom_block_buffered_bytes,
    custom_block_last_receive_at: custom_stats.custom_block_last_receive_at,
    custom_block_no_data_duration_ms: custom_stats.custom_block_no_data_duration_ms,
    custom_block_parser_mode: custom_stats.custom_block_parser_mode,
    custom_block_mock_active: custom_stats.custom_block_mock_active,
    h264_seen_sps: custom_stats.h264_parser_stats.h264_seen_sps,
    h264_seen_pps: custom_stats.h264_parser_stats.h264_seen_pps,
    h264_seen_idr: custom_stats.h264_parser_stats.h264_seen_idr,
    h264_last_nal_type: custom_stats.h264_parser_stats.h264_last_nal_type,
    h264_buffered_bytes: custom_stats.h264_parser_stats.h264_buffered_bytes,
    h264_nal_units_parsed: custom_stats.h264_parser_stats.h264_nal_units_parsed,
    h264_frames_submitted_to_decoder: custom_stats.h264_parser_stats.h264_frames_submitted_to_decoder,
    h264_frames_decoded: decoder_stats.frames_decoded,
    h264_frames_dropped: custom_stats.h264_parser_stats.h264_frames_dropped,
    h264_decoder_errors: decoder_stats.decoder_errors,
    h264_consecutive_decode_errors: decoder_stats.consecutive_decode_errors,
    dropped_old_frames: custom_stats.h264_parser_stats.h264_frames_dropped,
    dropped_by_backpressure: custom_stats.custom_block_dropped_blocks,
    decode_input_queue_len: 0,
    frame_render_queue_len: 0,
    avg_decode_cost_ms: decoder_stats.avg_decode_cost_ms,
    max_decode_cost_ms: decoder_stats.max_decode_cost_ms,
    last_render_cost_ms: 0.0,
    avg_end_to_end_latency_ms: 0.0,
  }
}

fn decoder_name_or_default(current_decoder_name: String, codec_mode: CodecMode) -> String {
  if current_decoder_name.trim().is_empty() {
    return codec_mode.decoder_name(REAL_DECODER_ENABLED);
  }
  current_decoder_name
}

fn emit_video_stats(app: &tauri::AppHandle, payload: VideoStatsPayload) {
  if let Err(error) = app.emit("video_stats", payload) {
    log::error!("emit video_stats failed: {error:?}");
  }
}

fn now_iso_like_string() -> String {
  let duration = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or(Duration::from_secs(0));
  format!("{}.{:03}Z", duration.as_secs(), duration.subsec_millis())
}
