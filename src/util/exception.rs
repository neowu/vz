use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt;
use std::io;

use tokio::task::JoinError;

pub struct Exception {
    message: String,
    context: Option<String>,
    trace: String,
}

impl Exception {
    pub fn new(message: String) -> Self {
        Exception::create(message, None)
    }

    pub fn from<T>(error: T) -> Self
    where
        T: Error + 'static,
    {
        Exception::create(error.to_string(), None)
    }

    pub fn from_with_context<T>(error: T, context: String) -> Self
    where
        T: Error + 'static,
    {
        Exception::create(error.to_string(), Some(context))
    }

    fn create(message: String, context: Option<String>) -> Self {
        Self {
            message,
            context,
            trace: Backtrace::force_capture().to_string(),
        }
    }
}

impl fmt::Debug for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Exception: {}\nContext: {}\nTrace:\n{}",
            self.message,
            self.context.as_ref().unwrap_or(&"".to_string()),
            self.trace
        )
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
        Exception::new(err.to_string())
    }
}

impl From<JoinError> for Exception {
    fn from(err: JoinError) -> Self {
        Exception::new(err.to_string())
    }
}
