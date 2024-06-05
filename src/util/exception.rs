use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt;
use std::io;
use std::sync::mpsc::RecvError;

use objc2_foundation::NSError;
use tokio::task::JoinError;

pub enum Exception {
    ValidationError(String),
    Unexpected { message: String, trace: String },
    ObjcError(String),
}

impl Exception {
    pub fn unexpected<T>(error: T) -> Self
    where
        T: std::error::Error,
    {
        Self::Unexpected {
            message: error.to_string(),
            trace: Backtrace::force_capture().to_string(),
        }
    }

    pub fn unexpected_with_context<T>(error: T, context: &str) -> Self
    where
        T: Error,
    {
        Self::Unexpected {
            message: format!("error={}, context={}", error, context),
            trace: Backtrace::force_capture().to_string(),
        }
    }

    pub fn from_ns_error(err: *mut NSError) -> Self {
        Self::ObjcError(unsafe { (*err).localizedDescription().to_string() })
    }
}

impl fmt::Debug for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Exception::ValidationError(message) => write!(f, "{}", message),
            Exception::Unexpected { message, trace } => write!(f, "{}\ntrace:\n{}", message, trace),
            Exception::ObjcError(message) => write!(f, "{}", message),
        }
    }
}

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Error for Exception {}

impl From<io::Error> for Exception {
    fn from(err: io::Error) -> Self {
        Exception::unexpected(err)
    }
}

impl From<JoinError> for Exception {
    fn from(err: JoinError) -> Self {
        Exception::unexpected(err)
    }
}

impl From<RecvError> for Exception {
    fn from(err: RecvError) -> Self {
        Exception::unexpected(err)
    }
}
