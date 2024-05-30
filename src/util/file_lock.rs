use std::ffi::CString;
use std::os::fd::RawFd;
use std::path::Path;

pub struct FileLock {
    fd: RawFd,
}

impl FileLock {
    pub fn new(path: &Path) -> Self {
        let path = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let fd = unsafe { libc::open(path.as_ptr(), libc::O_WRONLY) };
        Self { fd }
    }

    pub fn lock(&self) -> bool {
        let lock = libc::flock {
            l_start: 0,
            l_len: 0,
            l_pid: -1,
            l_type: libc::F_WRLCK,
            l_whence: libc::SEEK_SET as libc::c_short,
        };
        let result = unsafe { libc::fcntl(self.fd, libc::F_SETLK, &lock) };
        result == 0
    }

    pub fn pid(&self) -> Option<i32> {
        let mut lock = libc::flock {
            l_start: 0,
            l_len: 0,
            l_pid: -1,
            l_type: libc::F_RDLCK,
            l_whence: libc::SEEK_SET as libc::c_short,
        };
        unsafe { libc::fcntl(self.fd, libc::F_GETLK, &mut lock) };
        if lock.l_type == libc::F_WRLCK {
            Some(lock.l_pid)
        } else {
            None
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}
