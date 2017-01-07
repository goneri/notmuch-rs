extern crate libc;
extern crate tempdir;

use std::ffi::{CStr, CString};
use std::{str, ptr};
use std::path::Path;

mod ffi;
pub use ffi::Status;
pub use ffi::OpenMode;

#[derive(Debug)]
pub struct Database {
    database: ffi::database_t
}

impl Database {
    pub fn create(path: &Path) -> Result<Database, Status> {
        let path_str = path.to_str().unwrap();
        let cstring = CString::new(path_str.as_bytes()).unwrap();
        let mut database: ffi::database_t = ptr::null_mut();
        unsafe {
            match ffi::notmuch_database_create(cstring.as_ptr(), &mut database) {
                Status::Success => Ok(Database { database: database }),
                result => Err(result),
            }
        }
    }

    pub fn open(path: &Path, mode: OpenMode) -> Result<Database, Status> {
        let path_str = path.to_str().unwrap();
        let cstring = CString::new(path_str.as_bytes()).unwrap();
        let mut database: ffi::database_t = ptr::null_mut();
        unsafe {
            match ffi::notmuch_database_open(cstring.as_ptr(), mode, &mut database) {
                Status::Success => Ok(Database { database: database }),
                result => Err(result),
            }
        }
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        unsafe {
            ffi::notmuch_database_destroy(self.database);
            self.database = ptr::null_mut();
        }
    }
}

pub fn explain_status(status: Status) -> String {
    unsafe {
        let ptr = ffi::notmuch_status_to_string(status);
        let bytes = CStr::from_ptr(ptr).to_bytes();
        str::from_utf8(bytes).ok().unwrap().to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempdir::TempDir;

    macro_rules! t {
        ($e:expr) => (match $e { Ok(n) => n, Err(e) => panic!("error: {}", e) })
    }

    #[test]
    fn status_descriptions() {
        assert_eq!(explain_status(Status::Success), "No error occurred");
        assert_eq!(explain_status(Status::OutOfMemory), "Out of memory");
        assert_eq!(explain_status(Status::FileError), "Something went wrong trying to read or write a file");
    }

    #[test]
    fn create_and_open_database() {
        let dir = TempDir::new("db_dir").unwrap();
        {
            Database::create(dir.path()).unwrap();
        }
        {
            Database::open(dir.path(), OpenMode::ReadOnly).unwrap();
        }
        {
            Database::open(dir.path(), OpenMode::ReadWrite).unwrap();
        }
        t!(dir.close());
    }
}
