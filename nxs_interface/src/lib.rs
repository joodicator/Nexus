//! Abstract definitions of the interfaces of modules.

#[cfg(feature = "util")]
pub mod util;

#[cfg(feature = "root")]
pub mod root;

#[cfg(feature = "text")]
pub mod text;

pub type Error = &'static str;
pub type Result<T> = std::result::Result<T, Error>;
