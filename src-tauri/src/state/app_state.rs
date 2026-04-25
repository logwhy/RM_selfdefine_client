use crate::mqtt::client::MqttRuntime;
use crate::video::custom_block_reassembler::H264Reassembler;
use crate::video::custom_block_receiver::CustomBlockStats;
use crate::video::decoder::DecoderRuntime;
use crate::video::frame_hub::LatestFrameHub;
use crate::video::mock_sender::MockVideoRuntime;
use crate::video::source::VideoPipelineConfig;
use crate::video::udp_receiver::VideoRuntime;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub struct AppState {
  pub mqtt_runtime: Mutex<Option<MqttRuntime>>,
  pub mock_deploy_mode_active: Mutex<bool>,
  pub video_runtime: Mutex<Option<VideoRuntime>>,
  pub mock_video_runtime: Mutex<Option<MockVideoRuntime>>,
  pub mock_hero_lob_runtime: Mutex<Option<MockVideoRuntime>>,
  pub decoder_runtime: Mutex<Option<DecoderRuntime>>,
  pub decoder_codec_mode: Mutex<Option<crate::video::source::CodecMode>>,
  pub decoder_input_tx: Arc<Mutex<Option<mpsc::Sender<Vec<u8>>>>>,
  pub frame_hub: Arc<LatestFrameHub>,
  pub video_config: Arc<Mutex<VideoPipelineConfig>>,
  pub custom_block_stats: Arc<Mutex<CustomBlockStats>>,
  pub custom_block_reassembler: Arc<Mutex<H264Reassembler>>,
}

impl Default for AppState {
  fn default() -> Self {
    Self {
      mqtt_runtime: Mutex::new(None),
      mock_deploy_mode_active: Mutex::new(false),
      video_runtime: Mutex::new(None),
      mock_video_runtime: Mutex::new(None),
      mock_hero_lob_runtime: Mutex::new(None),
      decoder_runtime: Mutex::new(None),
      decoder_codec_mode: Mutex::new(None),
      decoder_input_tx: Arc::new(Mutex::new(None)),
      frame_hub: Arc::new(LatestFrameHub::new()),
      video_config: Arc::new(Mutex::new(VideoPipelineConfig::default())),
      custom_block_stats: Arc::new(Mutex::new(CustomBlockStats::default())),
      custom_block_reassembler: Arc::new(Mutex::new(H264Reassembler::new())),
    }
  }
}
