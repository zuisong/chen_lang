use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::rc::Rc;
use std::time::Duration;

use crate::value::Value;
use crate::vm::{Fiber, VMRuntimeError};

type FiberRef = Rc<RefCell<Fiber>>;
type ReadyTask = (FiberRef, Result<Value, VMRuntimeError>);
type ReadyQueue = Rc<RefCell<VecDeque<ReadyTask>>>;

/// 异步运行时状态
pub struct AsyncState {
    /// 待恢复的任务队列 (Fiber, ResumeValue)
    pub ready_queue: ReadyQueue,
    /// 对待处理任务的计数
    pub pending_tasks: Rc<RefCell<usize>>,
    pub notify: Rc<tokio::sync::Notify>,
}

impl Default for AsyncState {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncState {
    pub fn new() -> Self {
        Self {
            ready_queue: Rc::new(RefCell::new(VecDeque::new())),
            pending_tasks: Rc::new(RefCell::new(0)),
            notify: Rc::new(tokio::sync::Notify::new()),
        }
    }

    /// 注册一个延时任务
    pub fn spawn_sleep(&self, duration: Duration, fiber: Rc<RefCell<Fiber>>) {
        let queue = self.ready_queue.clone();
        let pending = self.pending_tasks.clone();
        *pending.borrow_mut() += 1;

        let notify = self.notify.clone();

        #[cfg(not(target_arch = "wasm32"))]
        tokio::task::spawn_local(async move {
            tokio::time::sleep(duration).await;
            // 唤醒：将 Fiber 加入就绪队列，Resume 值为 null
            queue.borrow_mut().push_back((fiber, Ok(Value::null())));
            *pending.borrow_mut() -= 1;
            notify.notify_one();
        });

        #[cfg(target_arch = "wasm32")]
        {
            // TODO: Implant timer for WASM
            // For now just decrement pending to avoid hanging if we ever called this
            *pending.borrow_mut() -= 1;
            notify.notify_one();
        }
    }

    /// 注册一个通用的 Future 任务
    pub fn spawn_future<F>(&self, fut: F, fiber: Rc<RefCell<Fiber>>)
    where
        F: Future<Output = Result<Value, VMRuntimeError>> + 'static,
    {
        let queue = self.ready_queue.clone();
        let pending = self.pending_tasks.clone();
        *pending.borrow_mut() += 1;

        let notify = self.notify.clone();

        #[cfg(not(target_arch = "wasm32"))]
        tokio::task::spawn_local(async move {
            let result = fut.await;
            queue.borrow_mut().push_back((fiber, result));
            *pending.borrow_mut() -= 1;
            notify.notify_one();
        });

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let result = fut.await;
            queue.borrow_mut().push_back((fiber, result));
            *pending.borrow_mut() -= 1;
            notify.notify_one();
        });
    }
}
