pub mod http_model;
#[cfg(not(target_arch = "wasm32"))] // reqwest opens its own sockets
pub mod reqwest_fetch;

pub use http_model::*;
#[cfg(not(target_arch = "wasm32"))] // reqwest opens its own sockets
pub use reqwest_fetch::*;
