#[no_mangle]
#[inline(always)]
/// # Safety
pub unsafe extern "C" fn memset(data: *mut u8, val: u8, size: usize) -> *mut core::ffi::c_void{
    for i in 0..size{
        *data.add(i) = val;
    }
    core::mem::transmute(data)
}

#[no_mangle]
#[inline(always)]
/// # Safety
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *mut u8, size: usize) {
    for i in 0..size{
        *dest.add(i) = *src.add(i);
    }
}


#[no_mangle]
#[inline(always)]
/// # Safety
pub unsafe extern "C" fn memcmp(str1: *mut u8, str2: *mut u8, size: usize) -> core::ffi::c_int {
    for i in 0..size{
        match (*str1.add(i)).cmp(&*str2.add(i)){
            core::cmp::Ordering::Less => return -1,
            core::cmp::Ordering::Equal => return 1,
            core::cmp::Ordering::Greater => continue,
        }
    }
    0
}

#[no_mangle]
#[inline(always)]
/// # Safety
pub unsafe extern "C" fn memmove(mut dest: *mut u8, mut src: *mut u8, count: usize) {
    if src.addr() < dest.addr(){
        dest = dest.add(count);
        src = src.add(count);
        for _ in 0..count{
            dest = dest.sub(1);
            src = src.sub(1);
            *dest = *src;
        }
    }else{
        for _ in 0..count{
            *dest = *src;
            dest = dest.add(1);
            src = src.add(1);
        }
    }
}


pub const BUFF_SIZE: usize = 2000;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let mut buf = ['\0' as u8; $crate::core_rust::BUFF_SIZE + 1];
        let buf = unsafe{core::slice::from_raw_parts_mut(buf.as_mut_ptr(), $crate::core_rust::BUFF_SIZE)};
        let mut wrapper = $crate::core_rust::Wrapper::new(buf);
        core::fmt::Write::write_fmt(&mut wrapper, core::format_args!($($arg)*)).expect("Cant write?");
        let str = unsafe{core::str::from_utf8_unchecked(buf)};
        $crate::sys::print_zero_term_str(str);
    };
}

#[macro_export]
#[allow_internal_unstable(format_args_nl)]
macro_rules! println {
    ($($arg:tt)*) => {
        let mut buf = ['\0' as u8; $crate::core_rust::BUFF_SIZE + 1];
        let buf = unsafe{core::slice::from_raw_parts_mut(buf.as_mut_ptr(), $crate::core_rust::BUFF_SIZE)};
        let mut wrapper = $crate::core_rust::Wrapper::new(buf);
        core::fmt::Write::write_fmt(&mut wrapper, core::format_args_nl!($($arg)*)).expect("Cant write?");
        let str = unsafe{core::str::from_utf8_unchecked(buf)};
        $crate::sys::print_zero_term_str(str);
    };
}

pub struct Wrapper<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> Wrapper<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Wrapper {
            buf,
            offset: 0,
        }
    }
}

impl<'a> core::fmt::Write for Wrapper<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();

        // Skip over already-copied data
        let remainder = &mut self.buf[self.offset..];
        // Check if there is space remaining (return error instead of panicking)
        if remainder.len() < bytes.len() { return Err(core::fmt::Error); }
        // Make the two slices the same length
        let remainder = &mut remainder[..bytes.len()];
        // Copy
        remainder.copy_from_slice(bytes);

        // Update offset to avoid overwriting
        self.offset += bytes.len();

        Ok(())
    }
}