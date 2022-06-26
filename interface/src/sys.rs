
use core::{arch::asm};


#[inline(always)]
pub fn halt() -> !{
    syscall_0_0::<0>();
    unsafe{
        core::hint::unreachable_unchecked();
    }
}

#[inline(always)]
pub fn print_isize(num: isize){
    syscall_1_0::<1>(num as usize);
}

#[inline(always)]
pub fn print_zero_term_str(str: &str){
    syscall_1_0::<4>(str.as_ptr().addr());
}

#[inline(always)]
pub fn print_str(str: &str){
    for char in str.chars(){
        print_char(char);
    }
}

#[inline(always)]
pub fn read_isize() -> isize{
    syscall_0_1::<5>() as isize
}

pub fn rand_range(min: isize, max: isize) -> isize{
    syscall_2_1::<99>(min as usize, max as usize) as isize
}

#[inline(always)]
pub fn print_char(char: char){
    syscall_1_0::<101>(char as usize);
}

pub fn is_key_pressed(key: char) -> bool{
    syscall_1_1::<104>(key as u32 as usize) == 1
}

#[inline(always)]
pub fn sleep_delta_mills(mills: usize){
    syscall_1_0::<106>(mills);
}

#[inline(always)]
pub fn sleep_mills(mills: usize){
    syscall_1_0::<105>(mills);
}

#[inline(always)]
pub fn old_breakpoint(){
    syscall_0_0::<111>();
}

#[inline(always)]
pub fn init_screen(width: usize, height: usize){
    syscall_2_0::<150>(width, height);
}

#[inline(always)]
pub fn set_pixel_coords(x: usize, y: usize, color: usize){
    syscall_3_0::<151>(x, y, color);
}

#[inline(always)]
pub fn set_pixel_index(index: usize, color: usize){
    syscall_2_0::<152>(index, color);
}

#[inline(always)]
pub fn update_screen(){
    syscall_0_0::<153>();
}

#[inline(always)]
pub fn fill_screen(color: usize){
    syscall_1_0::<156>(color);
}

#[inline(always)]
fn syscall_0_0<const CALL_ID: usize>(){
    unsafe{
        asm!(
            "syscall {0}",
            const(CALL_ID),
        )
    }
}

#[inline(always)]
fn syscall_1_0<const CALL_ID: usize>(arg1: usize){
    unsafe{
        asm!(
            "syscall {0}",
            const(CALL_ID),
            in("$4") arg1,
        )
    }
}

#[inline(always)]
fn syscall_0_1<const CALL_ID: usize>() -> usize{
    unsafe{
        let ret1;
        asm!(
            "syscall {0}",
            const(CALL_ID),
            out("$2") ret1,
        );
        ret1
    }
}

#[inline(always)]
fn syscall_1_1<const CALL_ID: usize>(arg1: usize) -> usize{
    unsafe{
        let ret1;
        asm!(
            "syscall {0}",
            const(CALL_ID),
            in("$4") arg1,
            out("$2") ret1,
        );
        ret1
    }
}

#[inline(always)]
fn syscall_2_0<const CALL_ID: usize>(arg1: usize, arg2: usize){
    unsafe{
        asm!(
            "syscall {0}",
            const(CALL_ID),
            in("$4") arg1,
            in("$5") arg2,
        );
    }
}

#[inline(always)]
fn syscall_3_0<const CALL_ID: usize>(arg1: usize, arg2: usize, arg3: usize){
    unsafe{
        asm!(
            "syscall {0}",
            const(CALL_ID),
            in("$4") arg1,
            in("$5") arg2,
            in("$6") arg3,
        );
    }
}

#[inline(always)]
fn syscall_2_1<const CALL_ID: usize>(arg1: usize, arg2: usize) -> usize{
    unsafe{
        let ret1;
        asm!(
            "syscall {0}",
            const(CALL_ID),
            in("$4") arg1,
            in("$5") arg2,
            out("$2") ret1,
        );
        ret1
    }
}