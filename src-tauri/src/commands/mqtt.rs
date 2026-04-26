use crate::mqtt::client::{
    spawn_mqtt_loop, InputDiagnostics, KeyboardMouseInput, ModeSyncEventPayload,
};
use crate::state::app_state::AppState;
use serde::Serialize;
use tauri::Emitter;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
pub async fn connect_mqtt(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    host: String,
    port: u16,
) -> Result<CommandResult, String> {
    let mut runtime_slot = state.mqtt_runtime.lock().await;

    if let Some(runtime) = runtime_slot.take() {
        if runtime.endpoint_matches(&host, port) {
            *runtime_slot = Some(runtime);
            app.emit(
                "mode_sync",
                ModeSyncEventPayload {
                    mqtt_connected: true,
                    mqtt_host: Some(host.clone()),
                    mqtt_port: Some(port),
                    deploy_mode_active: None,
                    last_mode_sync_at: None,
                },
            )
            .map_err(|error| format!("emit mode sync failed: {error:?}"))?;
            return Ok(CommandResult {
                success: true,
                message: format!("MQTT already connected to {host}:{port}"),
            });
        }
        drop(runtime_slot);
        runtime.stop().await;
        runtime_slot = state.mqtt_runtime.lock().await;
    }

    let runtime = spawn_mqtt_loop(
        app,
        host.clone(),
        port,
        state.decoder_input_tx.clone(),
        state.video_config.clone(),
        state.custom_block_stats.clone(),
        state.custom_block_reassembler.clone(),
        state.frame_hub.clone(),
        state.input_tx.subscribe(),
        state.input_diagnostics.clone(),
    );
    *runtime_slot = Some(runtime);

    Ok(CommandResult {
        success: true,
        message: format!("MQTT connecting started: {host}:{port}"),
    })
}

#[tauri::command]
pub async fn submit_keyboard_mouse_control(
    state: State<'_, AppState>,
    command: KeyboardMouseInput,
) -> Result<CommandResult, String> {
    state
        .input_tx
        .send(command)
        .map_err(|error| format!("input latest-value channel closed: {error:?}"))?;

    Ok(CommandResult {
        success: true,
        message: "KeyboardMouseControl latest value accepted".into(),
    })
}

#[tauri::command]
pub async fn send_zero_keyboard_mouse_control(
    state: State<'_, AppState>,
) -> Result<CommandResult, String> {
    let mut command = KeyboardMouseInput::default();
    command.produced_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_millis() as u64;

    state
        .input_tx
        .send(command)
        .map_err(|error| format!("input latest-value channel closed: {error:?}"))?;

    Ok(CommandResult {
        success: true,
        message: "zero KeyboardMouseControl sent to latest-value channel".into(),
    })
}

#[tauri::command]
pub async fn get_input_diagnostics(state: State<'_, AppState>) -> Result<InputDiagnostics, String> {
    Ok(state.input_diagnostics.lock().await.clone())
}

#[tauri::command]
pub async fn disconnect_mqtt(state: State<'_, AppState>) -> Result<CommandResult, String> {
    let mut runtime_slot = state.mqtt_runtime.lock().await;

    if let Some(runtime) = runtime_slot.take() {
        drop(runtime_slot);
        runtime.stop().await;
        Ok(CommandResult {
            success: true,
            message: "MQTT disconnected".into(),
        })
    } else {
        Ok(CommandResult {
            success: true,
            message: "MQTT is not connected".into(),
        })
    }
}

#[tauri::command]
pub async fn emit_mock_mode_sync(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<CommandResult, String> {
    let mut mock_active = state.mock_deploy_mode_active.lock().await;
    *mock_active = !*mock_active;
    let mqtt_connected = state.mqtt_runtime.lock().await.is_some();

    let payload = ModeSyncEventPayload {
        mqtt_connected,
        mqtt_host: None,
        mqtt_port: None,
        deploy_mode_active: Some(*mock_active),
        last_mode_sync_at: Some(format!(
            "{}.{:03}Z",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_secs(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .subsec_millis()
        )),
    };

    app.emit("mode_sync", payload)
        .map_err(|error| format!("emit mock mode sync failed: {error:?}"))?;

    Ok(CommandResult {
        success: true,
        message: "Mock DeployModeStatusSync emitted".into(),
    })
}
