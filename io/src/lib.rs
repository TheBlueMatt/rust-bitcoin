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
    use core::convert::TryInto;

    #[cfg(all(not(feature = "std"), not(feature = "core2")))]
    compile_error!("At least one of std or core2 must be enabled");

    #[cfg(feature = "std")]
    pub use std::io::{Cursor, Error, ErrorKind, Result};

    #[cfg(not(feature = "std"))]
    pub use core2::io::{Cursor, Error, ErrorKind, Result};

    /// A generic trait describing an input stream. See [`std::io::Read`] for more info.
    pub trait Read {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
        #[inline]
        fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
            while !buf.is_empty() {
                match self.read(buf) {
                    Ok(0) =>
                        return Err(Error::new(ErrorKind::UnexpectedEof, "")),
                    Ok(len) => {
                        buf = &mut buf[len..];
                    },
                    Err(e) if e.kind() == ErrorKind::Interrupted => {},
                    Err(e) => return Err(e),
                }
            }
            Ok(())
        }
        #[inline]
        fn take<'a>(&'a mut self, limit: u64) -> Take<'a, Self> {
            Take { reader: self, remaining: limit }
        }
        #[inline]
        fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
            let mut bytes_read = 0;
            let mut read_buf = [0u8; 512];
            loop {
                match self.read(&mut read_buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&read_buf[0..n]);
                        bytes_read += n;
                    },
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(e) => return Err(e),
                }
            }
            Ok(bytes_read)
        }
    }

    pub struct Take<'a, R: Read + ?Sized> {
        reader: &'a mut R,
        remaining: u64,
    }
    impl<'a, R: Read + ?Sized> Read for Take<'a, R> {
        #[inline]
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            let len = core::cmp::min(buf.len(), self.remaining.try_into().unwrap_or(buf.len()));
            let read = self.reader.read(&mut buf[..len])?;
            self.remaining -= read.try_into().unwrap_or(self.remaining);
            Ok(read)
        }
    }

    #[cfg(feature = "std")]
    impl<R: std::io::Read> Read for R {
        #[inline]
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            <R as std::io::Read>::read(self, buf)
        }
    }

    #[cfg(all(feature = "core2", not(feature = "std")))]
    impl<R: core2::io::Read> Read for R {
        #[inline]
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            <R as core2::io::Read>::read(self, buf)
        }
    }

    /// A generic trait describing an output stream. See [`std::io::Write`] for more info.
    pub trait Write {
        fn write(&mut self, buf: &[u8]) -> Result<usize>;
        fn flush(&mut self) -> Result<()>;

        #[inline]
        fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
            while !buf.is_empty() {
                match self.write(buf) {
                    Ok(0) =>
                        return Err(Error::new(ErrorKind::UnexpectedEof, "")),
                    Ok(len) => {
                        buf = &buf[len..];
                    },
                    Err(e) if e.kind() == ErrorKind::Interrupted => {},
                    Err(e) => return Err(e),
                }
            }
            Ok(())
        }
    }

    #[cfg(feature = "std")]
    impl<W: std::io::Write> Write for W {
        #[inline]
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            <W as std::io::Write>::write(self, buf)
        }
        #[inline]
        fn flush(&mut self) -> Result<()> {
            <W as std::io::Write>::flush(self)
        }
    }

    #[cfg(all(feature = "core2", not(feature = "std")))]
    impl<W: core2::io::Write> Write for W {
        #[inline]
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            <W as core2::io::Write>::write(self, buf)
        }
        #[inline]
        fn flush(&mut self) -> Result<()> {
            <W as core2::io::Write>::flush(self)
        }
    }

    /// A sink to which all writes succeed. See [`std::io::Sink`] for more info.
    pub struct Sink;
    impl Write for Sink {
        #[inline]
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            Ok(buf.len())
        }
        #[inline]
        fn flush(&mut self) -> Result<()> {
            Ok(())
        }
    }
    /// Returns a sink to which all writes succeed. See [`std::io::sink`] for more info.
    pub fn sink() -> Sink { Sink }
}
