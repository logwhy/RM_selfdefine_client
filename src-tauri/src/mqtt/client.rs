use crate::video::custom_block_reassembler::H264Reassembler;
use crate::video::custom_block_receiver::{
  decode_custom_byte_block_payload, emit_custom_block_error, handle_custom_block_data, CustomBlockStats,
};
use crate::video::decoder::{MOCK_DECODER_ENABLED, REAL_DECODER_ENABLED};
use crate::video::frame_hub::LatestFrameHub;
use crate::video::source::VideoPipelineConfig;
use crate::video::udp_receiver::VideoStatsPayload;
use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS};
use serde::Serialize;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::{sleep, Duration};

const DEPLOY_MODE_TOPIC: &str = "DeployModeStatusSync";
const CUSTOM_BYTE_BLOCK_TOPIC: &str = "CustomByteBlock";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeSyncEventPayload {
  pub mqtt_connected: bool,
  pub deploy_mode_active: Option<bool>,
  pub last_mode_sync_at: Option<String>,
}

pub struct MqttRuntime {
  stop_tx: Option<oneshot::Sender<()>>,
  join_handle: tokio::task::JoinHandle<()>,
}

impl MqttRuntime {
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

pub fn spawn_mqtt_loop(
  app: tauri::AppHandle,
  host: String,
  port: u16,
  decoder_input_tx: Arc<Mutex<Option<mpsc::Sender<Vec<u8>>>>>,
  video_config: Arc<Mutex<VideoPipelineConfig>>,
  custom_block_stats: Arc<Mutex<CustomBlockStats>>,
  custom_block_reassembler: Arc<Mutex<H264Reassembler>>,
  frame_hub: Arc<LatestFrameHub>,
) -> MqttRuntime {
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();

  let join_handle = tokio::spawn(async move {
    let mut mqtt_options = MqttOptions::new("hero-deploy-tauri-client", host, port);
    mqtt_options.set_keep_alive(Duration::from_secs(10));

    let (client, mut eventloop): (AsyncClient, EventLoop) = AsyncClient::new(mqtt_options, 16);

    emit_mode_sync(
      &app,
      ModeSyncEventPayload {
        mqtt_connected: false,
        deploy_mode_active: None,
        last_mode_sync_at: None,
      },
    );

    let mut subscribed = false;
    let mut connected = false;

    loop {
      tokio::select! {
        _ = &mut stop_rx => {
          break;
        }
        event = eventloop.poll() => {
          match event {
            Ok(Event::Incoming(Incoming::ConnAck(_))) => {
              connected = true;
              emit_mode_sync(
                &app,
                ModeSyncEventPayload {
                  mqtt_connected: true,
                  deploy_mode_active: None,
                  last_mode_sync_at: None,
                },
              );

              if !subscribed {
                if let Err(error) = client.subscribe(DEPLOY_MODE_TOPIC, QoS::AtLeastOnce).await {
                  log::error!("subscribe DeployModeStatusSync failed: {error:?}");
                } else if let Err(error) = client.subscribe(CUSTOM_BYTE_BLOCK_TOPIC, QoS::AtLeastOnce).await {
                  log::error!("subscribe CustomByteBlock failed: {error:?}");
                } else {
                  subscribed = true;
                }
              }
            }
            Ok(Event::Incoming(Incoming::Publish(message))) => {
              if message.topic == DEPLOY_MODE_TOPIC {
                let parsed = parse_deploy_mode_payload(&message.payload);
                emit_mode_sync(
                  &app,
                  ModeSyncEventPayload {
                    mqtt_connected: true,
                    deploy_mode_active: parsed,
                    last_mode_sync_at: Some(chrono_like_now_iso8601()),
                  },
                );
              } else if message.topic == CUSTOM_BYTE_BLOCK_TOPIC {
                if let Some(data) = decode_custom_byte_block_payload(&message.payload) {
                  handle_custom_block_data(
                    &data,
                    decoder_input_tx.clone(),
                    video_config.clone(),
                    custom_block_stats.clone(),
                    custom_block_reassembler.clone(),
                  )
                  .await;
                  emit_video_stats(
                    &app,
                    &frame_hub,
                    video_config.clone(),
                    custom_block_stats.clone(),
                  )
                  .await;
                } else {
                  custom_block_stats.lock().await.invalid_packets += 1;
                  emit_custom_block_error(
                    &app,
                    "CustomByteBlock protobuf parse failed: missing bytes field 1".into(),
                  );
                }
              }
            }
            Ok(_) => {}
            Err(error) => {
              log::warn!("mqtt poll error: {error:?}");
              if connected {
                connected = false;
                emit_mode_sync(
                  &app,
                  ModeSyncEventPayload {
                    mqtt_connected: false,
                    deploy_mode_active: None,
                    last_mode_sync_at: None,
                  },
                );
              }
              sleep(Duration::from_millis(800)).await;
            }
          }
        }
      }
    }

    emit_mode_sync(
      &app,
      ModeSyncEventPayload {
        mqtt_connected: false,
        deploy_mode_active: None,
        last_mode_sync_at: None,
      },
    );
  });

  MqttRuntime::new(stop_tx, join_handle)
}

fn emit_mode_sync(app: &tauri::AppHandle, payload: ModeSyncEventPayload) {
  if let Err(error) = app.emit("mode_sync", payload) {
    log::error!("emit mode_sync event failed: {error:?}");
  }
}

async fn emit_video_stats(
  app: &tauri::AppHandle,
  frame_hub: &Arc<LatestFrameHub>,
  video_config: Arc<Mutex<VideoPipelineConfig>>,
  custom_block_stats: Arc<Mutex<CustomBlockStats>>,
) {
  let decoder_stats = frame_hub.decoder_stats().await;
  let frame_age_ms = frame_hub.latest_frame_age_ms().await;
  let config = video_config.lock().await.clone();
  let stats_guard = custom_block_stats.lock().await;
  let custom_stats = stats_guard.payload(config.custom_block_parser_mode);
  let stream_alive = stats_guard.stream_alive();
  drop(stats_guard);

  if let Err(error) = app.emit(
    "video_stats",
    VideoStatsPayload {
      stream_alive,
      packet_loss_count: 0,
      last_frame_at: None,
      fps: 0.0,
      packets_received: 0,
      active_frames: 0,
      timeout_dropped_frames: 0,
      incomplete_frames: 0,
      ready_frames: 0,
      decoder_reset_count: decoder_stats.decoder_reset_count,
      last_decode_cost_ms: decoder_stats.last_decode_cost_ms,
      latest_frame_age_ms: frame_age_ms,
      is_rendering_real_frame: frame_age_ms.is_some(),
      real_decoder_enabled: REAL_DECODER_ENABLED,
      stub_decoder_enabled: MOCK_DECODER_ENABLED,
      current_mode: config.current_mode,
      current_video_source: config.current_video_source,
      current_codec_mode: config.current_codec_mode,
      current_decoder_name: decoder_name_or_default(
        decoder_stats.current_decoder_name,
        config.current_codec_mode,
      ),
      decoder_init_success: decoder_stats.decoder_init_success,
      custom_block_packets_received: custom_stats.custom_block_packets_received,
      custom_block_bytes_received: custom_stats.custom_block_bytes_received,
      custom_block_ready_frames: custom_stats.custom_block_ready_frames,
      custom_block_invalid_packets: custom_stats.custom_block_invalid_packets,
      custom_block_parser_mode: custom_stats.custom_block_parser_mode,
      custom_block_mock_active: custom_stats.custom_block_mock_active,
    },
  ) {
    log::error!("emit video_stats for CustomByteBlock failed: {error:?}");
  }
}

fn decoder_name_or_default(
  current_decoder_name: String,
  codec_mode: crate::video::source::CodecMode,
) -> String {
  if current_decoder_name.trim().is_empty() {
    return codec_mode.decoder_name(REAL_DECODER_ENABLED);
  }
  current_decoder_name
}

fn parse_deploy_mode_payload(payload: &[u8]) -> Option<bool> {
  if let Ok(text) = std::str::from_utf8(payload) {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
      if let Some(flag) = json.get("deployModeActive").and_then(|v| v.as_bool()) {
        return Some(flag);
      }
    }
    match text.trim() {
      "1" | "true" | "on" => return Some(true),
      "0" | "false" | "off" => return Some(false),
      _ => {}
    }
  }

  // TODO: replace this with official protobuf parser:
  // parse_deploy_mode_status_sync_proto(payload)
  None
}

fn chrono_like_now_iso8601() -> String {
  let now = std::time::SystemTime::now();
  let duration = now
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or(Duration::from_secs(0));
  format!("{}.{:03}Z", duration.as_secs(), duration.subsec_millis())
}
