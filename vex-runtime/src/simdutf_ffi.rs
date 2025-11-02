// vex-runtime: simdutf FFI bindings
// Ultra-fast UTF-8/UTF-16 validation and conversion using SIMD

use std::os::raw::{c_char, c_int, c_void};

// ============================================================================
// simdutf C bindings (external library)
// ============================================================================

#[link(name = "simdutf")]
extern "C" {
    // UTF-8 validation (20GB/s with AVX-512!)
    fn simdutf_validate_utf8(data: *const u8, len: usize) -> bool;
    fn simdutf_validate_utf8_with_errors(
        data: *const u8,
        len: usize,
        error: *mut simdutf_result,
    ) -> bool;

    // UTF-16 validation
    fn simdutf_validate_utf16le(data: *const u16, len: usize) -> bool;
    fn simdutf_validate_utf16be(data: *const u16, len: usize) -> bool;

    // UTF-8 → UTF-16 conversion
    fn simdutf_convert_utf8_to_utf16le(input: *const u8, len: usize, output: *mut u16) -> usize;
    fn simdutf_convert_utf8_to_utf16be(input: *const u8, len: usize, output: *mut u16) -> usize;

    // UTF-16 → UTF-8 conversion
    fn simdutf_convert_utf16le_to_utf8(input: *const u16, len: usize, output: *mut u8) -> usize;
    fn simdutf_convert_utf16be_to_utf8(input: *const u16, len: usize, output: *mut u8) -> usize;

    // UTF-8 character counting (SIMD)
    fn simdutf_count_utf8(data: *const u8, len: usize) -> usize;

    // UTF-16 character counting
    fn simdutf_count_utf16le(data: *const u16, len: usize) -> usize;

    // Get required buffer size for conversion
    fn simdutf_utf8_length_from_utf16le(data: *const u16, len: usize) -> usize;
    fn simdutf_utf16_length_from_utf8(data: *const u8, len: usize) -> usize;

    // Detect encoding
    fn simdutf_detect_encodings(data: *const u8, len: usize) -> c_int;
}

#[repr(C)]
pub struct simdutf_result {
    pub error: c_int,
    pub position: usize,
}

// Encoding detection flags
pub const SIMDUTF_ENCODING_UTF8: c_int = 1;
pub const SIMDUTF_ENCODING_UTF16_LE: c_int = 2;
pub const SIMDUTF_ENCODING_UTF16_BE: c_int = 4;
pub const SIMDUTF_ENCODING_UTF32_LE: c_int = 8;
pub const SIMDUTF_ENCODING_UTF32_BE: c_int = 16;

// ============================================================================
// Vex FFI Wrappers (safe, convenient)
// ============================================================================

/// Validate UTF-8 string (SIMD optimized, 20GB/s)
/// Returns: true if valid UTF-8, false otherwise
#[no_mangle]
pub extern "C" fn vex_utf8_validate(data: *const u8, len: usize) -> bool {
    if data.is_null() || len == 0 {
        return true; // Empty string is valid
    }

    unsafe { simdutf_validate_utf8(data, len) }
}

/// Validate UTF-16 Little Endian
#[no_mangle]
pub extern "C" fn vex_utf16_validate(data: *const u16, len: usize) -> bool {
    if data.is_null() || len == 0 {
        return true;
    }

    unsafe { simdutf_validate_utf16le(data, len) }
}

/// Count Unicode characters in UTF-8 string (not bytes!)
/// Returns: Number of Unicode codepoints
#[no_mangle]
pub extern "C" fn vex_utf8_count_chars(data: *const u8, len: usize) -> usize {
    if data.is_null() || len == 0 {
        return 0;
    }

    unsafe { simdutf_count_utf8(data, len) }
}

