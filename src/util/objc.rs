use std::path::Path;

use objc2::rc::Retained;
use objc2::ClassType;
use objc2_foundation::NSError;
use objc2_foundation::NSString;
use objc2_foundation::NSURL;

use super::exception::Exception;

pub fn file_url(path: &Path) -> Retained<NSURL> {
    unsafe { NSURL::initFileURLWithPath(NSURL::alloc(), &NSString::from_str(&path.to_string_lossy())) }
}

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