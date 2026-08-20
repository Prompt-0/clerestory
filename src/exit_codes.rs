//! Standard POSIX / BSD sysexits compatible exit codes for Clerestory.

/// Successful termination
pub const EX_OK: i32 = 0;

/// Command line usage error
pub const EX_USAGE: i32 = 2;

/// Data format error (e.g. malformed XML, unparseable answer file, invalid ISO header)
pub const EX_DATAERR: i32 = 64;

/// Cannot open input file or path does not exist
pub const EX_NOINPUT: i32 = 66;

/// Hardware service or prerequisite unavailable (e.g. missing /dev/kvm, swtpm, or OVMF)
pub const EX_UNAVAILABLE: i32 = 69;

/// Internal software error / domain synthesis failure
pub const EX_SOFTWARE: i32 = 70;

/// Operating system / I/O disk error
pub const EX_IOERR: i32 = 74;
