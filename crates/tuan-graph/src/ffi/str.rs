#[repr(C)]
#[derive(Debug, Clone)]
pub struct Str {
    pub ptr: *const u8,
    pub len: usize,
}

impl Into<String> for Str {
    fn into(self) -> String {
        unsafe {
            assert!(!self.ptr.is_null());
            let slice = std::slice::from_raw_parts(self.ptr, self.len);
            String::from_utf8_lossy(slice).to_string()
        }
    }
}

impl From<String> for Str {
    fn from(s: String) -> Self {
        let len = s.len();
        let boxed_str = s.into_boxed_str();
        let ptr = boxed_str.as_ptr();
        std::mem::forget(boxed_str); // Prevent deallocation
        Str { ptr, len }
    }
}
