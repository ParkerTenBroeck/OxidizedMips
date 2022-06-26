#![no_std]
#![no_main]
#![feature(lang_items)]

#![feature(asm_experimental_arch)]
#![feature(strict_provenance)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(panic_info_message)]

pub mod sys;
pub mod core_rust;

#[no_mangle]
#[naked]
#[link_section = ".text.start"]
pub extern "C" fn __start() -> !{
    unsafe{
        core::arch::asm!{
            "la $gp, _gp",
            "la $sp, _sp ",
            "move $fp, $sp",
            "jal main",
            "syscall 0", options(noreturn)
        }
    }
}

pub fn black_box<T>(dummy: T) -> T{
    unsafe {
        let ret = core::ptr::read_volatile(&dummy);
        core::mem::forget(dummy);
        ret
    }
}




