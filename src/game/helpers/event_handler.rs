use crate::prelude::*;

#[derive(Clone)]
pub struct EventHandler<T> {
    /// cached settings
    value: Arc<T>,

    /// what checks for new settings updates
    receiver: MultiBomb<Arc<T>>,
}
impl<T> EventHandler<T> {
    pub fn update(&mut self) -> bool {
        let mut changed = false;
        // while to get the most up-to-date settings
        while let Some(value) = self.receiver.exploded() {
            self.value = value;
            changed |= true;
        }
        changed
    }
}
impl<T> EventHandler<T> where T:Default+EventHandlerInitial+EventHandlerInit {
    pub async fn new() -> Self {
        Self {
            value: T::get_initial().await,
            receiver: T::get_receiver(),
        }
    }
}


impl<T> Deref for EventHandler<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> Default for EventHandler<T> where T:Default+EventHandlerInit {
    fn default() -> Self {
        Self { 
            value: Arc::new(T::default()), 
            receiver: T::get_receiver() 
        }
    }
}



pub trait EventHandlerInit:Sized {
    fn get_receiver() -> MultiBomb<Arc<Self>>;
}

#[async_trait]
pub trait EventHandlerInitial:Sized {
    async fn get_initial() -> Arc<Self>;
}
