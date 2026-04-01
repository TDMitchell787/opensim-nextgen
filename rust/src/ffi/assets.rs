//! FFI bindings for the asset pipeline.

extern "C" {
    fn asset_processor_new() -> *mut u8;
    fn asset_processor_destroy(processor: *mut u8);
    fn asset_processor_process(
        processor: *mut u8,
        asset_type: *const libc::c_char,
        data: *const u8,
        data_len: usize,
        // out params
        out_data: *mut *mut u8,
        out_len: *mut usize,
    ) -> i32;
}

pub struct AssetProcessor {
    ptr: *mut u8,
}

impl AssetProcessor {
    pub fn new() -> Self {
        let ptr = unsafe { asset_processor_new() };
        Self { ptr }
    }

    pub fn process(&self, asset_type: &str, data: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        let asset_type_cstr = std::ffi::CString::new(asset_type)?;
        let mut out_data: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;

        let result = unsafe {
            asset_processor_process(
                self.ptr,
                asset_type_cstr.as_ptr(),
                data.as_ptr(),
                data.len(),
                &mut out_data,
                &mut out_len,
            )
        };

        if result != 0 {
            return Err(anyhow::anyhow!("Asset processing failed with code {}", result));
        }

        if out_data.is_null() {
            return Ok(Vec::new());
        }

        let output = unsafe { std::slice::from_raw_parts(out_data, out_len).to_vec() };
        // NOTE: Assume the Zig side allocates memory that Rust can free with the global allocator.
        // This is okay if Zig uses `std.heap.c_allocator`.
        unsafe { libc::free(out_data as *mut libc::c_void) };

        Ok(output)
    }
}

impl Drop for AssetProcessor {
    fn drop(&mut self) {
        unsafe { asset_processor_destroy(self.ptr) };
    }
} 