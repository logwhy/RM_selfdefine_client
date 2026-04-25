use tokio::sync::oneshot;

pub struct MockVideoRuntime {
  stop_tx: Option<oneshot::Sender<()>>,
  join_handle: tokio::task::JoinHandle<()>,
}

impl MockVideoRuntime {
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
