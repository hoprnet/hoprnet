#[cfg(unix)]
pub(crate) mod unix;

#[cfg(unix)]
pub(crate) use self::unix::*;

#[cfg(not(unix))]
pub(crate) mod generic;

#[cfg(not(unix))]
pub(crate) use self::generic::*;

// TODO On Windows, use CreateFileW with FILE_ATTRIBUTE_HIDDEN, FILE_FLAG_DELETE_ON_CLOSE +
// MoveFileEx with MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH
