//! API bindings to the WebAssembly System Interface (WASI)

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
pub mod macros;

#[cfg(feature = "command")]
#[cfg_attr(not(feature = "normal"), path = "polyfill/command.rs")]
mod command;

#[cfg(feature = "command")]
pub use command::*;

#[cfg(not(feature = "command"))]
#[cfg_attr(not(feature = "normal"), path = "polyfill/reactor.rs")]
mod reactor;

#[cfg(not(feature = "command"))]
pub use reactor::*;

pub mod fd;
pub mod trapping_unwrap;

#[cfg(feature = "preview1")]
mod preview1;
