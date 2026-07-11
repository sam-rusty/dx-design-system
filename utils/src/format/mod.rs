mod date;
mod number;
mod percent;
mod phone;
mod text;

pub use date::*;
pub use number::*;
pub use percent::*;
pub use phone::*;
pub use text::*;

/// Convenience macro for `merge()` — joins non-empty class strings with spaces.
///
/// ```ignore
/// use utils::classes;
/// let s = classes!("px-4 py-2", "text-sm", &user_class);
/// ```
#[macro_export]
macro_rules! classes {
    ($($class:expr),* $(,)?) => {
        $crate::format::merge(&[$($class),*])
    };
}
