use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::sync::mpsc::RecvError;

use objc2::rc::Retained;
use objc2_foundation::NSError;

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

impl From<RecvError> for Exception {
    fn from(err: RecvError) -> Self {
        Exception::unexpected(err)
    }
}

impl From<Retained<NSError>> for Exception {
    fn from(err: Retained<NSError>) -> Self {
        Exception::ObjcError(err.localizedDescription().to_string())
    }
}

impl From<Option<Retained<objc2::exception::Exception>>> for Exception {
    fn from(err: Option<Retained<objc2::exception::Exception>>) -> Self {
        let message = match err {
            Some(err) => err.to_string(),
            // in objc, throw nil
            None => "nil".to_string(),
        };
        Exception::ObjcError(message)
    }
}

impl From<ParseIntError> for Exception {
    fn from(err: ParseIntError) -> Self {
        Exception::ValidationError(err.to_string())
    }
}
