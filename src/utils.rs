use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
pub fn parse_utf16_slice(string_slice : &[u16]) -> Option<OsString>{
    let null_index = string_slice.iter().position(|c| c == &0)?;

    Some(OsString::from_wide(&string_slice[..null_index]))
}