mod multiplayer_data;
mod score_submit_helper;

pub use multiplayer_data::*;
pub use score_submit_helper::*;

// FIXME: move this to common
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
