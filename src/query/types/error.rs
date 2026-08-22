use std::fmt;

use crate::server::data::ServerStatus;

/// Status codes shared by every non-A2S protocol implementation.
///
/// These start at 100 so they never collide with the A2S specific codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum QueryStatusCode {
    Io = 100,
    InvalidResponse = 101,
    BadAddress = 102,
    Challenge = 103,
    Unsupported = 104,
}

#[derive(Debug, Clone)]
pub enum QueryError {
    Timeout,
    /// The host answered, but nothing is listening on that port (or it is unreachable).
    Unreachable(String),
    Io(String),
    InvalidResponse(String),
    BadAddress(String),
    Challenge(String),
    Unsupported(&'static str),
}

impl QueryError {
    /// Maps the error onto the status we report for a server.
    ///
    /// A timeout means the server simply didn't answer, which we treat as offline instead of an error.
    pub fn status(&self) -> ServerStatus {
        match self {
            QueryError::Timeout | QueryError::Unreachable(_) => ServerStatus::Offline,
            QueryError::Io(_) => ServerStatus::Error(QueryStatusCode::Io as u16),
            QueryError::InvalidResponse(_) => {
                ServerStatus::Error(QueryStatusCode::InvalidResponse as u16)
            }
            QueryError::BadAddress(_) => ServerStatus::Error(QueryStatusCode::BadAddress as u16),
            QueryError::Challenge(_) => ServerStatus::Error(QueryStatusCode::Challenge as u16),
            QueryError::Unsupported(_) => ServerStatus::Error(QueryStatusCode::Unsupported as u16),
        }
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::Timeout => write!(f, "timed out"),
            QueryError::Unreachable(e) => write!(f, "unreachable: {}", e),
            QueryError::Io(e) => write!(f, "I/O error: {}", e),
            QueryError::InvalidResponse(e) => write!(f, "invalid response: {}", e),
            QueryError::BadAddress(e) => write!(f, "invalid address: {}", e),
            QueryError::Challenge(e) => write!(f, "challenge failed: {}", e),
            QueryError::Unsupported(e) => write!(f, "unsupported: {}", e),
        }
    }
}

impl std::error::Error for QueryError {}

impl From<std::io::Error> for QueryError {
    fn from(e: std::io::Error) -> Self {
        use std::io::ErrorKind;

        // A refused or unreachable port means the server isn't running, not that we failed.
        match e.kind() {
            ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::HostUnreachable
            | ErrorKind::NetworkUnreachable
            | ErrorKind::UnexpectedEof => QueryError::Unreachable(e.to_string()),
            ErrorKind::TimedOut => QueryError::Timeout,
            _ => QueryError::Io(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_unreachable_to_offline() {
        let err: QueryError =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused").into();

        assert_eq!(err.status(), ServerStatus::Offline);
    }

    #[test]
    fn maps_other_io_errors_to_an_error_code() {
        let err: QueryError =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied").into();

        assert_eq!(
            err.status(),
            ServerStatus::Error(QueryStatusCode::Io as u16)
        );
    }
}
