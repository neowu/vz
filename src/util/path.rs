use std::path::Path;
use std::path::PathBuf;

use objc2::AllocAnyThread;
use objc2::rc::Retained;
use objc2_foundation::NSString;
use objc2_foundation::NSURL;

pub trait PathExtension {
    fn to_absolute_path(&self) -> PathBuf;

    fn to_ns_url(&self) -> Retained<NSURL>;
}

impl PathExtension for Path {
    fn to_absolute_path(&self) -> PathBuf {
        if self.starts_with("~") {
            return PathBuf::from(format!(
                "{}/{}",
                env!("HOME"),
                self.strip_prefix("~").map_or("".to_string(), |path| path.to_string_lossy().to_string())
            ));
        }
        PathBuf::from(self)
    }

    fn to_ns_url(&self) -> Retained<NSURL> {
        let path = NSString::from_str(&self.to_string_lossy());
        unsafe { NSURL::initFileURLWithPath(NSURL::alloc(), &path) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_absolute_path() {
        assert_eq!(PathBuf::from("/Users").to_absolute_path(), PathBuf::from("/Users"));
        assert_eq!(PathBuf::from("~").to_absolute_path(), PathBuf::from(env!("HOME")));
        assert_eq!(PathBuf::from("~/").to_absolute_path(), PathBuf::from(format!("{}/", env!("HOME"))));
        assert_eq!(
            PathBuf::from("~/Desktop").to_absolute_path(),
            PathBuf::from(format!("{}/Desktop", env!("HOME")))
        );
    }
}
