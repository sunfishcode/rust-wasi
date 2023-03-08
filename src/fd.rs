//! An implementation of Unix-like dynamically-typed "file descriptors".
//!
//! WASI handles are statically typed. In order to support Rust code built for
//! Unix-style dynamically-typed file descriptors, this file contains utilities
//! for managing a table of file descriptors where indices into the table are
//! presented to the user as file descriptors.

use crate::filesystem::{
    append_via_stream, drop_descriptor, read_via_stream, write_via_stream, Descriptor, Filesize,
};
use crate::streams::{drop_input_stream, drop_output_stream, InputStream, OutputStream};
use crate::tcp::TcpSocket;
use crate::trapping_unwrap::TrappingUnwrap;
use core::cell::Cell;
use core::convert::TryFrom;
use core::num::TryFromIntError;

/// A file descriptor index.
pub type RawFd = u32;

/// A file descriptor table entry.
#[repr(C)]
pub enum FdEntry {
    /// A closed descriptor, holding a reference to the previous closed
    /// descriptor to support reusing them.
    Closed(Option<RawFd>),

    /// Input and/or output wasi-streams, along with stream metadata.
    Streams(Streams),

    /// Writes to `fd_write` will go to the `wasi-stderr` API.
    Stderr,
}

/// Input and/or output wasi-streams, along with a stream type that
/// identifies what kind of stream they are and possibly supporting
/// type-specific operations like seeking.
pub struct Streams {
    /// The output stream, if present.
    pub input: Cell<Option<InputStream>>,

    /// The input stream, if present.
    pub output: Cell<Option<OutputStream>>,

    /// Information about the source of the stream.
    pub type_: StreamType,
}

impl Streams {
    /// Return the input stream, initializing it on the fly if needed.
    pub fn get_read_stream(&self) -> Result<InputStream, NotReadable> {
        match &self.input.get() {
            Some(wasi_stream) => Ok(*wasi_stream),
            None => match &self.type_ {
                // For files, we may have adjusted the position for seeking, so
                // create a new stream.
                StreamType::File(file) => {
                    let input = read_via_stream(file.fd, file.position.get());
                    self.input.set(Some(input));
                    Ok(input)
                }
                _ => Err(NotReadable),
            },
        }
    }

    /// Return the output stream, initializing it on the fly if needed.
    pub fn get_write_stream(&self) -> Result<OutputStream, NotWriteable> {
        match &self.output.get() {
            Some(wasi_stream) => Ok(*wasi_stream),
            None => match &self.type_ {
                // For files, we may have adjusted the position for seeking, so
                // create a new stream.
                StreamType::File(file) => {
                    let output = if file.append {
                        append_via_stream(file.fd)
                    } else {
                        write_via_stream(file.fd, file.position.get())
                    };
                    self.output.set(Some(output));
                    Ok(output)
                }
                _ => Err(NotWriteable),
            },
        }
    }
}

/// The type of a stream, including information about the source
/// of the stream.
#[allow(dead_code)] // until Socket is implemented
pub enum StreamType {
    /// It's a valid stream but we don't know where it comes from.
    Unknown,

    /// A stdin source containing no bytes.
    EmptyStdin,

    /// Streaming data with a file.
    File(File),

    /// Streaming data with a socket connection.
    Socket(TcpSocket),
}

impl Drop for FdEntry {
    fn drop(&mut self) {
        match self {
            FdEntry::Streams(stream) => {
                if let Some(input) = stream.input.get() {
                    drop_input_stream(input);
                }
                if let Some(output) = stream.output.get() {
                    drop_output_stream(output);
                }
                match &stream.type_ {
                    StreamType::File(file) => drop_descriptor(file.fd),
                    StreamType::Socket(_) => unreachable!(), // TODO
                    StreamType::EmptyStdin | StreamType::Unknown => {}
                }
            }
            FdEntry::Stderr => {}
            FdEntry::Closed(_) => {}
        }
    }
}

