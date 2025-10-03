use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

// 定义函数类型别名
type Callback = Box<dyn Fn() + Send + 'static>;

enum Event {
    Trigger, // 触发 A 方法的事件
    Reset,   // 重置计时器的信号
    Quit,    // 停止线程的信号
}

pub struct DebounceManager {
    trigger_sender: mpsc::Sender<Event>, // 用于触发事件
    reset_sender: mpsc::Sender<Event>,   // 用于重置计时器
    handle: Option<thread::JoinHandle<()>>,
}

impl DebounceManager {
    pub fn new<A, B>(a: A, b: B, debounce_duration: Duration) -> Self
    where
        A: Fn() + Send + 'static,
        B: Fn() + Send + 'static,
    {
        let (trigger_sender, trigger_receiver) = mpsc::channel();
        let (reset_sender, reset_receiver) = mpsc::channel();

        // 将闭包包装为 Box 以便在线程间传递
        let a_callback: Callback = Box::new(a);
        let b_callback: Callback = Box::new(b);

        // 创建防抖线程
        let handle = thread::spawn(move || {
            let mut last_trigger_time = None::<Instant>;
            let debounce_duration = debounce_duration;

            loop {
                // 计算剩余等待时间
                let wait_time = match last_trigger_time {
                    Some(time) => {
                        let elapsed = time.elapsed();
                        if elapsed >= debounce_duration {
                            // 如果已超过防抖时间，执行 B 方法
                            b_callback();
                            last_trigger_time = None;
                            continue;
                        }
                        debounce_duration - elapsed
                    }
                    None => Duration::MAX, // 长时间等待
                };

                // 等待事件或超时
                match reset_receiver.recv_timeout(wait_time) {
                    Ok(event) => match event {
                        Event::Reset => {
                            // 重置计时器
                            last_trigger_time = Some(Instant::now());
                        }
                        Event::Quit => break,
                        _ => {}
                    },
                    Err(_) => {
                        // 超时后执行 B 方法
                        if last_trigger_time.is_some() {
                            b_callback();
                        }
                        last_trigger_time = None;
                    }
                }
            }
        });

        // 创建事件监听线程
        let event_reset = reset_sender.clone();
        thread::spawn(move || {
            while let Ok(event) = trigger_receiver.recv() {
                match event {
                    Event::Trigger => {
                        // 执行 A 方法
                        a_callback();
                        // 发送重置信号
                        event_reset.send(Event::Reset).unwrap();
                    }
                    Event::Quit => break,
                    _ => {}
                }
            }
        });

        DebounceManager {
            trigger_sender,
            reset_sender,
            handle: Some(handle),
        }
    }

    // 触发事件的方法
    pub fn trigger(&self) {
        self.trigger_sender.send(Event::Trigger).unwrap();
    }
}

impl Drop for DebounceManager {
    fn drop(&mut self) {
        // 发送退出信号
        self.trigger_sender.send(Event::Quit).unwrap();
        self.reset_sender.send(Event::Quit).unwrap();

        // 等待线程结束
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}
