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

pub struct GlobalObjectInstance<T: Send + Sync> {
    counter: usize,
    value: Arc<T>
}
impl<T:'static + Send + Sync> GlobalObjectInstance<T> {
    pub fn new() -> Self {
        let id = TypeId::of::<T>();
        let entry = THINGY
            .read()
            .unwrap();

        let Some(entry) = entry.get(&id) else { 
            let backtrace = std::backtrace::Backtrace::capture();
            panic!("Value not initialized for typeid {id:?}: {backtrace}")
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
impl<T:Send + Sync> Deref for GlobalObjectInstance<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}



#[test]
fn test() {
    // struct A;
    struct B(i32);
    
    GlobalObjectManager::update(Arc::new(B(21)));

    let mut instance = GlobalObjectInstance::<B>::new();

    println!("b: {}", instance.0);

    GlobalObjectManager::update(Arc::new(B(55)));

    instance.update();
    
    println!("b2: {}", instance.0);
}
