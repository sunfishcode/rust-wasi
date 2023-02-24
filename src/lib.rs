//! API bindings to the WebAssembly System Interface (WASI)

#![cfg_attr(not(feature = "std"), no_std)]

mod cli;

pub use cli::*;
