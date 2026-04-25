use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameSnapshot {
  pub version: u64,
  pub width: u32,
  pub height: u32,
  pub rgba: Vec<u8>,
  pub produced_at_ms: u128,
}

#[derive(Debug, Clone, Default)]
pub struct DecoderStats {
  pub decoder_reset_count: u64,
  pub last_decode_cost_ms: f64,
}

#[derive(Debug, Clone, Default)]
struct LatestFrame {
  version: u64,
  width: u32,
  height: u32,
  rgba: Vec<u8>,
  produced_at_ms: u128,
}

#[derive(Clone, Default)]
pub struct LatestFrameHub {
  frame: Arc<RwLock<LatestFrame>>,
  decoder_stats: Arc<RwLock<DecoderStats>>,
}

impl LatestFrameHub {
  pub fn new() -> Self {
    Self::default()
  }

  pub async fn publish_frame(&self, width: u32, height: u32, rgba: Vec<u8>, decode_cost_ms: f64) {
    {
      let mut frame = self.frame.write().await;
      frame.version += 1;
      frame.width = width;
      frame.height = height;
      frame.rgba = rgba;
      frame.produced_at_ms = now_epoch_ms();
    }

    let mut stats = self.decoder_stats.write().await;
    stats.last_decode_cost_ms = decode_cost_ms;
  }

  pub async fn snapshot_if_newer(&self, since_version: Option<u64>) -> Option<FrameSnapshot> {
    let frame = self.frame.read().await;
    if frame.version == 0 {
      return None;
    }
    if let Some(v) = since_version {
      if frame.version <= v {
        return None;
      }
    }

    Some(FrameSnapshot {
      version: frame.version,
      width: frame.width,
      height: frame.height,
      rgba: frame.rgba.clone(),
      produced_at_ms: frame.produced_at_ms,
    })
  }

  pub async fn mark_decoder_reset(&self) {
    let mut stats = self.decoder_stats.write().await;
    stats.decoder_reset_count += 1;
  }

  pub async fn decoder_stats(&self) -> DecoderStats {
    self.decoder_stats.read().await.clone()
  }

  pub async fn latest_frame_age_ms(&self) -> Option<u128> {
    let frame = self.frame.read().await;
    if frame.version == 0 {
      return None;
    }
    Some(now_epoch_ms().saturating_sub(frame.produced_at_ms))
  }

  pub async fn clear(&self) {
    let mut frame = self.frame.write().await;
    *frame = LatestFrame::default();
  }
}

fn now_epoch_ms() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or(Duration::from_secs(0))
    .as_millis()
}
