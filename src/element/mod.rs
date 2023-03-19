pub mod r#box;

pub use r#box::{UiBox, UiBoxBundle};

#[doc(hidden)]
pub mod prelude {
    pub use crate::element::{UiBox, UiBoxBundle};
}
