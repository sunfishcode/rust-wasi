//! API bindings to the WebAssembly System Interface (WASI)

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
pub mod macros;

#[cfg(feature = "command")]
#[cfg_attr(not(feature = "normal"), path = "polyfill/cli.rs")]
mod cli;

#[cfg(feature = "command")]
pub use cli::*;

#[cfg(not(feature = "command"))]
#[cfg_attr(not(feature = "normal"), path = "polyfill/cli_reactor.rs")]
mod cli_reactor;

#[cfg(not(feature = "command"))]
pub use cli_reactor::*;

pub mod fd;
pub mod trapping_unwrap;

#[cfg(feature = "preview1")]
mod preview1;
