#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(asm_experimental_arch)]
#![feature(strict_provenance)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(allow_internal_unstable)]
#![feature(linkage)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

pub mod core_rust;
pub mod sys;

#[no_mangle]
#[linkage = "extern_weak"]
pub static _sp: usize = 0;
#[linkage = "extern_weak"]
#[no_mangle]
pub static _heap: usize = 0;

#[no_mangle]
#[naked]
#[link_section = ".text.start"]
pub extern "C" fn _start() -> ! {
    unsafe {
        core::arch::asm! {
            ".set noat",
            "la $gp, _gp",
            "la $sp, _sp ",
            "move $fp, $sp",
            "jal main",
            "1:",
            "syscall 0",
            "b 1b", options(noreturn),
        }
    }
}

extern "C" {
    pub fn main();
}

pub fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = core::ptr::read_volatile(&dummy);
        core::mem::forget(dummy);
        ret
    }
}
#[inline(always)]
/// # Safety
/// this is the start of the heap dont touch it if you arent the global allocator ;)
pub unsafe fn heap_address() -> *mut u8 {
    let ret;
    core::arch::asm!(
        "la {0}, _heap",
        out(reg) ret
    );
    ret
}
