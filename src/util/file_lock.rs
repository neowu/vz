use std::ffi::CString;
use std::os::fd::RawFd;
use std::path::Path;

use libc::pid_t;

pub struct FileLock {
    fd: RawFd,
}

impl FileLock {
    // fd returned by File.as_raw_fd() doesn't work with fcntl
    pub fn new(path: &Path) -> Self {
        let path = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let fd = unsafe { libc::open(path.as_ptr(), libc::O_WRONLY) };
        Self { fd }
    }

    // refer to "man fcntl", once process obtain the lock, it must not reopen fd and close if,
    // close fd will release all locks of current process !!! e.g. lock one file, then read the file / close file
    // https://apenwarr.ca/log/20101213
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

    // return pid of write lock owner
    pub fn pid(&self) -> Option<pid_t> {
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
        // close fd will release all locks
        unsafe { libc::close(self.fd) };
    }
}
