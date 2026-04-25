use crate::state::app_state::AppState;
use crate::video::decoder::spawn_decoder;
use crate::video::decoder::{MOCK_DECODER_ENABLED, REAL_DECODER_ENABLED};
use crate::video::frame_hub::FrameSnapshot;
use crate::video::mock_sender::MockVideoRuntime;
use crate::video::udp_receiver::{spawn_video_receiver_with_decoder, VideoStatsPayload};
use serde::Serialize;
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
  pub mock_decoder_enabled: bool,
}

#[tauri::command]
pub async fn start_video(
  app: tauri::AppHandle,
  state: State<'_, AppState>,
  port: u16,
) -> Result<CommandResult, String> {
  let mut slot = state.video_runtime.lock().await;
  if slot.is_some() {
    return Ok(CommandResult {
      success: true,
      message: "video receiver already running".into(),
    });
  }

  let decoder_tx = {
    let mut tx_slot = state.decoder_input_tx.lock().await;
    if tx_slot.is_none() {
      let (decoder_runtime, input_tx) = spawn_decoder(state.frame_hub.clone());
      let mut runtime_slot = state.decoder_runtime.lock().await;
      *runtime_slot = Some(decoder_runtime);
      *tx_slot = Some(input_tx);
    }
    tx_slot
      .as_ref()
      .cloned()
      .ok_or_else(|| "decoder input channel not ready".to_string())?
  };

  let runtime = spawn_video_receiver_with_decoder(app, port, decoder_tx, state.frame_hub.clone());
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
    {
      let mut tx_slot = state.decoder_input_tx.lock().await;
      *tx_slot = None;
    }
    if let Some(decoder_runtime) = state.decoder_runtime.lock().await.take() {
      decoder_runtime.stop().await;
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
    mock_decoder_enabled: MOCK_DECODER_ENABLED,
  }
}

#[tauri::command]
pub async fn reset_video_stats(
  app: tauri::AppHandle,
  state: State<'_, AppState>,
) -> Result<CommandResult, String> {
  let decoder_stats = state.frame_hub.decoder_stats().await;
  let frame_age_ms = state.frame_hub.latest_frame_age_ms().await;

  app
    .emit(
      "video_stats",
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
        decoder_reset_count: decoder_stats.decoder_reset_count,
        last_decode_cost_ms: decoder_stats.last_decode_cost_ms,
        latest_frame_age_ms: frame_age_ms,
        is_rendering_real_frame: false,
        real_decoder_enabled: REAL_DECODER_ENABLED,
        mock_decoder_enabled: MOCK_DECODER_ENABLED,
      },
    )
    .map_err(|error| format!("emit reset video stats failed: {error:?}"))?;

  Ok(CommandResult {
    success: true,
    message: "video stats reset".into(),
  })
}
