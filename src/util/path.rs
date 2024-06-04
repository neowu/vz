use std::path::PathBuf;

pub trait UserPath {
    fn to_absolute_path(&self) -> PathBuf;
}

impl UserPath for PathBuf {
    fn to_absolute_path(&self) -> PathBuf {
        if self.starts_with("~") {
            return PathBuf::from(format!(
                "{}/{}",
                env!("HOME"),
                self.strip_prefix("~").map_or("".to_string(), |path| path.to_string_lossy().to_string())
            ));
        }
        self.clone()
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
