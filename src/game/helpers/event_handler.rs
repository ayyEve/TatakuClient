use crate::prelude::*;

#[derive(Clone)]
pub struct EventHandler<T> {
    /// cached settings
    value: Arc<T>,

    /// has the value been set yet?
    initialized: bool,
}
impl<T:EventHandlerInitial+PartialEq> EventHandler<T> {
    pub fn update(&mut self) -> bool {
        let v = T::get_current();
        if &self.value != &v {
            self.value = v;
            return true;
        }
        false
    }
}
impl<T> EventHandler<T> where T:EventHandlerInitial {
    pub async fn new() -> Self {
        Self {
            value: T::get_initial().await,
            initialized: true
        }
    }
}
impl<T> EventHandler<T> where T:EventHandlerInitial {
    pub async fn init(&mut self) {
        if self.initialized { return }
        self.value = T::get_initial().await;
        self.initialized = true;
    }
}


impl<T> Deref for EventHandler<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> Default for EventHandler<T> where T:Default {
    fn default() -> Self {
        Self { 
            value: Arc::new(T::default()), 
            initialized: false,
        }
    }
}

#[async_trait]
pub trait EventHandlerInitial:Sized {
    async fn get_initial() -> Arc<Self>;
    fn get_current() -> Arc<Self>;
}
