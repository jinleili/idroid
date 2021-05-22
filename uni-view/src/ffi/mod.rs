pub mod strings;
pub use self::strings::*;

#[cfg(target_arch = "wasm32")]
pub mod web;