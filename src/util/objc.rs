use std::error::Error;
use std::fmt;

use objc2::rc::Retained;
use objc2_foundation::NSError;

#[derive(Debug)]
pub struct ObjcError(String);

impl fmt::Display for ObjcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ObjcError {}

impl From<Retained<NSError>> for ObjcError {
    fn from(err: Retained<NSError>) -> Self {
        ObjcError(err.localizedDescription().to_string())
    }
}

impl From<*mut NSError> for ObjcError {
    fn from(err: *mut NSError) -> Self {
        ObjcError(unsafe { (*err).localizedDescription().to_string() })
    }
}

impl From<Option<Retained<objc2::exception::Exception>>> for ObjcError {
    fn from(err: Option<Retained<objc2::exception::Exception>>) -> Self {
        let message = match err {
            Some(err) => err.to_string(),
            // in objc, throw nil
            None => "nil".to_string(),
        };
        ObjcError(message)
    }
}
