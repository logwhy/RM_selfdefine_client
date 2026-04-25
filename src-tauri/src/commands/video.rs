use crate::state::app_state::AppState;
use crate::video::custom_block_receiver::handle_custom_block_data;
use crate::video::decoder::spawn_decoder;
use crate::video::decoder::{MOCK_DECODER_ENABLED, REAL_DECODER_ENABLED};
use crate::video::frame_hub::FrameSnapshot;
use crate::video::mock_sender::MockVideoRuntime;
use crate::video::source::{
  ClientMode, CodecMode, CustomBlockParserMode, VideoPipelineConfig, VideoSource,
};
use crate::video::udp_receiver::{spawn_video_receiver_with_decoder, VideoStatsPayload};
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri::State;
use tokio::net::UdpSocket;
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
  pub success: bool,
  pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecoderModeResult {
  pub real_decoder_enabled: bool,
  pub stub_decoder_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoPipelineConfigInput {
  pub current_mode: ClientMode,
  pub current_video_source: VideoSource,
  pub current_codec_mode: CodecMode,
  pub custom_block_parser_mode: CustomBlockParserMode,
}

async fn ensure_decoder(
  state: &State<'_, AppState>,
  codec_mode: CodecMode,
) -> Result<tokio::sync::mpsc::Sender<Vec<u8>>, String> {
  let existing_codec = *state.decoder_codec_mode.lock().await;
  if existing_codec.is_some() && existing_codec != Some(codec_mode) {
    {
      let mut tx_slot = state.decoder_input_tx.lock().await;
      *tx_slot = None;
    }
    if let Some(decoder_runtime) = state.decoder_runtime.lock().await.take() {
      decoder_runtime.stop().await;
    }
    *state.decoder_codec_mode.lock().await = None;
  }

  let mut tx_slot = state.decoder_input_tx.lock().await;
  if tx_slot.is_none() {
    let (decoder_runtime, input_tx) = spawn_decoder(state.frame_hub.clone(), codec_mode);
    let mut runtime_slot = state.decoder_runtime.lock().await;
    *runtime_slot = Some(decoder_runtime);
    *tx_slot = Some(input_tx);
    *state.decoder_codec_mode.lock().await = Some(codec_mode);
  }

  tx_slot
    .as_ref()
    .cloned()
    .ok_or_else(|| "decoder input channel not ready".to_string())
}

async fn stop_decoder_if_idle(state: &State<'_, AppState>) {
  if state.video_runtime.lock().await.is_some() {
    return;
  }
  if state.mock_hero_lob_runtime.lock().await.is_some() {
    return;
  }

  {
    let mut tx_slot = state.decoder_input_tx.lock().await;
    *tx_slot = None;
  }
  if let Some(decoder_runtime) = state.decoder_runtime.lock().await.take() {
    decoder_runtime.stop().await;
  }
  *state.decoder_codec_mode.lock().await = None;
}

async fn emit_pipeline_stats(app: &tauri::AppHandle, state: &State<'_, AppState>) -> Result<(), String> {
  let decoder_stats = state.frame_hub.decoder_stats().await;
  let frame_age_ms = state.frame_hub.latest_frame_age_ms().await;
  let config = state.video_config.lock().await.clone();
  let custom_stats = state
    .custom_block_stats
    .lock()
    .await
    .payload(config.custom_block_parser_mode);

  app
    .emit(
      "video_stats",
      VideoStatsPayload {
        stream_alive: state.custom_block_stats.lock().await.stream_alive(),
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
    )
    .map_err(|error| format!("emit video stats failed: {error:?}"))
}

fn decoder_name_or_default(current_decoder_name: String, codec_mode: CodecMode) -> String {
  if current_decoder_name.trim().is_empty() {
    return codec_mode.decoder_name(REAL_DECODER_ENABLED);
  }
  current_decoder_name
}

#[tauri::command]
pub async fn start_video(
  app: tauri::AppHandle,
  state: State<'_, AppState>,
  port: u16,
  codec_mode: Option<CodecMode>,
) -> Result<CommandResult, String> {
  let mut slot = state.video_runtime.lock().await;
  if slot.is_some() {
    return Ok(CommandResult {
      success: true,
      message: "video receiver already running".into(),
    });
  }

  let selected_codec = match codec_mode.unwrap_or(CodecMode::Hevc) {
    CodecMode::H264 => CodecMode::Hevc,
    other => other,
  };
  {
    let mut config = state.video_config.lock().await;
    config.current_mode = ClientMode::Normal;
    config.current_video_source = VideoSource::UdpHevc;
    config.current_codec_mode = selected_codec;
  }

  let decoder_tx = ensure_decoder(&state, selected_codec).await?;

  let runtime = spawn_video_receiver_with_decoder(
    app,
    port,
    decoder_tx,
    state.frame_hub.clone(),
    state.video_config.clone(),
    state.custom_block_stats.clone(),
  );
  *slot = Some(runtime);

  Ok(CommandResult {
    success: true,
    message: format!("video receiver started on UDP {port}"),
  })
}

#[tauri::command]
pub async fn stop_video(state: State<'_, AppState>) -> Result<CommandResult, String> {
  let mut slot = state.video_runtime.lock().await;
  if let Some(runtime) = slot.take() {
    drop(slot);
    runtime.stop().await;
    let keep_decoder_for_custom_blocks = state
      .video_config
      .lock()
      .await
      .current_video_source
      == VideoSource::CustomByteBlockH264;
    if !keep_decoder_for_custom_blocks {
      stop_decoder_if_idle(&state).await;
    }
    state.frame_hub.clear().await;
    Ok(CommandResult {
      success: true,
      message: "video receiver stopped".into(),
    })
  } else {
    Ok(CommandResult {
      success: true,
      message: "video receiver is not running".into(),
    })
  }
}

#[tauri::command]
pub async fn start_mock_video_source(
  state: State<'_, AppState>,
  port: u16,
) -> Result<CommandResult, String> {
  let mut slot = state.mock_video_runtime.lock().await;
  if slot.is_some() {
    return Ok(CommandResult {
      success: true,
      message: "mock video source already running".into(),
    });
  }

  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let join_handle = tokio::spawn(async move {
    let socket = match UdpSocket::bind("127.0.0.1:0").await {
      Ok(s) => s,
      Err(error) => {
        log::error!("bind mock udp sender failed: {error:?}");
        return;
      }
    };

    let target = format!("127.0.0.1:{port}");
    let mut frame_id: u16 = 1;

    loop {
      tokio::select! {
        _ = &mut stop_rx => {
          break;
        }
        _ = sleep(Duration::from_millis(33)) => {
          let frame_total_bytes: usize = 1500;
          let fragment_count: usize = 3;
          let fragment_payload = frame_total_bytes / fragment_count;

          for fragment_index in 0..fragment_count {
            let mut packet = Vec::with_capacity(8 + fragment_payload);
            packet.extend_from_slice(&frame_id.to_be_bytes());
            packet.extend_from_slice(&(fragment_index as u16).to_be_bytes());
            packet.extend_from_slice(&(frame_total_bytes as u32).to_be_bytes());
            packet.extend(std::iter::repeat(fragment_index as u8).take(fragment_payload));
            let _ = socket.send_to(&packet, &target).await;
          }

          frame_id = frame_id.wrapping_add(1);
        }
      }
    }
  });

  *slot = Some(MockVideoRuntime::new(stop_tx, join_handle));
  Ok(CommandResult {
    success: true,
    message: "mock video source started".into(),
  })
}

#[tauri::command]
pub async fn start_mock_video(
  state: State<'_, AppState>,
  port: u16,
) -> Result<CommandResult, String> {
  start_mock_video_source(state, port).await
}

#[tauri::command]
pub async fn stop_mock_video_source(state: State<'_, AppState>) -> Result<CommandResult, String> {
  let mut slot = state.mock_video_runtime.lock().await;
  if let Some(runtime) = slot.take() {
    drop(slot);
    runtime.stop().await;
    Ok(CommandResult {
      success: true,
      message: "mock video source stopped".into(),
    })
  } else {
    Ok(CommandResult {
      success: true,
      message: "mock video source is not running".into(),
    })
  }
}

#[tauri::command]
pub async fn stop_mock_video(state: State<'_, AppState>) -> Result<CommandResult, String> {
  stop_mock_video_source(state).await
}

#[tauri::command]
pub async fn set_video_pipeline_config(
  state: State<'_, AppState>,
  config: VideoPipelineConfigInput,
) -> Result<CommandResult, String> {
  {
    let mut current = state.video_config.lock().await;
    *current = VideoPipelineConfig {
      current_mode: config.current_mode,
      current_video_source: config.current_video_source,
      current_codec_mode: config.current_codec_mode,
      custom_block_parser_mode: config.custom_block_parser_mode,
    };
  }

  if config.current_video_source == VideoSource::CustomByteBlockH264 {
    let _ = ensure_decoder(&state, CodecMode::H264).await?;
  }

  Ok(CommandResult {
    success: true,
    message: "video pipeline config updated".into(),
  })
}

#[tauri::command]
pub async fn start_hero_lob_video(state: State<'_, AppState>) -> Result<CommandResult, String> {
  {
    let mut config = state.video_config.lock().await;
    config.current_mode = ClientMode::HeroLob;
    config.current_video_source = VideoSource::CustomByteBlockH264;
    config.current_codec_mode = CodecMode::H264;
  }
  let _ = ensure_decoder(&state, CodecMode::H264).await?;

  Ok(CommandResult {
    success: true,
    message: "hero lob H264 CustomByteBlock receiver armed".into(),
  })
}

#[tauri::command]
pub async fn start_mock_hero_lob_h264(
  app: tauri::AppHandle,
  state: State<'_, AppState>,
) -> Result<CommandResult, String> {
  let mut slot = state.mock_hero_lob_runtime.lock().await;
  if slot.is_some() {
    return Ok(CommandResult {
      success: true,
      message: "mock hero lob H264 source already running".into(),
    });
  }

  {
    let mut config = state.video_config.lock().await;
    config.current_mode = ClientMode::HeroLob;
    config.current_video_source = VideoSource::CustomByteBlockH264;
    config.current_codec_mode = CodecMode::H264;
  }
  let _ = ensure_decoder(&state, CodecMode::H264).await?;
  state.custom_block_stats.lock().await.mock_active = true;

  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
  let decoder_input_tx = state.decoder_input_tx.clone();
  let config = state.video_config.clone();
  let stats = state.custom_block_stats.clone();
  let reassembler = state.custom_block_reassembler.clone();
  let frame_hub = state.frame_hub.clone();
  let app_for_task = app.clone();

  let join_handle = tokio::spawn(async move {
    let mut tick: u8 = 0;
    loop {
      tokio::select! {
        _ = &mut stop_rx => {
          break;
        }
        _ = sleep(Duration::from_millis(20)) => {
          let mut data = Vec::with_capacity(300);
          data.extend_from_slice(&[0x00, 0x00, 0x00, 0x01, 0x09, 0x10]);
          data.extend(std::iter::repeat(tick).take(294));
          tick = tick.wrapping_add(1);
          handle_custom_block_data(
            &data,
            decoder_input_tx.clone(),
            config.clone(),
            stats.clone(),
            reassembler.clone(),
          ).await;
          let decoder_stats = frame_hub.decoder_stats().await;
          let frame_age_ms = frame_hub.latest_frame_age_ms().await;
          let pipeline_config = config.lock().await.clone();
          let custom_stats = stats.lock().await.payload(pipeline_config.custom_block_parser_mode);
          let _ = app_for_task.emit("video_stats", VideoStatsPayload {
            stream_alive: true,
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
            custom_block_parser_mode: custom_stats.custom_block_parser_mode,
            custom_block_mock_active: custom_stats.custom_block_mock_active,
          });
        }
      }
    }
  });

  *slot = Some(MockVideoRuntime::new(stop_tx, join_handle));
  emit_pipeline_stats(&app, &state).await?;
  Ok(CommandResult {
    success: true,
    message: "mock hero lob H264 CustomByteBlock source started".into(),
  })
}

#[tauri::command]
pub async fn stop_mock_hero_lob_h264(
  state: State<'_, AppState>,
) -> Result<CommandResult, String> {
  let mut slot = state.mock_hero_lob_runtime.lock().await;
  if let Some(runtime) = slot.take() {
    drop(slot);
    runtime.stop().await;
    state.custom_block_stats.lock().await.mock_active = false;
    stop_decoder_if_idle(&state).await;
    Ok(CommandResult {
      success: true,
      message: "mock hero lob H264 source stopped".into(),
    })
  } else {
    Ok(CommandResult {
      success: true,
      message: "mock hero lob H264 source is not running".into(),
    })
  }
}

#[tauri::command]
pub async fn get_latest_frame(
  state: State<'_, AppState>,
  since_version: Option<u64>,
) -> Result<Option<FrameSnapshot>, String> {
  Ok(state.frame_hub.snapshot_if_newer(since_version).await)
}

#[tauri::command]
pub fn get_decoder_mode() -> DecoderModeResult {
  DecoderModeResult {
    real_decoder_enabled: REAL_DECODER_ENABLED,
    stub_decoder_enabled: MOCK_DECODER_ENABLED,
  }
}

#[tauri::command]
pub async fn reset_video_stats(
  app: tauri::AppHandle,
  state: State<'_, AppState>,
) -> Result<CommandResult, String> {
  emit_pipeline_stats(&app, &state).await?;

  Ok(CommandResult {
    success: true,
    message: "video stats reset".into(),
  })
}
