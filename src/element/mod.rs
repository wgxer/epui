pub mod r#box;
pub mod text;

pub use r#box::{UiBox, UiBoxBundle};
pub use text::{FontSize, UiText, UiTextBundle};

#[doc(hidden)]
pub mod prelude {
    pub use crate::element::{FontSize, UiBox, UiBoxBundle, UiText, UiTextBundle};
}
