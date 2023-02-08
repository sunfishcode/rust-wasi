//! API bindings to the WebAssembly System Interface (WASI)

#![cfg_attr(not(feature = "std"), no_std)]

// Generate the bindings.

// There are a few options for the generated bindings. One option we may
// want to use in the future is `unchecked`; this option disables the
// generated code that checks that interface values are well-formed. At
// this time, enough things are experimental that it's worth keeping the
// checks on, but in the future it may be a desirable optimization.
//
// Another option is that instead of using these proc macros, we could
// generate the bindings ahead-of-time using the wit-bindgen CLI. That's
// a little less convenient for development but saves compile time and
// build complexity for users.

#[cfg(feature = "std")]
wit_bindgen_guest_rust::generate!({
    world: "experiment",
});

#[cfg(not(feature = "std"))]
wit_bindgen_guest_rust::generate!({
    world: "experiment",
    no_std,
});
