use core::hint::black_box;
use preview1::*;

impl From<crate::fd::error::NotReadable> for Errno {
    fn from(_err: crate::fd::error::NotReadable) -> Self {
        ERRNO_BADF
    }
}

impl From<crate::fd::error::NotWriteable> for Errno {
    fn from(_err: crate::fd::error::NotWriteable) -> Self {
        ERRNO_BADF
    }
}

impl From<crate::fd::error::BadFd> for Errno {
    fn from(_err: crate::fd::error::BadFd) -> Self {
        ERRNO_BADF
    }
}

impl From<crate::fd::error::OutOfMemory> for Errno {
    fn from(_err: crate::fd::error::OutOfMemory) -> Self {
        ERRNO_NOMEM
    }
}

impl From<crate::network::Error> for Errno {
    fn from(error: crate::network::Error) -> Errno {
        match error {
            crate::network::Error::Unknown => unreachable!(), // TODO
            crate::network::Error::Again => ERRNO_AGAIN,
            /* TODO
            // Use a black box to prevent the optimizer from generating a
            // lookup table, which would require a static initializer.
            ConnectionAborted => black_box(ERRNO_CONNABORTED),
            ConnectionRefused => ERRNO_CONNREFUSED,
            ConnectionReset => ERRNO_CONNRESET,
            HostUnreachable => ERRNO_HOSTUNREACH,
            NetworkDown => ERRNO_NETDOWN,
            NetworkUnreachable => ERRNO_NETUNREACH,
            Timedout => ERRNO_TIMEDOUT,
            _ => unreachable!(),
            */
        }
    }
}

impl From<crate::filesystem::ErrorCode> for Errno {
    #[inline(never)] // Disable inlining as this is bulky and relatively cold.
    fn from(err: crate::filesystem::ErrorCode) -> Errno {
        match err {
            // Use a black box to prevent the optimizer from generating a
            // lookup table, which would require a static initializer.
            crate::filesystem::ErrorCode::Access => black_box(ERRNO_ACCES),
            crate::filesystem::ErrorCode::WouldBlock => ERRNO_AGAIN,
            crate::filesystem::ErrorCode::Already => ERRNO_ALREADY,
            crate::filesystem::ErrorCode::BadDescriptor => ERRNO_BADF,
            crate::filesystem::ErrorCode::Busy => ERRNO_BUSY,
            crate::filesystem::ErrorCode::Deadlock => ERRNO_DEADLK,
            crate::filesystem::ErrorCode::Quota => ERRNO_DQUOT,
            crate::filesystem::ErrorCode::Exist => ERRNO_EXIST,
            crate::filesystem::ErrorCode::FileTooLarge => ERRNO_FBIG,
            crate::filesystem::ErrorCode::IllegalByteSequence => ERRNO_ILSEQ,
            crate::filesystem::ErrorCode::InProgress => ERRNO_INPROGRESS,
            crate::filesystem::ErrorCode::Interrupted => ERRNO_INTR,
            crate::filesystem::ErrorCode::Invalid => ERRNO_INVAL,
            crate::filesystem::ErrorCode::Io => ERRNO_IO,
            crate::filesystem::ErrorCode::IsDirectory => ERRNO_ISDIR,
            crate::filesystem::ErrorCode::Loop => ERRNO_LOOP,
            crate::filesystem::ErrorCode::TooManyLinks => ERRNO_MLINK,
            crate::filesystem::ErrorCode::MessageSize => ERRNO_MSGSIZE,
            crate::filesystem::ErrorCode::NameTooLong => ERRNO_NAMETOOLONG,
            crate::filesystem::ErrorCode::NoDevice => ERRNO_NODEV,
            crate::filesystem::ErrorCode::NoEntry => ERRNO_NOENT,
            crate::filesystem::ErrorCode::NoLock => ERRNO_NOLCK,
            crate::filesystem::ErrorCode::InsufficientMemory => ERRNO_NOMEM,
            crate::filesystem::ErrorCode::InsufficientSpace => ERRNO_NOSPC,
            crate::filesystem::ErrorCode::NotDirectory => ERRNO_NOTDIR,
            crate::filesystem::ErrorCode::NotEmpty => ERRNO_NOTEMPTY,
            crate::filesystem::ErrorCode::NotRecoverable => ERRNO_NOTRECOVERABLE,
            crate::filesystem::ErrorCode::Unsupported => ERRNO_NOTSUP,
            crate::filesystem::ErrorCode::NoTty => ERRNO_NOTTY,
            crate::filesystem::ErrorCode::NoSuchDevice => ERRNO_NXIO,
            crate::filesystem::ErrorCode::Overflow => ERRNO_OVERFLOW,
            crate::filesystem::ErrorCode::NotPermitted => ERRNO_PERM,
            crate::filesystem::ErrorCode::Pipe => ERRNO_PIPE,
            crate::filesystem::ErrorCode::ReadOnly => ERRNO_ROFS,
            crate::filesystem::ErrorCode::InvalidSeek => ERRNO_SPIPE,
            crate::filesystem::ErrorCode::TextFileBusy => ERRNO_TXTBSY,
            crate::filesystem::ErrorCode::CrossDevice => ERRNO_XDEV,
        }
    }
}

impl From<crate::filesystem::DescriptorType> for preview1::Filetype {
    fn from(ty: crate::filesystem::DescriptorType) -> preview1::Filetype {
        match ty {
            crate::filesystem::DescriptorType::RegularFile => FILETYPE_REGULAR_FILE,
            crate::filesystem::DescriptorType::Directory => FILETYPE_DIRECTORY,
            crate::filesystem::DescriptorType::BlockDevice => FILETYPE_BLOCK_DEVICE,
            crate::filesystem::DescriptorType::CharacterDevice => FILETYPE_CHARACTER_DEVICE,
            // preview1 never had a FIFO code.
            crate::filesystem::DescriptorType::Fifo => FILETYPE_UNKNOWN,
            // TODO: Add a way to disginguish between FILETYPE_SOCKET_STREAM and
            // FILETYPE_SOCKET_DGRAM.
            crate::filesystem::DescriptorType::Socket => unreachable!(),
            crate::filesystem::DescriptorType::SymbolicLink => FILETYPE_SYMBOLIC_LINK,
            crate::filesystem::DescriptorType::Unknown => FILETYPE_UNKNOWN,
        }
    }
}
