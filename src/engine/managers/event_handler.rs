use std::any::TypeId;
use crate::prelude::*;

lazy_static::lazy_static! {
    static ref THINGY: ShardedLock<HashMap<TypeId, Entry>> = Default::default();
}

struct Entry {
    value: parking_lot::RwLock<Arc<dyn std::any::Any + Send + Sync>>,
    counter: AtomicUsize
}

pub struct GlobalObjectManager;
impl GlobalObjectManager {
    pub fn update<T:'static + Send + Sync>(new_value: Arc<T>) {
        let lock = THINGY.read().unwrap();

        let id = TypeId::of::<T>();
        if let Some(entry) = lock.get(&id) {
            entry.counter.fetch_add(1, SeqCst);
            *entry.value.write() = new_value;
        } else {
            drop(lock);
            let mut lock = THINGY.write().unwrap();
            
            lock.insert(id, Entry {
                value: parking_lot::RwLock::new(new_value),
                counter: AtomicUsize::new(0)
            });
        }
    }

    pub fn get<T:'static + Send + Sync>() -> Option<Arc<T>> {
        let id = TypeId::of::<T>();
        THINGY
            .read()
            .unwrap()
            .get(&id)
            .and_then(|i|i.value.read().clone().downcast::<T>().ok())
    }

    pub fn get_mut<T:'static + Send + Sync + Clone>() -> Option<GlobalObjectMutValue<T>> {
        Self::get().map(|v|GlobalObjectMutValue::new(v))
    }
    
    pub fn check<T:'static + Send + Sync>(last: &mut usize) -> Option<Arc<T>> {
        let id = TypeId::of::<T>();

        let entry = THINGY
            .read()
            .unwrap();
        let entry = entry.get(&id)?;

        let current = entry.counter.load(SeqCst);
        if current > *last {
            *last = current;
            entry.value.read().clone().downcast::<T>().ok()
        } else {
            None
        }
    }
}

pub struct GlobalObjectValue<T: Send + Sync> {
    counter: usize,
    value: Arc<T>
}
impl<T:'static + Send + Sync> GlobalObjectValue<T> {
    pub fn new() -> Self {
        let id = TypeId::of::<T>();
        let entry = THINGY
            .read()
            .unwrap();

        let Some(entry) = entry.get(&id) else { 
            let backtrace = std::backtrace::Backtrace::capture();
            let name = std::any::type_name::<T>();
            panic!("Value not initialized for {name}: {backtrace}")
        };

        let counter = entry.counter.load(SeqCst);
        let value = entry.value.read().clone().downcast::<T>().unwrap();

        Self {
            counter,
            value
        }
    }

    pub fn update(&mut self) -> bool {
        if let Some(new_val) = GlobalObjectManager::check::<T>(&mut self.counter) {
            self.value = new_val;
            true
        } else {
            false
        }
    }
}
impl<T:Send + Sync> Deref for GlobalObjectValue<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}


pub struct GlobalObjectMutValue<T:'static + Send + Sync + Clone>(T);
impl<T:'static + Send + Sync + Clone> GlobalObjectMutValue<T> {
    fn new(val: Arc<T>) -> Self { Self(val.as_ref().clone()) }
}
impl<T:'static + Send + Sync + Clone> Deref for GlobalObjectMutValue<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<T:'static + Send + Sync + Clone> DerefMut for GlobalObjectMutValue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
impl<T:'static + Send + Sync + Clone> Drop for GlobalObjectMutValue<T> {
    fn drop(&mut self) { GlobalObjectManager::update(Arc::new(self.0.clone())) }
}


#[macro_export]
macro_rules! create_value_helper {
    ($struct: ident, $type: ty, $helper_name: ident) => {
        #[derive(Default)]
        pub struct $struct(pub $type);
        impl Deref for $struct {
            type Target = $type;
            fn deref(&self) -> &Self::Target { &self.0 }
        }
        
        pub type $helper_name = GlobalObjectValue<$struct>;
    }
}


#[test]
fn test() {
    #[derive(Clone)]
    struct B(i32);
    
    GlobalObjectManager::update(Arc::new(B(21)));
    let mut instance = GlobalObjectValue::<B>::new();
    assert_eq!(instance.0, 21);

    GlobalObjectManager::update(Arc::new(B(55)));
    instance.update();
    assert_eq!(instance.0, 55);

    {
        let mut b = GlobalObjectManager::get_mut::<B>().unwrap();
        b.0.0 = 500;
    }

    instance.update();
    assert_eq!(instance.0, 500);
}
