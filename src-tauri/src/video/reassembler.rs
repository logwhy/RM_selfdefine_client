use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct ReassemblerStats {
  pub active_frames: usize,
  pub timeout_dropped_frames: u64,
  pub incomplete_frames: u64,
  pub ready_frames: u64,
}

#[derive(Debug, Clone)]
struct PartialFrame {
  frame_total_bytes: u32,
  fragments: HashMap<u16, Vec<u8>>,
  collected_bytes: usize,
  updated_at: Instant,
}

impl PartialFrame {
  fn new(frame_total_bytes: u32) -> Self {
    Self {
      frame_total_bytes,
      fragments: HashMap::new(),
      collected_bytes: 0,
      updated_at: Instant::now(),
    }
  }
}

pub struct FrameReassembler {
  frames: HashMap<u16, PartialFrame>,
  timeout: Duration,
  stats: ReassemblerStats,
  ready_timestamps: VecDeque<Instant>,
}

impl FrameReassembler {
  pub fn new(timeout: Duration) -> Self {
    Self {
      frames: HashMap::new(),
      timeout,
      stats: ReassemblerStats::default(),
      ready_timestamps: VecDeque::new(),
    }
  }

  pub fn push_fragment(
    &mut self,
    frame_id: u16,
    fragment_index: u16,
    frame_total_bytes: u32,
    payload: &[u8],
  ) -> Option<Vec<u8>> {
    let frame = self
      .frames
      .entry(frame_id)
      .or_insert_with(|| PartialFrame::new(frame_total_bytes));

    frame.updated_at = Instant::now();
    frame.frame_total_bytes = frame_total_bytes;

    if frame
      .fragments
      .insert(fragment_index, payload.to_vec())
      .is_none()
    {
      frame.collected_bytes += payload.len();
    }

    let ready = frame.collected_bytes >= frame.frame_total_bytes as usize;
    if ready {
      let completed = self.frames.remove(&frame_id);
      self.stats.ready_frames += 1;
      self.ready_timestamps.push_back(Instant::now());
      if let Some(frame) = completed {
        let mut ordered_indices = frame.fragments.keys().copied().collect::<Vec<_>>();
        ordered_indices.sort_unstable();
        let mut merged = Vec::with_capacity(frame.collected_bytes);
        for idx in ordered_indices {
          if let Some(chunk) = frame.fragments.get(&idx) {
            merged.extend_from_slice(chunk);
          }
        }
        self.stats.active_frames = self.frames.len();
        return Some(merged);
      }
    }

    self.stats.active_frames = self.frames.len();
    None
  }

  pub fn sweep_timeouts(&mut self) {
    let now = Instant::now();
    let mut expired_keys = Vec::new();

    for (frame_id, frame) in &self.frames {
      if now.duration_since(frame.updated_at) > self.timeout {
        expired_keys.push(*frame_id);
      }
    }

    for frame_id in expired_keys {
      self.frames.remove(&frame_id);
      self.stats.timeout_dropped_frames += 1;
      self.stats.incomplete_frames += 1;
    }

    while let Some(ts) = self.ready_timestamps.front() {
      if now.duration_since(*ts) > Duration::from_secs(1) {
        self.ready_timestamps.pop_front();
      } else {
        break;
      }
    }

    self.stats.active_frames = self.frames.len();
  }

  pub fn fps_estimate(&self) -> f64 {
    self.ready_timestamps.len() as f64
  }

  pub fn stats(&self) -> ReassemblerStats {
    self.stats.clone()
  }
}
