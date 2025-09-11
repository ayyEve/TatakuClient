use crate::*;
use std::sync::atomic::{ AtomicBool, Ordering };

#[derive(Clone)]
pub struct AsyncLoader<T> {
    value: Arc<Mutex<Option<T>>>,
    written: Arc<AtomicBool>,
    abort_handle: Arc<tokio::task::AbortHandle>,
}
impl<T:Send + Sync + 'static> AsyncLoader<T> {
    pub fn new<F: std::future::IntoFuture<Output = T> + Send + 'static>(f: F) -> Self where <F as std::future::IntoFuture>::IntoFuture: Send {
        let value = Arc::new(Mutex::new(None));
        let written = Arc::new(AtomicBool::new(false));
        
        let val = value.clone();
        let wrote = written.clone();
        let task = tokio::spawn(async move {
            let v = f.into_future().await;
            *val.lock() = Some(v);
            wrote.store(true, Ordering::Release);
        });

        let abort_handle = Arc::new(task.abort_handle());

        Self {
            value,
            written,
            abort_handle,
        }
    }

    pub fn abort(&self) {
        self.abort_handle.abort();
    }

    pub fn is_complete(&self) -> bool {
        self.written.load(Ordering::Acquire)
    }

    pub fn check(&self) -> Option<T> {
        if self.written.load(Ordering::Acquire) {
            std::mem::take(&mut *self.value.lock())
        } else {
            None
        }
    }
}
impl<T> std::fmt::Debug for AsyncLoader<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AsyncLoader")
    }
}
