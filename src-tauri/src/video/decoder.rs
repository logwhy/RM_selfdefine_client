use crate::video::frame_hub::LatestFrameHub;
use crate::video::source::CodecMode;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, oneshot};

#[cfg(not(feature = "real-decoder"))]
use std::time::Duration;
#[cfg(not(feature = "real-decoder"))]
use tokio::time::sleep;

#[cfg(feature = "real-decoder")]
use ffmpeg_next as ffmpeg;
#[cfg(feature = "real-decoder")]
use std::sync::{
  atomic::{AtomicBool, Ordering},
  mpsc as std_mpsc,
};

pub struct DecoderRuntime {
  stop_tx: Option<oneshot::Sender<()>>,
  join_handle: tokio::task::JoinHandle<()>,
}

impl DecoderRuntime {
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

pub fn spawn_decoder(
  frame_hub: Arc<LatestFrameHub>,
  codec_mode: CodecMode,
) -> (DecoderRuntime, mpsc::Sender<Vec<u8>>) {
  #[cfg(feature = "real-decoder")]
  {
    return spawn_real_decoder(frame_hub, codec_mode);
  }
  #[cfg(not(feature = "real-decoder"))]
  {
    return spawn_stub_decoder(frame_hub, codec_mode);
  }
}

#[cfg(feature = "real-decoder")]
fn spawn_real_decoder(
  frame_hub: Arc<LatestFrameHub>,
  codec_mode: CodecMode,
) -> (DecoderRuntime, mpsc::Sender<Vec<u8>>) {
  let (tx, mut rx) = mpsc::channel::<Vec<u8>>(1);
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();

  let join_handle = tokio::spawn(async move {
    let rt_handle = tokio::runtime::Handle::current();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_for_decoder = Arc::clone(&stop_flag);
    let frame_hub_for_decoder = Arc::clone(&frame_hub);
    let (packet_tx, packet_rx) = std_mpsc::sync_channel::<Vec<u8>>(1);

    let decoder_worker = tokio::task::spawn_blocking(move || {
      run_real_decoder_loop(packet_rx, stop_flag_for_decoder, frame_hub_for_decoder, rt_handle, codec_mode);
    });

    loop {
      tokio::select! {
        _ = &mut stop_rx => {
          stop_flag.store(true, Ordering::Relaxed);
          break;
        }
        maybe_data = rx.recv() => {
          let Some(data) = maybe_data else {
            stop_flag.store(true, Ordering::Relaxed);
            break;
          };
          let mut latest = data;
          while let Ok(newer) = rx.try_recv() {
            latest = newer;
          }
          match packet_tx.try_send(latest) {
            Ok(()) => {}
            Err(std_mpsc::TrySendError::Full(_)) => {}
            Err(std_mpsc::TrySendError::Disconnected(_)) => {
              stop_flag.store(true, Ordering::Relaxed);
              break;
            }
          }
        }
      }
    }

    drop(packet_tx);
    let _ = decoder_worker.await;
  });

  (DecoderRuntime::new(stop_tx, join_handle), tx)
}

#[cfg(feature = "real-decoder")]
fn run_real_decoder_loop(
  packet_rx: std_mpsc::Receiver<Vec<u8>>,
  stop_flag: Arc<AtomicBool>,
  frame_hub: Arc<LatestFrameHub>,
  rt_handle: tokio::runtime::Handle,
  codec_mode: CodecMode,
) {
  let _ = ffmpeg::init();

  let decoder_name = codec_mode.decoder_name(true);
  let mut decoder = match create_video_decoder(codec_mode) {
    Ok(v) => {
      rt_handle.block_on(frame_hub.set_decoder_status(decoder_name.clone(), true));
      Some(v)
    }
    Err(error) => {
      log::error!("create {codec_mode:?} decoder failed: {error}");
      rt_handle.block_on(frame_hub.set_decoder_status(decoder_name.clone(), false));
      rt_handle.block_on(frame_hub.mark_decoder_reset());
      None
    }
  };
  let mut scaler: Option<ffmpeg::software::scaling::Context> = None;
  let mut decoded = ffmpeg::frame::Video::empty();

  while !stop_flag.load(Ordering::Relaxed) {
    let data = match packet_rx.recv_timeout(std::time::Duration::from_millis(20)) {
      Ok(v) => v,
      Err(std_mpsc::RecvTimeoutError::Timeout) => continue,
      Err(std_mpsc::RecvTimeoutError::Disconnected) => break,
    };

    if decoder.is_none() {
      match create_video_decoder(codec_mode) {
        Ok(value) => {
          rt_handle.block_on(frame_hub.set_decoder_status(decoder_name.clone(), true));
          decoder = Some(value);
        }
        Err(_) => {
          rt_handle.block_on(frame_hub.set_decoder_status(decoder_name.clone(), false));
          rt_handle.block_on(frame_hub.mark_decoder_reset());
          continue;
        }
      }
    }

    let start = Instant::now();
    let packet = ffmpeg::Packet::copy(&data);
    let send_ok = if let Some(decoder_ref) = decoder.as_mut() {
      decoder_ref.send_packet(&packet).is_ok()
    } else {
      false
    };

    if !send_ok {
      decoder = None;
      scaler = None;
      rt_handle.block_on(frame_hub.mark_decoder_reset());
      continue;
    }

    loop {
      let receive = if let Some(decoder_ref) = decoder.as_mut() {
        decoder_ref.receive_frame(&mut decoded)
      } else {
        break;
      };

      match receive {
        Ok(()) => {
          if scaler.is_none() {
            scaler = ffmpeg::software::scaling::Context::get(
              decoded.format(),
              decoded.width(),
              decoded.height(),
              ffmpeg::format::Pixel::RGBA,
              decoded.width(),
              decoded.height(),
              ffmpeg::software::scaling::flag::Flags::BILINEAR,
            )
            .ok();
          }

          let Some(scaler_ref) = scaler.as_mut() else {
            decoder = None;
            rt_handle.block_on(frame_hub.mark_decoder_reset());
            break;
          };

          let mut rgba = ffmpeg::frame::Video::empty();
          rgba.set_format(ffmpeg::format::Pixel::RGBA);
          rgba.set_width(decoded.width());
          rgba.set_height(decoded.height());

          if scaler_ref.run(&decoded, &mut rgba).is_err() {
            decoder = None;
            scaler = None;
            rt_handle.block_on(frame_hub.mark_decoder_reset());
            break;
          }

          let width = rgba.width() as usize;
          let height = rgba.height() as usize;
          let stride = rgba.stride(0);
          let src = rgba.data(0);
          let mut packed = vec![0u8; width * height * 4];

          for y in 0..height {
            let src_start = y * stride;
            let src_end = src_start + width * 4;
            let dst_start = y * width * 4;
            let dst_end = dst_start + width * 4;
            packed[dst_start..dst_end].copy_from_slice(&src[src_start..src_end]);
          }

          rt_handle.block_on(
            frame_hub.publish_frame(
              width as u32,
              height as u32,
              packed,
              start.elapsed().as_secs_f64() * 1000.0,
            ),
          );
        }
        Err(_) => {
          break;
        }
      }
    }
  }
}

#[cfg(not(feature = "real-decoder"))]
fn spawn_stub_decoder(
  frame_hub: Arc<LatestFrameHub>,
  codec_mode: CodecMode,
) -> (DecoderRuntime, mpsc::Sender<Vec<u8>>) {
  let (tx, mut rx) = mpsc::channel::<Vec<u8>>(1);
  let (stop_tx, mut stop_rx) = oneshot::channel::<()>();

  let join_handle = tokio::spawn(async move {
    frame_hub
      .set_decoder_status(codec_mode.decoder_name(false), true)
      .await;
    let mut tick: u8 = 0;
    loop {
      tokio::select! {
        _ = &mut stop_rx => {
          break;
        }
        maybe_data = rx.recv() => {
          let Some(_data) = maybe_data else {
            break;
          };

          let start = Instant::now();
          let width = 640usize;
          let height = 360usize;
          let mut rgba = vec![0u8; width * height * 4];
          for y in 0..height {
            for x in 0..width {
              let i = (y * width + x) * 4;
              rgba[i] = tick;
              rgba[i + 1] = (x as u8).wrapping_add(tick / 2);
              rgba[i + 2] = (y as u8).wrapping_add(tick / 3);
              rgba[i + 3] = 255;
            }
          }
          tick = tick.wrapping_add(3);

          frame_hub
            .publish_frame(width as u32, height as u32, rgba, start.elapsed().as_secs_f64() * 1000.0)
            .await;
        }
        _ = sleep(Duration::from_millis(5)) => {}
      }
    }
  });

  (DecoderRuntime::new(stop_tx, join_handle), tx)
}

#[cfg(feature = "real-decoder")]
fn create_video_decoder(codec_mode: CodecMode) -> Result<ffmpeg::decoder::Video, ffmpeg::Error> {
  let codec_id = match codec_mode {
    CodecMode::Auto | CodecMode::Hevc => ffmpeg::codec::Id::HEVC,
    CodecMode::H264 => ffmpeg::codec::Id::H264,
  };
  let codec = ffmpeg::codec::decoder::find(codec_id).ok_or(ffmpeg::Error::DecoderNotFound)?;
  let context = ffmpeg::codec::Context::new_with_codec(codec);
  context.decoder().video()
}

pub const REAL_DECODER_ENABLED: bool = cfg!(feature = "real-decoder");
pub const MOCK_DECODER_ENABLED: bool = !REAL_DECODER_ENABLED;
