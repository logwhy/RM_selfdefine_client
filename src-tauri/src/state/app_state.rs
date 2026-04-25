use crate::mqtt::client::MqttRuntime;
use crate::video::decoder::DecoderRuntime;
use crate::video::frame_hub::LatestFrameHub;
use crate::video::udp_receiver::VideoRuntime;
use crate::video::mock_sender::MockVideoRuntime;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub struct AppState {
  pub mqtt_runtime: Mutex<Option<MqttRuntime>>,
  pub mock_deploy_mode_active: Mutex<bool>,
  pub video_runtime: Mutex<Option<VideoRuntime>>,
  pub mock_video_runtime: Mutex<Option<MockVideoRuntime>>,
  pub decoder_runtime: Mutex<Option<DecoderRuntime>>,
  pub decoder_input_tx: Mutex<Option<mpsc::Sender<Vec<u8>>>>,
  pub frame_hub: Arc<LatestFrameHub>,
}

impl Default for AppState {
  fn default() -> Self {
    Self {
      mqtt_runtime: Mutex::new(None),
      mock_deploy_mode_active: Mutex::new(false),
      video_runtime: Mutex::new(None),
      mock_video_runtime: Mutex::new(None),
      decoder_runtime: Mutex::new(None),
      decoder_input_tx: Mutex::new(None),
      frame_hub: Arc::new(LatestFrameHub::new()),
    }
  }
}
