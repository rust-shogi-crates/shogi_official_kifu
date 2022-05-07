/// # Safety
/// `s` must be a nul-terminated C string.
pub unsafe fn strlen(s: *const u8) -> usize {
    let mut length = 0;
    while *s.add(length) != 0 {
        length += 1;
    }
    length
}

/// Make a C binding of parse_usi_slice
#[inline(always)] // because C bindings are supposed to directly delegate to this function
pub unsafe fn make_parse_usi_slice_c<T: crate::FromUsi>(dest: &mut T, s: *const u8) -> isize {
    let length = strlen(s);
    let slice = core::slice::from_raw_parts(s, length);
    match T::parse_usi_slice(slice) {
        Ok((slice, resulting_data)) => {
            *dest = resulting_data;
            slice.as_ptr().offset_from(s)
        }
        Err(_) => -1,
    }
}