/// An open file.
#[repr(C)]
pub struct File {
    /// The handle to the preview2 descriptor that this file is referencing.
    pub fd: Descriptor,

    /// The current-position pointer.
    pub position: Cell<Filesize>,

    /// In append mode, all writes append to the file.
    pub append: bool,
}

/// A file descriptor table implementation trait.
///
/// This trait is designed to let users control how the file descriptor table
/// is allocated, accessed, and extended, while providing utility routines on
/// top for managing files, sockets, and streams.
///
/// Users implement `push`, `descriptors`, `descriptors_mut`, `closed`,
/// and `set_closed`, and this trait provides implementations for the rest.
pub trait FdTableAccessors {
    /// Push an entry to the end of the entries table. This doesn't reuse
    /// closed entries; most users should use `insert` instead.
    ///
    /// This uses internal mutability.
    fn push(&self, entry: FdEntry) -> Result<RawFd, OutOfMemory>;

    /// Return a slice containing all the file descriptor entries.
    fn descriptors(&self) -> &[FdEntry];

    /// Return a mutable slice containing all the file descriptor entries.
    fn descriptors_mut(&mut self) -> &mut [FdEntry];

    /// Returns the head of a list of closed file descriptors that may be
    /// reused.
    fn closed(&self) -> Option<RawFd>;

    /// Returns the head of a list of closed file descriptors that may be
    /// reused.
    fn set_closed(&mut self, fd: Option<RawFd>);

    /// Close the given file descriptor.
    fn close(&mut self, fd: RawFd) -> Result<(), BadFd> {
        let closed = self.closed();
        let entry = self.get_mut(fd)?;
        *entry = FdEntry::Closed(closed);
        self.set_closed(Some(fd));
        Ok(())
    }

    /// Add `entry` to the file descriptor table, and on success return the
    /// index it was added at.
    fn insert(&mut self, entry: FdEntry) -> Result<RawFd, OutOfMemory> {
        match self.closed() {
            // No free fds; create a new one.
            None => self.push(entry),
            // `recycle_fd` is a free fd.
            Some(recycle_fd) => {
                let recycle_entry = self.get_mut(recycle_fd).trapping_unwrap();
                let next_closed = match recycle_entry {
                    FdEntry::Closed(next) => *next,
                    _ => unreachable!(),
                };
                *recycle_entry = entry;
                self.set_closed(next_closed);
                Ok(recycle_fd)
            }
        }
    }

    /// Initialize the table. This inserts default-value entries for stdin,
    /// stdout, and stderr.
    fn init(&mut self) -> Result<(), OutOfMemory> {
        // Set up a default stdin. This will be overridden when `command`
        // is called.
        self.insert(FdEntry::Streams(Streams {
            input: Cell::new(None),
            output: Cell::new(None),
            type_: StreamType::Unknown,
        }))?;

        // Set up a default stdout, writing to the stderr device. This will
        // be overridden when `command` is called.
        self.insert(FdEntry::Stderr)?;

        // Set up a default stderr.
        self.insert(FdEntry::Stderr)?;

        Ok(())
    }

    /// Get the `FdEntry` associated with a `RawFd`.
    fn get(&self, fd: RawFd) -> Result<&FdEntry, BadFd> {
        self.descriptors().get(usize::try_from(fd)?).ok_or(BadFd)
    }

    /// Get the mutable `FdEntry` associated with a `RawFd`.
    fn get_mut(&mut self, fd: RawFd) -> Result<&mut FdEntry, BadFd> {
        self.descriptors_mut()
            .get_mut(usize::try_from(fd)?)
            .ok_or(BadFd)
    }

    /// A wrapper around `get` that returns the `Streams` associated with a
    /// `RawFd`, if there is one.
    fn get_streams(&self, fd: RawFd) -> Result<&Streams, BadStreamFd> {
        match self.get(fd)? {
            FdEntry::Streams(streams) => Ok(streams),
            FdEntry::Closed(_) => Err(BadStreamFd::BadFd),
            _ => Err(BadStreamFd::NotStream),
        }
    }

