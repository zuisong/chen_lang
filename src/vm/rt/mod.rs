use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::rc::Rc;
use std::time::Duration;

use crate::promise::{Promise, PromiseState};
use crate::value::Value;
use crate::vm::{Fiber, VMRuntimeError};

type FiberRef = Rc<RefCell<Fiber>>;
type ReadyTask = (FiberRef, Result<Value, VMRuntimeError>);
type ReadyQueue = Rc<RefCell<VecDeque<ReadyTask>>>;
type PromiseRef = Rc<RefCell<Promise>>;
type CompletedPromise = (PromiseRef, PromiseState);
type CompletedPromiseQueue = Rc<RefCell<VecDeque<CompletedPromise>>>;

/// 异步运行时状态
pub struct AsyncState {
    /// 待恢复的任务队列 (Fiber, ResumeValue)
    pub ready_queue: ReadyQueue,
    /// 已完成的原生异步 Promise，由 VM 主循环统一结算
    pub completed_promises: CompletedPromiseQueue,
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
            completed_promises: Rc::new(RefCell::new(VecDeque::new())),
            pending_tasks: Rc::new(RefCell::new(0)),
            notify: Rc::new(tokio::sync::Notify::new()),
        }
    }

    pub fn spawn_promise_task<F>(&self, promise_rc: PromiseRef, fut: F)
    where
        F: Future<Output = Result<Value, VMRuntimeError>> + 'static,
    {
        let completed_promises = self.completed_promises.clone();
        let pending = self.pending_tasks.clone();
        *pending.borrow_mut() += 1;

        let notify = self.notify.clone();

        let task = async move {
            let result = fut.await;
            let state = match result {
                Ok(value) => PromiseState::Fulfilled(value),
                Err(err) => PromiseState::Rejected(Value::string(err.to_string())),
            };

            completed_promises.borrow_mut().push_back((promise_rc, state));
            *pending.borrow_mut() -= 1;
            notify.notify_one();
        };

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);

        #[cfg(not(target_arch = "wasm32"))]
        tokio::task::spawn_local(task);
    }

    pub fn enqueue_pending_fiber(&self, fiber: FiberRef, resume_value: Result<Value, VMRuntimeError>) {
        self.ready_queue.borrow_mut().push_back((fiber, resume_value));
        *self.pending_tasks.borrow_mut() += 1;
        self.notify.notify_one();
    }

    pub fn drain_completed_promises(&self) -> Vec<CompletedPromise> {
        self.completed_promises.borrow_mut().drain(..).collect()
    }
}

pub async fn sleep(duration: Duration) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::time::sleep(duration).await;
    }

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::prelude::wasm_bindgen;

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = globalThis, js_name = setTimeout)]
            fn set_timeout(handler: &js_sys::Function, timeout: i32) -> i32;
        }

        let timeout = duration.as_millis().min(i32::MAX as u128) as i32;
        let js_promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let callback = Closure::once(move || {
                let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
            });
            set_timeout(callback.as_ref().unchecked_ref(), timeout);
            callback.forget();
        });

        let _ = wasm_bindgen_futures::JsFuture::from(js_promise).await;
    }
}
