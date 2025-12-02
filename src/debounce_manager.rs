use log::{debug, info};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{Instant, sleep};

pub(crate) struct DebounceManager {
    trigger_tx: mpsc::Sender<()>,
    shutdown_tx: mpsc::Sender<()>,
    task_handle: tokio::task::JoinHandle<()>,
}

impl DebounceManager {
    pub fn new<F, Fut>(closer: F, debounce_duration: Duration) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let (trigger_tx, trigger_rx) = mpsc::channel(100);
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

        let task_handle = tokio::spawn(async move {
            let mut last_trigger_time = None;
            let mut trigger_rx = trigger_rx;

            loop {
                tokio::select! {
                    _ = trigger_rx.recv() => {
                        last_trigger_time = Some(Instant::now());
                        debug!("trigger_rx received");
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                    _ = async {
                        match last_trigger_time {
                            Some(last_time) => {
                                sleep(last_time + debounce_duration - Instant::now()).await;
                                closer().await;
                                last_trigger_time = None;
                            }
                            None => {
                                // 如果没有触发事件，等待更长时间
                                sleep(Duration::MAX).await;
                            }
                        }
                    } => {}
                }
            }
        });

        Self {
            trigger_tx,
            shutdown_tx,
            task_handle,
        }
    }

    // 触发发送事件
    pub async fn trigger(&self) {
        let _ = self.trigger_tx.send(()).await;
    }

    pub fn blocking_trigger(&self) {
        let _ = self.trigger_tx.blocking_send(());
    }

    #[allow(dead_code)]
    // 关闭后台任务
    pub async fn shutdown(self) {
        let _ = self.shutdown_tx.send(()).await;
        self.task_handle.await.ok();
    }
}
