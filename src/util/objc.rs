use objc2::rc::Retained;
use objc2_foundation::NSError;

use super::exception::Exception;

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
