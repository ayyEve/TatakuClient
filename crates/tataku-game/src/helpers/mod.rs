use crate::prelude::*;

mod score_submit_helper;
pub use score_submit_helper::*;



#[macro_export]
macro_rules! create_value_helper {
    ($struct: ident, $type: ty, $helper_name: ident) => {
        #[derive(Default)]
        pub struct $struct(pub $type);
        impl Deref for $struct {
            type Target = $type;
            fn deref(&self) -> &Self::Target { &self.0 }
        }
        
        pub type $helper_name = GlobalValue<$struct>;
    }
}

pub trait Remove<T> {
    fn remove_item(&mut self, item:T);
}
impl<T> Remove<T> for Vec<T> where T:Eq {
    fn remove_item(&mut self, remove_item:T) {
        for (index, item) in self.iter().enumerate() {
            if *item == remove_item {
                self.remove(index);
                return;
            }
        }
    }
}


pub trait UnwrapNormal {
    fn normal_or(self, other:Self) -> Self;
}
impl UnwrapNormal for f32 {
    fn normal_or(self, other:Self) -> Self {
        if self.is_normal() {self} else {other}
    }
}
impl UnwrapNormal for f64 {
    fn normal_or(self, other:Self) -> Self {
        if self.is_normal() {self} else {other}
    }
}


pub trait CopyDefault<T> {
    fn copy_or_default(&self) -> T;
}
impl<T:Copy+Default> CopyDefault<T> for Option<&T> {
    fn copy_or_default(&self) -> T {
        self.map(|n|*n).unwrap_or_default()
    }
}

/// helper trait so i dont need to if let Some(v) = v {v.thing()}
pub trait OkDo<T> {
    fn ok_do(&self, f: impl FnOnce(&T));
    fn ok_do_mut(&mut self, f: impl FnOnce(&mut T));
}
impl<T> OkDo<T> for Option<T> {
    fn ok_do(&self, f: impl FnOnce(&T)) {
        if let Some(s) = self { f(s) }
    }
    fn ok_do_mut(&mut self, f: impl FnOnce(&mut T)) {
        if let Some(s) = self { f(s) }
    }
}


#[macro_export]
macro_rules! async_retain {
    ($list:ident, $item:ident, $check_fn:expr) => {{

        let mut to_remove = Vec::new();
        for (n, $item) in $list.iter().enumerate() {
            if !$check_fn {
                to_remove.push(n)
            }
        }

        for i in to_remove.into_iter().rev() {
            $list.remove(i);
        }

    }}
}
