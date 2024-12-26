pub struct Pool<T> {
    items: Vec<PoolEntry<T>>,
    next_available: Option<usize>,
}
impl<T> Pool<T> {
    pub fn new(size: usize, init: impl Fn(usize) -> T) -> Self {
        if size == 0 { panic!("tried to create a pool of size 0"); }

        let mut items:Vec<PoolEntry<T>> = (0..size).map(|i| PoolEntry::new(init(i), i)).collect();
        items.last_mut().unwrap().set_next(None);

        Self {
            items,
            next_available: Some(0),
        }
    }

    /// Returns the next free item in the list (if available)
    /// 
    /// Note that the returned element has not been cleared since is previous use
    #[allow(clippy::should_implement_trait, reason = "switch to lending iterator")]
    pub fn next(&mut self) -> Option<&mut PoolEntry<T>> {
        if let Some(next) = self.next_available {
            let next = &mut self.items[next];
            next.in_use = true;
            self.next_available = next.next;
            return Some(next);
        }
        
        None
    }

    /// Get the entry at `index`
    pub fn get(&mut self, index: usize) -> Option<&mut PoolEntry<T>> {
        self.items.get_mut(index)
    }

    /// Frees the entry at `index`
    pub fn remove(&mut self, index: usize) {
        let i = &mut self.items[index];
        if !i.in_use { return }

        i.set_next(self.next_available);
        i.in_use = false;
        self.next_available = Some(index);
    }

    /// iterate through used elements
    pub fn iter_used(&self) -> impl Iterator<Item=&PoolEntry<T>> {
        self.items.iter().filter(|i|i.in_use)
    }
    /// iterate through used elements
    pub fn iter_used_mut(&mut self) -> impl Iterator<Item=&mut PoolEntry<T>> {
        self.items.iter_mut().filter(|i|i.in_use)
    }

    /// resets the pool, removing all used entries
    pub fn clear(&mut self) {
        let to_remove = self.iter_used().map(|p| p.get_index()).collect::<Vec<_>>();
        to_remove.into_iter().for_each(|p| self.remove(p));
    }
}
impl<T:Clone> Pool<T> {
    /// convenience function if init type is clone
    pub fn new_cloning(size: usize, init: T) -> Self {
        Self::new(size, |_|init.clone())
    }
}



#[derive(Debug)]
pub struct PoolEntry<T> {
    entry: T,
    in_use: bool,
    index: usize,
    next: Option<usize>,
}
impl<T> PoolEntry<T> {
    fn new(entry: T, index: usize) -> Self {
        Self {
            entry,
            index,
            in_use: false,
            next: Some(index+1)
        }
    }
    fn set_next(&mut self, next: Option<usize>) {
        self.next = next
    }

    pub fn get_index(&self) -> usize {
        self.index
    }
}

impl<T> std::ops::Deref for PoolEntry<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}
impl<T> std::ops::DerefMut for PoolEntry<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entry
    }
}



#[test]
#[allow(unused)]
fn test() {
    let mut pool = Pool::new_cloning(5, 0);

    let p = pool.next().unwrap();
    **p = 10;
    // drop(p);

    let p2 = pool.next().unwrap();
    **p2 = 100;
    let p2_i = p2.get_index();
    // drop(p2);

    println!("{:#?}", pool.items);

    let p3 = pool.next().unwrap();
    **p3 = 200;
    // drop(p3);

    println!("removing index {p2_i}");
    pool.remove(p2_i);

    // should be same index as p2
    let p4 = pool.next().unwrap();
    **p4 = 600;
    assert_eq!(p2_i, p4.get_index());
    // drop(p4);

    let p5 = pool.next().unwrap();
    let p5_i = p5.get_index();
    println!("got index {p5_i}");
    **p5 = 900;
    // drop(p5);

    println!("{:#?}", pool.items);

    for i in pool.iter_used() { }
}

