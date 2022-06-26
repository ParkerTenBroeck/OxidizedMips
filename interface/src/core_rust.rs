#[no_mangle]
#[inline(always)]
pub unsafe extern "C" fn memset(data: *mut u8, val: u8, size: usize) -> *mut core::ffi::c_void{
    for i in 0..size{
        *data.add(i) = val;
    }
    core::mem::transmute(data)
}

#[no_mangle]
#[inline(always)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *mut u8, size: usize) {
    for i in 0..size{
        *dest.add(i) = *src.add(i);
    }
}

#[no_mangle]
#[inline(always)]
pub unsafe extern "C" fn memmove(dest: *mut u8, source: *mut u8, size: usize) {
    panic!("not implemented")
    // for i in 0..size{
    //     *data.add(i) = val;
    // }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let mut buf = ['\0' as u8; 2001];
        let mut wrapper = interface::core_rust::Wrapper::new(&mut buf);
        use core::fmt::Write;
        core::write!(wrapper, $($arg)*).expect("cant write panic info??");
        let str = unsafe{core::str::from_utf8_unchecked(&buf)};
        interface::sys::print_zero_term_str(str);
        //$dst.write_fmt($crate::format_args!($($arg)*))
    };
}

pub struct Wrapper<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> Wrapper<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Wrapper {
            buf: buf,
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

#[panic_handler]
#[no_mangle]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut buf = ['a' as u8; 2000];
    let mut wrapper = Wrapper::new(&mut buf);
    use core::fmt::Write;
    core::write!(wrapper, "{}\0", info).expect("cant write panic info??");
    let str = unsafe{core::str::from_utf8_unchecked(&buf)};
    crate::sys::print_zero_term_str(str);
    crate::sys::print_zero_term_str("EXITING\0");
    crate::sys::halt()
}