    /// A wrapper around `get` that returns the `File` associated with a
    /// `RawFd`, if there is one.
    fn get_file(&self, fd: RawFd) -> Result<&File, BadFileFd> {
        match self.get(fd)? {
            FdEntry::Streams(Streams {
                type_: StreamType::File(file),
                ..
            }) => Ok(file),
            FdEntry::Closed(_) => Err(BadFileFd::BadFd),
            _ => Err(BadFileFd::NotFile),
        }
    }

    /// A wrapper around `get` that returns the `TcpSocket` associated with a
    /// `RawFd`, if there is one.
    #[allow(dead_code)] // until Socket is implemented
    fn get_socket(&self, fd: RawFd) -> Result<TcpSocket, BadSocketFd> {
        match self.get(fd)? {
            FdEntry::Streams(Streams {
                type_: StreamType::Socket(socket),
                ..
            }) => Ok(*socket),
            FdEntry::Closed(_) => Err(BadSocketFd::BadFd),
            _ => Err(BadSocketFd::NotSocket),
        }
    }

    /// A wrapper around `get` that returns the `InputStream` associated with a
    /// `RawFd`, if possible, creating one if needed.
    fn get_read_stream(&self, fd: RawFd) -> Result<InputStream, BadStreamFd> {
        match self.get(fd)? {
            FdEntry::Streams(streams) => Ok(streams.get_read_stream()?),
            FdEntry::Closed(_) | FdEntry::Stderr => Err(BadStreamFd::NotStream),
        }
    }

    /// A wrapper around `get` that returns the `OutputStream` associated with
    /// a `RawFd`, if possible, creating one if needed.
    fn get_write_stream(&self, fd: RawFd) -> Result<OutputStream, BadStreamFd> {
        match self.get(fd)? {
            FdEntry::Streams(streams) => Ok(streams.get_write_stream()?),
            FdEntry::Closed(_) | FdEntry::Stderr => Err(BadStreamFd::NotStream),
        }
    }
}

/// Error types for file descriptor table accessors.
pub mod error {
    /// Error type for an out-of-bounds or closed file descriptor.
    pub struct BadFd;

    /// Error type for running out of memory for the file descriptor table.
    pub struct OutOfMemory;

    /// Error type for attempting to read on an unreadable file descriptor.
    pub struct NotReadable;

    /// Error type for attempting to write on an unwriteable file descriptor.
    pub struct NotWriteable;

    /// Error type for attempting a stream operation on a non-stream file
    /// descriptor.
    pub enum BadStreamFd {
        BadFd,
        NotStream,
    }

    /// Error type for attempting a file operation on a non-file file descriptor.
    pub enum BadFileFd {
        BadFd,
        NotFile,
    }

    /// Error type for attempting a socket operation on a non-socket file
    /// descriptor.
    pub enum BadSocketFd {
        BadFd,
        NotSocket,
    }
}
use error::*;

impl From<TryFromIntError> for BadFd {
    fn from(_err: TryFromIntError) -> Self {
        Self
    }
}

impl From<BadFd> for BadStreamFd {
    fn from(_bad_fd: BadFd) -> Self {
        Self::BadFd
    }
}

impl From<NotReadable> for BadStreamFd {
    fn from(_not_readable: NotReadable) -> Self {
        Self::BadFd
    }
}

impl From<NotWriteable> for BadStreamFd {
    fn from(_not_writeable: NotWriteable) -> Self {
        Self::BadFd
    }
}

impl From<BadFd> for BadFileFd {
    fn from(_bad_fd: BadFd) -> Self {
        Self::BadFd
    }
}

impl From<BadFd> for BadSocketFd {
    fn from(_bad_fd: BadFd) -> Self {
        Self::BadFd
    }
}