/// Convert UTF-8 to UTF-16 Little Endian
/// output_buffer must be pre-allocated (use vex_utf8_to_utf16_length to get size)
/// Returns: Number of UTF-16 code units written, or 0 on error
#[no_mangle]
pub extern "C" fn vex_utf8_to_utf16(
    input: *const u8,
    input_len: usize,
    output: *mut u16,
    output_capacity: usize,
) -> usize {
    if input.is_null() || output.is_null() || input_len == 0 {
        return 0;
    }

    // Check if output buffer is large enough
    let required_size = unsafe { simdutf_utf16_length_from_utf8(input, input_len) };
    if required_size > output_capacity {
        return 0; // Buffer too small
    }

    unsafe { simdutf_convert_utf8_to_utf16le(input, input_len, output) }
}

/// Convert UTF-16 Little Endian to UTF-8
/// output_buffer must be pre-allocated
/// Returns: Number of bytes written, or 0 on error
#[no_mangle]
pub extern "C" fn vex_utf16_to_utf8(
    input: *const u16,
    input_len: usize,
    output: *mut u8,
    output_capacity: usize,
) -> usize {
    if input.is_null() || output.is_null() || input_len == 0 {
        return 0;
    }

    let required_size = unsafe { simdutf_utf8_length_from_utf16le(input, input_len) };
    if required_size > output_capacity {
        return 0;
    }

    unsafe { simdutf_convert_utf16le_to_utf8(input, input_len, output) }
}

/// Get required buffer size for UTF-8 → UTF-16 conversion
/// Returns: Number of UTF-16 code units needed
#[no_mangle]
pub extern "C" fn vex_utf8_to_utf16_length(data: *const u8, len: usize) -> usize {
    if data.is_null() || len == 0 {
        return 0;
    }

    unsafe { simdutf_utf16_length_from_utf8(data, len) }
}

/// Get required buffer size for UTF-16 → UTF-8 conversion
/// Returns: Number of bytes needed
#[no_mangle]
pub extern "C" fn vex_utf16_to_utf8_length(data: *const u16, len: usize) -> usize {
    if data.is_null() || len == 0 {
        return 0;
    }

    unsafe { simdutf_utf8_length_from_utf16le(data, len) }
}

/// Detect encoding of byte array
/// Returns: Bitmask of possible encodings (SIMDUTF_ENCODING_*)
#[no_mangle]
pub extern "C" fn vex_detect_encoding(data: *const u8, len: usize) -> c_int {
    if data.is_null() || len == 0 {
        return 0;
    }

    unsafe { simdutf_detect_encodings(data, len) }
}

/// Validate UTF-8 with error position
/// error_pos will be set to position of first error, or 0 if valid
/// Returns: true if valid, false otherwise
#[no_mangle]
pub extern "C" fn vex_utf8_validate_with_errors(
    data: *const u8,
    len: usize,
    error_pos: *mut usize,
) -> bool {
    if data.is_null() || len == 0 {
        return true;
    }

    let mut result = simdutf_result {
        error: 0,
        position: 0,
    };

    let valid = unsafe { simdutf_validate_utf8_with_errors(data, len, &mut result) };

    if !error_pos.is_null() {
        unsafe {
            *error_pos = result.position;
        }
    }

    valid
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Check if string is ASCII (subset of UTF-8)
/// Fast path for ASCII-only strings
#[no_mangle]
pub extern "C" fn vex_is_ascii(data: *const u8, len: usize) -> bool {
    if data.is_null() || len == 0 {
        return true;
    }

    let slice = unsafe { std::slice::from_raw_parts(data, len) };

    // SIMD-optimized ASCII check
    slice.iter().all(|&b| b < 128)
}

/// Get UTF-8 character at index (slow, use sparingly!)
/// Returns: Unicode codepoint, or 0xFFFFFFFF on error
#[no_mangle]
pub extern "C" fn vex_utf8_char_at(data: *const u8, len: usize, index: usize) -> u32 {
    if data.is_null() || len == 0 {
        return 0xFFFFFFFF;
    }

    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let s = match std::str::from_utf8(slice) {
        Ok(s) => s,
        Err(_) => return 0xFFFFFFFF,
    };

    match s.chars().nth(index) {
        Some(c) => c as u32,
        None => 0xFFFFFFFF,
    }
}
