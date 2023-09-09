//! Rust-Bitcoin IO Library
//!
//! Because the core `std::io` module is not yet expose in `no-std` Rust, building `no-std`
//! applications which require reading and writing objects via standard traits is not generally
//! possible. While there is ongoing work to improve this situation, this module is not likely to
//! be available for applications with broad rustc version support for some time.
//!
//! Thus, this library exists to export a minmal version of `std::io`'s traits which `no-std`
//! applications may need. With the `std` feature, these traits are also implemented for the
//! `std::io` traits, allowing standard objects to be used wherever the traits from this crate are
//! required.

#![cfg_attr(not(feature = "std"), no_std)]

/// Utility for general-purpose errors, equivalent to `std`'s [`std::error`] module.
pub mod error {
    use core::fmt::{Debug, Display};

    /// A generic `Error` type which is equivalent to `std`'s [`std::error::Error`]. See std's
    /// documentation for more info.
    pub trait Error: Debug + Display {
        fn source(&self) -> Option<&(dyn Error + 'static)> { None }
        fn description(&self) -> &str { "" }
        fn cause(&self) -> Option<&dyn Error> { None }
    }

    #[cfg(feature = "std")]
    impl<E: std::error::Error> Error for E {
        // While it should be possible to munge a dyn std Error into a dyn Error, rustc does
        // not currently propagate lifetimes through such a conversion and refuses to allow it.
        // Thus, we are stuck with `source` and `cause` returning `None` by falling back to the
        // provided implementation, rather than being able to convert.

        fn description(&self) -> &str {
            #[allow(deprecated)]
            <E as std::error::Error>::description(self)
        }
    }

    #[cfg(all(not(feature = "std"), feature = "core2"))]
    impl<E: core2::error::Error> Error for E {
        // While it should be possible to munge a dyn core2 Error into a dyn Error, rustc does
        // not currently propagate lifetimes through such a conversion and refuses to allow it.
        // Thus, we are stuck with `source` and `cause` returning `None` by falling back to the
        // provided implementation, rather than being able to convert.

        // core2's `Error` doesn't contain a `description` method, leaning on the default.
        // Thus, we simply leave the default in place.
    }
}

/// Standard I/O stream definitions which are API-equivalent to `std`'s `io` module. See
/// [`std::io`] for more info.
pub mod io {
    #[cfg(all(not(feature = "std"), not(feature = "core2")))]
    compile_error!("At least one of std or core2 must be enabled");

    #[cfg(feature = "std")]
    pub use std::io::{Read, Write, Cursor, Error, ErrorKind, Result};

    #[cfg(not(feature = "std"))]
    pub use core2::io::{Read, Write, Cursor, Error, ErrorKind, Result};


}
