//! Minimal versions of standard-library panicking and printing macros.
//!
//! We're avoiding static initializers, so we can't have things like string
//! literals. Replace the standard assert macros with simpler implementations.

#[doc(hidden)]
pub use byte_array;

#[allow(dead_code)]
#[doc(hidden)]
pub fn print(message: &[u8]) {
    crate::stderr::print(message)
}

/// A minimal `eprint` for debugging.
#[macro_export]
macro_rules! eprint {
    ($arg:tt) => {{
        // We have to expand string literals into byte arrays to prevent them
        // from getting statically initialized.
        let message = $crate::macros::byte_array::str!($arg);
        $crate::macros::print(&message);
    }};
}

/// A minimal `eprintln` for debugging.
#[macro_export]
macro_rules! eprintln {
    ($arg:tt) => {{
        // We have to expand string literals into byte arrays to prevent them
        // from getting statically initialized.
        let message = $crate::macros::byte_array::str_nl!($arg);
        $crate::macros::print(&message);
    }};
}

#[allow(dead_code)]
#[doc(hidden)]
pub fn eprint_u32(x: u32) {
    if x == 0 {
        eprint!("0");
    } else {
        eprint_u32_impl(x)
    }

    fn eprint_u32_impl(x: u32) {
        if x != 0 {
            eprint_u32_impl(x / 10);

            let digit = [b'0' + ((x % 10) as u8)];
            print(&digit);
        }
    }
}

/// A minimal `unreachable`.
#[macro_export]
macro_rules! unreachable {
    () => {{
        eprint!("unreachable executed at line ");
        $crate::macros::eprint_u32(line!());
        eprint!("\n");
        core::arch::wasm32::unreachable()
    }};

    ($arg:tt) => {{
        eprint!("unreachable executed at line ");
        $crate::macros::eprint_u32(line!());
        eprint!(": ");
        eprintln!($arg);
        core::arch::wasm32::unreachable()
    }};
}

/// A minimal `assert`.
#[macro_export]
macro_rules! assert {
    ($cond:expr $(,)?) => {
        if !$cond {
            unreachable!("assertion failed")
        }
    };
}

/// A minimal `assert_eq`.
#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr $(,)?) => {
        assert!($left == $right);
    };
}
