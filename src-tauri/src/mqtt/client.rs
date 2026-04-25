use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS};
use serde::Serialize;
use tauri::Emitter;
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};

const DEPLOY_MODE_TOPIC: &str = "DeployModeStatusSync";

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

pub fn spawn_mqtt_loop(app: tauri::AppHandle, host: String, port: u16) -> MqttRuntime {
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
