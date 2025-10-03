use crate::get_ipv6::get_ipv6;
use crate::reporters::reporter::Reporter;
use log::error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{Instant, sleep};

pub struct DebounceManager<T: Reporter + ?Sized> {
    trigger_tx: mpsc::Sender<()>,
    shutdown_tx: mpsc::Sender<()>,
    task_handle: tokio::task::JoinHandle<()>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Reporter + ?Sized + 'static> DebounceManager<T> {
    pub fn new(debounce_duration: Duration, network_name: String, reporter: Box<dyn Reporter>) -> Self {
        let (trigger_tx, trigger_rx) = mpsc::channel(100);
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

        let task_handle = tokio::spawn(async move {
            let mut last_trigger_time = None;
            let mut trigger_rx = trigger_rx;

            loop {
                tokio::select! {
                    _ = trigger_rx.recv() => {
                        last_trigger_time = Some(Instant::now());
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                    _ = async {
                        match last_trigger_time {
                            Some(last_time) => {
                                sleep(last_time + debounce_duration - Instant::now()).await;
                                if last_trigger_time.map(|t| t == last_time).unwrap_or(false) {
                                    let ip = match get_ipv6(network_name.as_str()) {
                                        None => {
                                            error!("Get ipv6 failed");
                                            last_trigger_time = None;
                                            return;
                                        },
                                        Some(ipv6) => {ipv6}
                                    };
                                    _ = match reporter.report(ip).await {
                                        Ok(_) => {}
                                        Err(e) => {error!("Report failed: {:?}", e);}
                                    };
                                    last_trigger_time = None;
                                }
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
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn trigger(&self) {
        let _ = self.trigger_tx.send(()).await;
    }

    // 关闭后台任务
    pub async fn shutdown(self) {
        let _ = self.shutdown_tx.send(()).await;
        self.task_handle.await.ok();
    }
}
