pub mod strings;
pub use self::strings::*;

#[cfg(target_arch = "wasm32")]
#[macro_use]
pub mod web;
