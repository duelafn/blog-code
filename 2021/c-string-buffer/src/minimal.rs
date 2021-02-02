
#[cfg(test)]
mod tests {
    use std::ffi::CString;
    type FILE = std::ffi::c_void;
    extern {
        fn fgets(buf: *mut i8, n: i32, stream: *mut FILE) -> *mut i8;
        fn fopen(pathname: *const i8, mode: *const i8) -> *mut FILE;
        fn fclose(stream: *mut FILE) -> i32;
    }

    #[test]
    fn test() {
        let pathname = CString::new("test.txt").expect("CString::new failed");
        let mode = CString::new("r").expect("CString::new failed");
        let fh = unsafe { fopen(pathname.as_ptr(), mode.as_ptr()) };
        if !fh.is_null() {


            // Initialize a buffer of all zeroes
            let mut buf: Vec<u8> = vec![0; 128];

            // Pass a mutable pointer to our C function, we cast it to be of the right type
            let rv = unsafe { fgets(buf.as_mut_ptr() as *mut i8, buf.len() as i32, fh) };

            if !rv.is_null() {
                // Search for the position of the null byte
                let strlen = match buf.iter().position(|&x| 0 == x) {
                    Some(n) => n,
                    None    => buf.len(),
                };
                buf.truncate(strlen); // exclude the trailing nulls

                // Interpret the bytes as a string, provided they are a valid UTF-8 sequence
                let result = String::from_utf8(buf);

                // Success?
                if let Ok(result) = result {
                    assert_eq!(result, String::from("Hello World!\n"));
                } else {
                    assert!(false); // Bummer
                }
            } else {
                assert!(false); // Bummer
            }


            unsafe { fclose(fh) };
        }
    }
}
