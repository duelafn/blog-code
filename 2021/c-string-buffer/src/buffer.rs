
pub struct CStrBuf {
    vec: Vec<u8>,
}
impl CStrBuf {
    /// New buffer. Since this buffer is intended to be used with C
    /// functions, we panic if the requested length is larger than
    /// i32::MAX.
    pub fn new(len: usize) -> CStrBuf {
        // Safety, The intended use is for passing to C functions which
        // expect an i32 length. Check the length at construction. I panic,
        // but one could instead change return type to Option<CStrBuf>.
        if len > (std::i32::MAX as usize) { panic!("Expected length <= i32::MAX"); }
        // Fully initialized buffer to protect against undefined behavior
        // of reading uninitialized memory. See:
        //    https://www.ralfj.de/blog/2019/07/14/uninit.html
        //    https://users.rust-lang.org/t/is-it-ub-to-read-uninitialized-integers-in-a-vector-why/39682
        CStrBuf { vec: vec![0; len] }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const i8 {
        self.vec.as_ptr() as *const i8    // u8 -> i8 is a safe cast
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut i8 {
        self.vec.as_mut_ptr() as *mut i8  // u8 -> i8 is a safe cast
    }

    #[inline]
    pub fn buffer_len(&self) -> i32 {
        self.vec.len() as i32   // cast to i32 is safe because of checks in new()
    }

    /// C-style string length, search for the first null byte.
    /// Fall back to full vector length.
    #[inline]
    pub fn strlen(&self) -> usize {
        // match self.vec.iter().position(|&x| 0 == x) {
        match memchr::memchr(0, &self.vec) {
            Some(n) => n,
            None    => self.vec.len(),
        }
    }

    /// Copy null-terminated contents to a new String
    #[inline]
    pub fn to_string(&self) -> Result<String, std::string::FromUtf8Error> {
        let len = self.strlen();
        return String::from_utf8(self.vec[0..len].to_vec());
    }

    /// Zero-copy, consume the buffer into a string
    #[inline]
    pub fn into_string(mut self) -> Result<String, std::string::FromUtf8Error> {
        let len = self.strlen();
        self.vec.truncate(len);
        return String::from_utf8(self.vec);
    }

    /// Zero-copy borrow of the data as a &str
    #[inline]
    pub fn to_str(&self) -> Result<&str, std::str::Utf8Error> {
        let len = self.strlen();
        return std::str::from_utf8(&self.vec[0..len]);
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::CString;
    type FILE = std::ffi::c_void;
    extern {
        fn fgets(buf: *mut i8, n: i32, stream: *mut FILE) -> *mut i8;
        fn fopen(pathname: *const i8, mode: *const i8) -> *mut FILE;
        fn fclose(stream: *mut FILE) -> i32;
    }

    #[test]
    fn test_into_string() {
        let pathname = CString::new("test.txt").expect("CString::new failed");
        let mode = CString::new("r").expect("CString::new failed");

        let fh = unsafe { fopen(pathname.as_ptr(), mode.as_ptr()) };
        assert!(!fh.is_null());

        let mut buf = CStrBuf::new(128);
        let saved_ptr = buf.as_ptr();

        let rv = unsafe { fgets(buf.as_mut_ptr(), buf.buffer_len(), fh) };
        unsafe { fclose(fh) };
        assert!(!rv.is_null());

        let content = buf.into_string().unwrap();
        assert_eq!(content, String::from("Hello World!\n"));

        // no move or allocation chsnges
        assert_eq!(content.capacity(), 128);
        assert!(saved_ptr == content.as_ptr() as *const i8);
    }

    #[test]
    fn test_to_string() {
        let pathname = CString::new("test.txt").expect("CString::new failed");
        let mode = CString::new("r").expect("CString::new failed");

        let fh = unsafe { fopen(pathname.as_ptr(), mode.as_ptr()) };
        assert!(!fh.is_null());

        let mut buf = CStrBuf::new(128);
        let saved_ptr = buf.as_ptr();

        let rv = unsafe { fgets(buf.as_mut_ptr(), buf.buffer_len(), fh) };
        unsafe { fclose(fh) };
        assert!(!rv.is_null());

        let content = buf.to_string().unwrap();
        assert_eq!(content, String::from("Hello World!\n"));

        // copied out of buf and right-sized
        assert!(saved_ptr != content.as_ptr() as *const i8);
        assert_eq!(content.capacity(), 13);
    }

    #[test]
    fn test_to_str() {
        let pathname = CString::new("test.txt").expect("CString::new failed");
        let mode = CString::new("r").expect("CString::new failed");

        let mut buf = CStrBuf::new(7);
        let saved_ptr = buf.as_ptr() as *const u8;

        let fh = unsafe { fopen(pathname.as_ptr(), mode.as_ptr()) };

        let rv = unsafe { fgets(buf.as_mut_ptr(), buf.buffer_len(), fh) };
        assert!(!rv.is_null());
        let a = buf.to_str().unwrap();
        assert_eq!(a, "Hello ");
        assert_eq!(saved_ptr, a.as_ptr());

        let rv = unsafe { fgets(buf.as_mut_ptr(), buf.buffer_len(), fh) };
        assert!(!rv.is_null());
        let b = buf.to_str().unwrap();
        assert_eq!(b, "World!");
        assert_eq!(saved_ptr, b.as_ptr());
        // println!("{}", a); // Uncomment for compile error (can't use a after reusing buffer)

        let rv = unsafe { fgets(buf.as_mut_ptr(), buf.buffer_len(), fh) };
        assert!(!rv.is_null());
        let c = buf.to_str().unwrap();
        assert_eq!(c, "\n");
        assert_eq!(saved_ptr, c.as_ptr());

        let rv = unsafe { fgets(buf.as_mut_ptr(), buf.buffer_len(), fh) };
        assert!(rv.is_null());
        unsafe { fclose(fh) };

        // Last read fails (end of file), so buffer will not be modified!
        assert_eq!(buf.to_str().unwrap(), "\n");

        // no move or allocation chsnges
        assert_eq!(buf.vec.capacity(), 7);
        assert!(saved_ptr == buf.as_ptr() as *const u8);
    }
}
