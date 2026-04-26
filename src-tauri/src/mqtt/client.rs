use crate::video::custom_block_reassembler::H264Reassembler;
use crate::video::custom_block_receiver::{
    decode_custom_byte_block_payload, emit_custom_block_error, handle_custom_block_data,
    CustomBlockStats,
};
use crate::video::decoder::{MOCK_DECODER_ENABLED, REAL_DECODER_ENABLED};
use crate::video::frame_hub::LatestFrameHub;
use crate::video::source::VideoPipelineConfig;
use crate::video::udp_receiver::VideoStatsPayload;
use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::{mpsc, oneshot, watch, Mutex};
use tokio::time::{sleep, Duration, MissedTickBehavior};

const DEPLOY_MODE_TOPIC: &str = "DeployModeStatusSync";
const CUSTOM_BYTE_BLOCK_TOPIC: &str = "CustomByteBlock";
const KEYBOARD_MOUSE_CONTROL_TOPIC: &str = "KeyboardMouseControl";
const REFEREE_TOPICS: &[&str] = &[
    "GameStatus",
    "GlobalUnitStatus",
    "GlobalLogisticsStatus",
    "GlobalSpecialMechanism",
    "Event",
    "RobotInjuryStat",
    "RobotRespawnStatus",
    "RobotStaticStatus",
    "RobotDynamicStatus",
    "RobotModuleStatus",
    "RobotPosition",
    "Buff",
    "PenaltyInfo",
    "RobotPathPlanInfo",
    "RadarInfoToClient",
    "AssemblyCommand",
    "TechCoreMotionStateSync",
    "RobotPerformanceSelectionSync",
    "RuneStatusSync",
    "SentryStatusSync",
    "DartSelectTargetStatusSync",
    "SentryCtrlResult",
    "AirSupportStatusSync",
    "DeployModeStatusSync",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeSyncEventPayload {
    pub mqtt_connected: bool,
    pub deploy_mode_active: Option<bool>,
    pub last_mode_sync_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStatusPayload {
    pub stage_countdown_sec: Option<i32>,
    pub current_stage: Option<u32>,
    pub current_round: Option<u32>,
    pub total_rounds: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotDynamicStatusPayload {
    pub current_health: Option<u32>,
    pub current_heat: Option<f32>,
    pub current_chassis_energy: Option<u32>,
    pub current_buffer_energy: Option<u32>,
    pub remaining_ammo: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotStaticStatusPayload {
    pub connection_state: Option<u32>,
    pub field_state: Option<u32>,
    pub alive_state: Option<u32>,
    pub robot_id: Option<u32>,
    pub robot_type: Option<u32>,
    pub level: Option<u32>,
    pub max_health: Option<u32>,
    pub max_heat: Option<u32>,
    pub heat_cooldown_rate: Option<f32>,
    pub max_power: Option<u32>,
    pub max_buffer_energy: Option<u32>,
    pub max_chassis_energy: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefereeEventPayload {
    pub event_id: Option<i32>,
    pub param: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefereeMessagePayload {
    pub topic: String,
    pub bytes: usize,
    pub received_at: String,
    pub game_status: Option<GameStatusPayload>,
    pub robot_dynamic_status: Option<RobotDynamicStatusPayload>,
    pub robot_static_status: Option<RobotStaticStatusPayload>,
    pub event: Option<RefereeEventPayload>,
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
    mut input_rx: watch::Receiver<KeyboardMouseInput>,
    input_diagnostics: Arc<Mutex<InputDiagnostics>>,
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
        let mut stats_ticker = tokio::time::interval(Duration::from_millis(500));
        stats_ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        let mut input_sent_count: u64 = 0;
        let mut last_input_hz_sample = std::time::Instant::now();

        loop {
            tokio::select! {
              _ = &mut stop_rx => {
                break;
              }
              _ = stats_ticker.tick() => {
                emit_video_stats(
                  &app,
                  &frame_hub,
                  video_config.clone(),
                  custom_block_stats.clone(),
                )
                .await;
                let elapsed = last_input_hz_sample.elapsed().as_secs_f64();
                if elapsed >= 0.5 {
                  let mut diagnostics = input_diagnostics.lock().await;
                  diagnostics.input_send_hz = input_sent_count as f64 / elapsed;
                  input_sent_count = 0;
                  last_input_hz_sample = std::time::Instant::now();
                }
              }
              changed = input_rx.changed() => {
                if changed.is_err() {
                  continue;
                }
                let command = input_rx.borrow().clone();
                update_input_diagnostics(&input_diagnostics, &command).await;
                if connected && !command.dry_run {
                  let payload = encode_keyboard_mouse_control(&command);
                  match client.publish(KEYBOARD_MOUSE_CONTROL_TOPIC, QoS::AtMostOnce, false, payload).await {
                    Ok(_) => {
                      input_sent_count += 1;
                    }
                    Err(error) => {
                      log::warn!("publish KeyboardMouseControl failed: {error:?}");
                    }
                  }
                }
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
                      let mut subscribe_failed = false;
                      for topic in REFEREE_TOPICS {
                        if let Err(error) = client.subscribe(*topic, QoS::AtLeastOnce).await {
                          subscribe_failed = true;
                          log::error!("subscribe {topic} failed: {error:?}");
                        }
                      }
                      if let Err(error) = client.subscribe(CUSTOM_BYTE_BLOCK_TOPIC, QoS::AtLeastOnce).await {
                        subscribe_failed = true;
                        log::error!("subscribe CustomByteBlock failed: {error:?}");
                      }
                      subscribed = !subscribe_failed;
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
                    } else if REFEREE_TOPICS.contains(&message.topic.as_str()) {
                      emit_referee_message(&app, &message.topic, &message.payload);
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

fn emit_referee_message(app: &tauri::AppHandle, topic: &str, payload: &[u8]) {
    let event = RefereeMessagePayload {
        topic: topic.to_string(),
        bytes: payload.len(),
        received_at: chrono_like_now_iso8601(),
        game_status: if topic == "GameStatus" {
            Some(parse_game_status_payload(payload))
        } else {
            None
        },
        robot_dynamic_status: if topic == "RobotDynamicStatus" {
            Some(parse_robot_dynamic_status_payload(payload))
        } else {
            None
        },
        robot_static_status: if topic == "RobotStaticStatus" {
            Some(parse_robot_static_status_payload(payload))
        } else {
            None
        },
        event: if topic == "Event" {
            Some(parse_event_payload(payload))
        } else {
            None
        },
    };

    if let Err(error) = app.emit("referee_message", event) {
        log::error!("emit referee_message event failed: {error:?}");
    }
}

fn parse_game_status_payload(payload: &[u8]) -> GameStatusPayload {
    let mut parsed = GameStatusPayload::default();
    visit_protobuf_fields(payload, |field_number, wire_type, value| match (field_number, wire_type, value) {
        (1, 0, ProtoValue::Varint(value)) => parsed.current_round = Some(value as u32),
        (2, 0, ProtoValue::Varint(value)) => parsed.total_rounds = Some(value as u32),
        (5, 0, ProtoValue::Varint(value)) => parsed.current_stage = Some(value as u32),
        (6, 0, ProtoValue::Varint(value)) => parsed.stage_countdown_sec = Some(decode_i32_varint(value)),
        _ => {}
    });
    parsed
}

fn parse_robot_dynamic_status_payload(payload: &[u8]) -> RobotDynamicStatusPayload {
    let mut parsed = RobotDynamicStatusPayload::default();
    visit_protobuf_fields(payload, |field_number, wire_type, value| match (field_number, wire_type, value) {
        (1, 0, ProtoValue::Varint(value)) => parsed.current_health = Some(value as u32),
        (2, 5, ProtoValue::Fixed32(value)) => parsed.current_heat = Some(f32::from_bits(value)),
        (4, 0, ProtoValue::Varint(value)) => parsed.current_chassis_energy = Some(value as u32),
        (5, 0, ProtoValue::Varint(value)) => parsed.current_buffer_energy = Some(value as u32),
        (9, 0, ProtoValue::Varint(value)) => parsed.remaining_ammo = Some(value as u32),
        _ => {}
    });
    parsed
}

fn parse_robot_static_status_payload(payload: &[u8]) -> RobotStaticStatusPayload {
    let mut parsed = RobotStaticStatusPayload::default();
    visit_protobuf_fields(payload, |field_number, wire_type, value| match (field_number, wire_type, value) {
        (1, 0, ProtoValue::Varint(value)) => parsed.connection_state = Some(value as u32),
        (2, 0, ProtoValue::Varint(value)) => parsed.field_state = Some(value as u32),
        (3, 0, ProtoValue::Varint(value)) => parsed.alive_state = Some(value as u32),
        (4, 0, ProtoValue::Varint(value)) => parsed.robot_id = Some(value as u32),
        (5, 0, ProtoValue::Varint(value)) => parsed.robot_type = Some(value as u32),
        (8, 0, ProtoValue::Varint(value)) => parsed.level = Some(value as u32),
        (9, 0, ProtoValue::Varint(value)) => parsed.max_health = Some(value as u32),
        (10, 0, ProtoValue::Varint(value)) => parsed.max_heat = Some(value as u32),
        (11, 5, ProtoValue::Fixed32(value)) => parsed.heat_cooldown_rate = Some(f32::from_bits(value)),
        (12, 0, ProtoValue::Varint(value)) => parsed.max_power = Some(value as u32),
        (13, 0, ProtoValue::Varint(value)) => parsed.max_buffer_energy = Some(value as u32),
        (14, 0, ProtoValue::Varint(value)) => parsed.max_chassis_energy = Some(value as u32),
        _ => {}
    });
    parsed
}

fn parse_event_payload(payload: &[u8]) -> RefereeEventPayload {
    let mut parsed = RefereeEventPayload::default();
    visit_protobuf_fields(payload, |field_number, wire_type, value| match (field_number, wire_type, value) {
        (1, 0, ProtoValue::Varint(value)) => parsed.event_id = Some(decode_i32_varint(value)),
        (2, 2, ProtoValue::Bytes(bytes)) => {
            parsed.param = String::from_utf8(bytes).ok();
        }
        _ => {}
    });
    parsed
}

enum ProtoValue {
    Varint(u64),
    Fixed32(u32),
    Bytes(Vec<u8>),
}

fn visit_protobuf_fields(payload: &[u8], mut visitor: impl FnMut(u64, u64, ProtoValue)) {
    let mut index = 0usize;
    while index < payload.len() {
        let Some(key) = read_varint_from(payload, &mut index) else {
            return;
        };
        let field_number = key >> 3;
        let wire_type = key & 0x07;
        match wire_type {
            0 => {
                if let Some(value) = read_varint_from(payload, &mut index) {
                    visitor(field_number, wire_type, ProtoValue::Varint(value));
                } else {
                    return;
                }
            }
            2 => {
                let Some(len) = read_varint_from(payload, &mut index).map(|value| value as usize) else {
                    return;
                };
                let start = index;
                index = index.saturating_add(len);
                if index > payload.len() {
                    return;
                }
                visitor(field_number, wire_type, ProtoValue::Bytes(payload[start..index].to_vec()));
            }
            5 => {
                if index + 4 > payload.len() {
                    return;
                }
                let value = u32::from_le_bytes([
                    payload[index],
                    payload[index + 1],
                    payload[index + 2],
                    payload[index + 3],
                ]);
                index += 4;
                visitor(field_number, wire_type, ProtoValue::Fixed32(value));
            }
            _ => return,
        }
    }
}

fn read_varint_from(bytes: &[u8], index: &mut usize) -> Option<u64> {
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

fn decode_i32_varint(value: u64) -> i32 {
    value as i64 as i32
}

async fn update_input_diagnostics(
    input_diagnostics: &Arc<Mutex<InputDiagnostics>>,
    command: &KeyboardMouseInput,
) {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis() as u64;
    let mut diagnostics = input_diagnostics.lock().await;
    diagnostics.input_latency_ms = now_ms.saturating_sub(command.produced_at_ms) as f64;
    diagnostics.cmd_x = command.mouse_x;
    diagnostics.cmd_y = command.mouse_y;
    diagnostics.keyboard_value = command.keyboard_value;
    diagnostics.dry_run = command.dry_run;
}

fn encode_keyboard_mouse_control(command: &KeyboardMouseInput) -> Vec<u8> {
    let mut payload = Vec::with_capacity(32);
    write_varint_field(&mut payload, 1, command.mouse_x as i64 as u64);
    write_varint_field(&mut payload, 2, command.mouse_y as i64 as u64);
    write_varint_field(&mut payload, 3, command.mouse_z as i64 as u64);
    write_varint_field(
        &mut payload,
        4,
        u64::from(command.left_button_down && !command.disabled_fire),
    );
    write_varint_field(&mut payload, 5, u64::from(command.right_button_down));
    write_varint_field(&mut payload, 6, command.keyboard_value as u64);
    write_varint_field(&mut payload, 7, u64::from(command.mid_button_down));
    payload
}

fn write_varint_field(payload: &mut Vec<u8>, field_number: u32, value: u64) {
    write_varint(payload, ((field_number as u64) << 3) | 0);
    write_varint(payload, value);
}

fn write_varint(payload: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        payload.push((value as u8) | 0x80);
        value >>= 7;
    }
    payload.push(value as u8);
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
            h264_frames_submitted_to_decoder: custom_stats
                .h264_parser_stats
                .h264_frames_submitted_to_decoder,
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
        },
    ) {
        log::error!("emit video_stats for CustomByteBlock failed: {error:?}");
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardMouseInput {
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_z: i32,
    pub left_button_down: bool,
    pub right_button_down: bool,
    pub mid_button_down: bool,
    pub keyboard_value: u32,
    pub dry_run: bool,
    pub disabled_fire: bool,
    pub produced_at_ms: u64,
}

impl Default for KeyboardMouseInput {
    fn default() -> Self {
        Self {
            mouse_x: 0,
            mouse_y: 0,
            mouse_z: 0,
            left_button_down: false,
            right_button_down: false,
            mid_button_down: false,
            keyboard_value: 0,
            dry_run: true,
            disabled_fire: true,
            produced_at_ms: 0,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputDiagnostics {
    pub input_send_hz: f64,
    pub input_latency_ms: f64,
    pub dropped_input_frames: u64,
    pub cmd_x: i32,
    pub cmd_y: i32,
    pub keyboard_value: u32,
    pub dry_run: bool,
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
