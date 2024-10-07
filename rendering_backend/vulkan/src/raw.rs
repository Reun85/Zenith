use std::ffi::CStr;
pub const fn from_cstr_to_array<const MAX_SIZE: usize>(cstr: &CStr) -> [i8; MAX_SIZE] {
    let mut array = [b'\0' as i8; MAX_SIZE];
    let bytes = cstr.to_bytes(); // Get bytes without the null terminator
    let len = if bytes.len() < MAX_SIZE - 1 {
        bytes.len()
    } else {
        MAX_SIZE - 1
    };

    // Copy the bytes into the `name` array, converting u8 to i8
    let mut i = 0;
    while i < len {
        array[i] = bytes[i] as i8;
        i += 1;
    }

    // The array is automatically padded with zeros (because of the array initialization)
    array
}

pub fn string_to_cstring_remove_nuls(s: &str) -> std::ffi::CString {
    let mut bytes = s.as_bytes().to_vec();
    bytes.retain(|&x| x != 0);
    unsafe { std::ffi::CString::from_vec_unchecked(bytes) }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ffi::CString;
    #[test]
    fn test_from_cstr_to_array() {
        let cstr = CString::new("Hello").unwrap();
        let array = from_cstr_to_array::<10>(&cstr);
        assert_eq!(
            array,
            [b'H' as i8, b'e' as i8, b'l' as i8, b'l' as i8, b'o' as i8, 0, 0, 0, 0, 0]
        );

        let x = unsafe { CStr::from_ptr(array.as_ptr()) };
        assert_eq!(x.to_str().unwrap(), "Hello");
    }
    #[test]
    fn test_from_cstr_to_array_with_long_string() {
        let cstr = CString::new("Hello, World!").unwrap();
        let array = from_cstr_to_array::<6>(&cstr);
        assert_eq!(
            array,
            [b'H' as i8, b'e' as i8, b'l' as i8, b'l' as i8, b'o' as i8, 0]
        );
    }

    #[test]
    fn test_string_to_cstring_remove_nuls() {
        let s = "Hello, World!";
        let cstr = string_to_cstring_remove_nuls(s);
        assert_eq!(cstr.to_str().unwrap(), s);
    }
    #[test]
    fn test_string_to_cstring_remove_nuls2() {
        let s = "Hello, World!";
        let s2 = "Hello,\0 World!\0";
        let cstr = string_to_cstring_remove_nuls(s2);
        assert_eq!(cstr.to_str().unwrap(), s);
    }
}
