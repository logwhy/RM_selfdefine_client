use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientMode {
  Normal,
  HeroLob,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoSource {
  UdpHevc,
  #[serde(rename = "custombyteblock_h264")]
  CustomByteBlockH264,
  Mock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodecMode {
  Auto,
  Hevc,
  H264,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomBlockParserMode {
  #[serde(rename = "raw_annexb_stream")]
  RawAnnexBStream,
  PacketizedFrame,
}

#[derive(Debug, Clone)]
pub struct VideoPipelineConfig {
  pub current_mode: ClientMode,
  pub current_video_source: VideoSource,
  pub current_codec_mode: CodecMode,
  pub custom_block_parser_mode: CustomBlockParserMode,
}

impl Default for VideoPipelineConfig {
  fn default() -> Self {
    Self {
      current_mode: ClientMode::Normal,
      current_video_source: VideoSource::UdpHevc,
      current_codec_mode: CodecMode::Hevc,
      custom_block_parser_mode: CustomBlockParserMode::RawAnnexBStream,
    }
  }
}

impl CodecMode {
  pub fn decoder_name(self, real_decoder_enabled: bool) -> String {
    if !real_decoder_enabled {
      return "stub-decoder".into();
    }

    match self {
      CodecMode::Auto => "ffmpeg-auto-hevc".into(),
      CodecMode::Hevc => "ffmpeg-hevc".into(),
      CodecMode::H264 => "ffmpeg-h264".into(),
    }
  }
}
