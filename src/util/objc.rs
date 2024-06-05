use objc2::rc::Retained;
use objc2_foundation::NSError;

use super::exception::Exception;

impl From<Retained<NSError>> for Exception {
    fn from(err: Retained<NSError>) -> Self {
        Exception::new(err.localizedDescription().to_string())
    }
}

impl From<Option<Retained<objc2::exception::Exception>>> for Exception {
    fn from(err: Option<Retained<objc2::exception::Exception>>) -> Self {
        match err {
            Some(err) => Exception::new(err.to_string()),
            // in objc, throw nil
            None => Exception::new("nil".to_string()),
        }
    }
}

pub fn error_message(err: *mut NSError) -> String {
    unsafe { (*err).localizedDescription().to_string() }
}
