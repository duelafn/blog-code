use serde_json;
use serde_json::value::Value;

/// Read some *ffi-owned* JSON, do some processing and return a *rust-owned* string.
/// The FFI caller will need to call `mylib_free_string` on the returned pointer.
#[no_mangle]
pub extern fn mylib_myfunc_str(raw: *const std::os::raw::c_char) -> *const i8 {
    // *Copy* input to a rust-owned string
    if raw.is_null() { return std::ptr::null(); }
    let bytes = unsafe { std::ffi::CStr::from_ptr(raw).to_bytes() };

    // Internal processing
    let res = String::from_utf8(bytes.to_vec())
                .map_err(|e| format!("Encoding error: {}", e))
                .and_then(|req| myfunc(&req));

    // Formatting a response
    let rv = match serde_json::to_string(&res) {
        Ok(json) => json,
        // "rv" must be valid JSON, so we don't try including the error message
        Err(_)   => String::from("{\"Err\":\"JSON encode error\"}"),
    };

    // Return a *python-owned* string
    return match std::ffi::CString::new(rv) {
        Ok(cstr) => cstr.into_raw(),
        Err(_)   => std::ptr::null(),
    }
}

fn myfunc(request: &str) -> Result<Value, String> {
    let req: Value = serde_json::from_str(&request).map_err(|e| format!("JSON Parse error: {}", e))?;

    // Do whatever we like with the Value.
    if let Some(Value::String(val)) = req.get("plugh") {
        return Ok(Value::from(format!("plugh has length {}", val.len())));
    } else {
        return Err(String::from("plugh not present or not valid"));
    }
}

/// FFI users who receive a returned string from us MUST call this function
/// to free that string.
#[no_mangle]
pub extern fn mylib_free_string(raw: *mut std::os::raw::c_char) {
    unsafe { let _ = std::ffi::CString::from_raw(raw); }
}
