#![no_std]
#![no_main]
#![feature(const_for)]
#![feature(strict_provenance)]
#![feature(default_alloc_error_handler)]

extern crate alloc;

use util::display::Color;

use crate::util::display::draw_chacater;

extern crate interface;

pub mod tetris;
pub mod util;

pub mod allocater {
    use core::{
        alloc::{GlobalAlloc, Layout},
        cell::UnsafeCell,
        fmt::Debug,
        mem::{align_of, size_of},
        ptr::{self, NonNull},
    };

    #[repr(C)]
    struct AllocatedMemory {
        size: usize,
        align: usize,
        data: *mut u8,
        next: Option<NonNull<AllocatedMemory>>,
    }
    impl Debug for AllocatedMemory {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("AllocatedMemory")
                .field("_start_", &(self as *const Self))
                .field("size", &self.size)
                .field("align", &self.align)
                .field("data", &self.data)
                .field("next", &self.next)
                .field("_end_", unsafe { &self.alligned_end() })
                .finish()
        }
    }

    impl AllocatedMemory {
        unsafe fn alligned_end(&self) -> *mut u8 {
            self.data.map_addr(|mut end| {
                end += self.size;
                end += align_of::<&Self>() - 1;
                end &= !(align_of::<&Self>() - 1);
                end
            })
        }

        #[allow(unused)]
        unsafe fn calc_free_space_ahead(&self) -> usize {
            if let Option::Some(next) = self.next {
                next.as_ptr().addr() - self.alligned_end().addr()
            } else {
                usize::MAX - (self.data.addr() + self.size)
            }
        }
    }
    struct AllocatedMemoryIterator {
        current: Option<NonNull<AllocatedMemory>>,
    }
    impl Iterator for AllocatedMemoryIterator {
        type Item = NonNull<AllocatedMemory>;

        fn next(&mut self) -> Option<Self::Item> {
            unsafe {
                let curr = self.current;
                if let Option::Some(curr) = curr {
                    self.current = (*curr.as_ptr()).next;
                }
                curr
            }
        }
    }

    struct SimpleAccocator {
        head: UnsafeCell<*mut AllocatedMemory>,
    }
    #[global_allocator]
    static ALLOCATOR: SimpleAccocator = SimpleAccocator {
        head: UnsafeCell::new(0 as *mut AllocatedMemory),
    };
    unsafe impl Send for SimpleAccocator {}
    unsafe impl Sync for SimpleAccocator {}

    unsafe impl GlobalAlloc for SimpleAccocator {
        unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
            if (*self.head.get()).is_null() {
                *self.head.get() = core::mem::transmute(interface::heap_address());
                let head = *self.head.get();
                Self::add_new(head, &layout);
                (*head).data
            } else {
                let iter = AllocatedMemoryIterator {
                    current: Option::Some(NonNull::new_unchecked(*self.head.get())),
                };
                for alloc in iter {
                    let mut start = (*alloc.as_ptr()).alligned_end().addr();
                    start += size_of::<AllocatedMemory>();
                    start += layout.align() - 1;
                    start &= !(layout.align() - 1);
                    if let Option::Some(next) = (*alloc.as_ptr()).next {
                        if start < next.addr().into() {
                            let new = (*alloc.as_ptr()).alligned_end();
                            let new: *mut AllocatedMemory = core::mem::transmute(new);
                            Self::add_new(new, &layout);
                            (*new).next = (*alloc.as_ptr()).next;
                            (*alloc.as_ptr()).next = Option::Some(NonNull::new_unchecked(new));
                            return (*new).data;
                        } else {
                            continue;
                        }
                    } else {
                        let new = (*alloc.as_ptr()).alligned_end();
                        let new: *mut AllocatedMemory = core::mem::transmute(new);
                        Self::add_new(new, &layout);
                        (*alloc.as_ptr()).next = Option::Some(NonNull::new_unchecked(new));
                        return (*new).data;
                    }
                }
                ptr::null_mut()
            }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
            if (*self.head.get()).is_null() {
                panic!();
            } else {
                let mut iter = AllocatedMemoryIterator {
                    current: Option::Some(NonNull::new_unchecked(*self.head.get())),
                };
                let mut last = iter.next().unwrap();
                if (*last.as_ptr()).data == ptr {
                    (*self.head.get()) = match (*last.as_ptr()).next {
                        Some(some) => some.as_ptr(),
                        None => ptr::null_mut(),
                    };
                    return;
                }
                for alloc in iter {
                    if (*alloc.as_ptr()).data == ptr {
                        (*last.as_ptr()).next = (*alloc.as_ptr()).next;
                        return;
                    } else {
                        last = alloc;
                    }
                }
            }
            panic!("to free: {:?}\nhead: {:?}", ptr, (**self.head.get()));
        }
    }
    impl SimpleAccocator {
        unsafe fn add_new(alloc: *mut AllocatedMemory, layout: &Layout) {
            (*alloc).align = layout.align();
            (*alloc).size = layout.size();
            (*alloc).next = Option::None;
            (*alloc).data = core::mem::transmute(alloc.map_addr(|mut add| {
                let align_mask_to_round_down = !(layout.align() - 1);
                add += size_of::<AllocatedMemory>();
                add += layout.align() - 1;
                add & align_mask_to_round_down
            }));
        }
        #[allow(unused)]
        unsafe fn count_allocations(&self) -> usize {
            if self.head.get().is_null() {
                0
            } else {
                let iter = AllocatedMemoryIterator {
                    current: Option::Some(NonNull::new_unchecked(*self.head.get())),
                };
                iter.count()
            }
        }
    }
}

struct MenuScreen {
    scroll_index: usize,
}

impl MenuScreen {
    pub fn new() -> Self {
        let mut s = Self { scroll_index: 0 };
        s.init();
        s
    }
    pub fn init(&mut self) {
        interface::sys::init_screen(
            crate::tetris::renderer::WIDTH,
            crate::tetris::renderer::HEIGHT,
        );
    }
    pub fn update(&mut self) -> bool {
        interface::sys::fill_screen(Color::from_rgb(50, 50, 50).into());
        draw_wiggly_text();

        // let mut vec = alloc::vec::Vec::new();
        // for i in 0..interface::sys::rand_range(30, 300){
        //     vec.insert((i>>1) as usize, Box::new((i, interface::sys::rand_range(i32::MIN, i32::MAX))))
        // }
        // drop(vec);

        self.update_demo_selection();
        interface::sys::update_screen_vsync();

        !interface::sys::is_key_pressed('\x08')
    }

    pub fn update_demo_selection(&mut self) {
        let items = [("Tetris", crate::tetris::run_tetris)];

        if interface::sys::is_key_pressed('\n') {
            items[self.scroll_index].1();
            while interface::sys::is_key_pressed('\x08') {
                interface::sys::sleep_mills(1);
            }
            interface::sys::init_screen(
                crate::tetris::renderer::WIDTH,
                crate::tetris::renderer::HEIGHT,
            );
        }
        if interface::sys::is_key_pressed('s') {}
        if interface::sys::is_key_pressed('w') {}
    }
}

#[no_mangle]
pub fn main() {
    let mut menu = MenuScreen::new();
    while menu.update() {
        //interface::sys::sleep_delta_mills(16);
        if interface::sys::is_key_pressed('p') {
            interface::sys::set_pixel_index(u32::MAX, 0)
        }
    }
}

pub const fn convert_str<const S: usize>(str: &[u8; S]) -> [u8; S] {
    let mut conv = [0u8; S];
    let mut i = 0;
    while i < S {
        conv[i] = match str[i] as char {
            'a'..='z' => ((str[i]) - b'a') + 65,
            'A'..='Z' => ((str[i]) - b'A') + 33,
            '0'..='9' => str[i] - b'0' + 16,
            ' ' => 0,
            '!' => 1,
            '#' => 3,
            '/' => 15,
            '~' => 94,
            _ => 0,
        };
        i += 1;
    }
    conv
}

fn draw_wiggly_text() {
    const TEXT: &[u8] = convert_str(b"~~~~~MIPS Interactive Demo!~~~~~").as_slice();
    //let text = [33,34,35,36,37,38,39];Multi
    let micros = interface::sys::get_micros();
    for (i, c) in TEXT.iter().enumerate() {
        let init = Color::from_rgb(
            (((micros as u32 / 10000).wrapping_sub(i as u32 * 16)) % 255) as u8,
            255,
            255,
        );
        let color = interface::sys::hsv_to_rgb(init.into()).into();
        draw_chacater(
            [
                i as u32 * 8,
                sin((i as u32 * 64 + (micros as u32 / 1000)) % 1024) as u32 * 16 / 65536,
            ],
            *c as u32,
            //((micros / 1000) % 255) as u8
            color,
            Color::clear(),
        );
    }
}

#[panic_handler]
#[no_mangle]
fn panic(info: &core::panic::PanicInfo) -> ! {
    interface::println!("{}", info);
    interface::println!("STOPPING");
    interface::sys::halt();
}

pub fn sin(angle: u32) -> u16 {
    const TABLE: [u16; 1024] = [
        0x8000, 0x80c9, 0x8192, 0x825b, 0x8324, 0x83ee, 0x84b7, 0x8580, 0x8649, 0x8712, 0x87db,
        0x88a4, 0x896c, 0x8a35, 0x8afe, 0x8bc6, 0x8c8e, 0x8d57, 0x8e1f, 0x8ee7, 0x8fae, 0x9076,
        0x913e, 0x9205, 0x92cc, 0x9393, 0x945a, 0x9521, 0x95e7, 0x96ad, 0x9773, 0x9839, 0x98fe,
        0x99c4, 0x9a89, 0x9b4d, 0x9c12, 0x9cd6, 0x9d9a, 0x9e5e, 0x9f21, 0x9fe4, 0xa0a7, 0xa169,
        0xa22b, 0xa2ed, 0xa3af, 0xa470, 0xa530, 0xa5f1, 0xa6b1, 0xa770, 0xa830, 0xa8ef, 0xa9ad,
        0xaa6b, 0xab29, 0xabe6, 0xaca3, 0xad5f, 0xae1b, 0xaed7, 0xaf92, 0xb04d, 0xb107, 0xb1c0,
        0xb27a, 0xb332, 0xb3ea, 0xb4a2, 0xb559, 0xb610, 0xb6c6, 0xb77c, 0xb831, 0xb8e5, 0xb999,
        0xba4d, 0xbb00, 0xbbb2, 0xbc64, 0xbd15, 0xbdc6, 0xbe76, 0xbf25, 0xbfd4, 0xc082, 0xc12f,
        0xc1dc, 0xc288, 0xc334, 0xc3df, 0xc489, 0xc533, 0xc5dc, 0xc684, 0xc72c, 0xc7d3, 0xc879,
        0xc91f, 0xc9c3, 0xca67, 0xcb0b, 0xcbae, 0xcc4f, 0xccf1, 0xcd91, 0xce31, 0xced0, 0xcf6e,
        0xd00b, 0xd0a8, 0xd144, 0xd1df, 0xd279, 0xd313, 0xd3ac, 0xd443, 0xd4db, 0xd571, 0xd606,
        0xd69b, 0xd72f, 0xd7c2, 0xd854, 0xd8e5, 0xd975, 0xda05, 0xda93, 0xdb21, 0xdbae, 0xdc3a,
        0xdcc5, 0xdd4f, 0xddd9, 0xde61, 0xdee9, 0xdf6f, 0xdff5, 0xe07a, 0xe0fd, 0xe180, 0xe202,
        0xe283, 0xe303, 0xe382, 0xe400, 0xe47d, 0xe4fa, 0xe575, 0xe5ef, 0xe668, 0xe6e0, 0xe758,
        0xe7ce, 0xe843, 0xe8b7, 0xe92b, 0xe99d, 0xea0e, 0xea7e, 0xeaed, 0xeb5b, 0xebc8, 0xec34,
        0xec9f, 0xed09, 0xed72, 0xedda, 0xee41, 0xeea7, 0xef0b, 0xef6f, 0xefd1, 0xf033, 0xf093,
        0xf0f2, 0xf150, 0xf1ad, 0xf209, 0xf264, 0xf2be, 0xf316, 0xf36e, 0xf3c4, 0xf41a, 0xf46e,
        0xf4c1, 0xf513, 0xf564, 0xf5b3, 0xf602, 0xf64f, 0xf69b, 0xf6e6, 0xf730, 0xf779, 0xf7c1,
        0xf807, 0xf84d, 0xf891, 0xf8d4, 0xf916, 0xf956, 0xf996, 0xf9d4, 0xfa11, 0xfa4d, 0xfa88,
        0xfac1, 0xfafa, 0xfb31, 0xfb67, 0xfb9c, 0xfbd0, 0xfc02, 0xfc33, 0xfc63, 0xfc92, 0xfcc0,
        0xfcec, 0xfd17, 0xfd42, 0xfd6a, 0xfd92, 0xfdb8, 0xfdde, 0xfe01, 0xfe24, 0xfe46, 0xfe66,
        0xfe85, 0xfea3, 0xfec0, 0xfedb, 0xfef5, 0xff0e, 0xff26, 0xff3c, 0xff52, 0xff66, 0xff79,
        0xff8a, 0xff9b, 0xffaa, 0xffb8, 0xffc4, 0xffd0, 0xffda, 0xffe3, 0xffeb, 0xfff1, 0xfff6,
        0xfffa, 0xfffd, 0xffff, 0xffff, 0xfffe, 0xfffc, 0xfff8, 0xfff4, 0xffee, 0xffe7, 0xffdf,
        0xffd5, 0xffca, 0xffbe, 0xffb1, 0xffa2, 0xff93, 0xff82, 0xff6f, 0xff5c, 0xff47, 0xff31,
        0xff1a, 0xff02, 0xfee8, 0xfece, 0xfeb1, 0xfe94, 0xfe76, 0xfe56, 0xfe35, 0xfe13, 0xfdf0,
        0xfdcb, 0xfda5, 0xfd7e, 0xfd56, 0xfd2d, 0xfd02, 0xfcd6, 0xfca9, 0xfc7b, 0xfc4b, 0xfc1b,
        0xfbe9, 0xfbb6, 0xfb82, 0xfb4c, 0xfb16, 0xfade, 0xfaa5, 0xfa6b, 0xfa2f, 0xf9f3, 0xf9b5,
        0xf976, 0xf936, 0xf8f5, 0xf8b2, 0xf86f, 0xf82a, 0xf7e4, 0xf79d, 0xf755, 0xf70c, 0xf6c1,
        0xf675, 0xf629, 0xf5db, 0xf58c, 0xf53b, 0xf4ea, 0xf498, 0xf444, 0xf3ef, 0xf399, 0xf342,
        0xf2ea, 0xf291, 0xf237, 0xf1db, 0xf17f, 0xf121, 0xf0c3, 0xf063, 0xf002, 0xefa0, 0xef3d,
        0xeed9, 0xee74, 0xee0e, 0xeda6, 0xed3e, 0xecd5, 0xec6a, 0xebff, 0xeb92, 0xeb24, 0xeab6,
        0xea46, 0xe9d6, 0xe964, 0xe8f1, 0xe87d, 0xe809, 0xe793, 0xe71c, 0xe6a4, 0xe62c, 0xe5b2,
        0xe537, 0xe4bc, 0xe43f, 0xe3c1, 0xe343, 0xe2c3, 0xe243, 0xe1c1, 0xe13f, 0xe0bc, 0xe037,
        0xdfb2, 0xdf2c, 0xdea5, 0xde1d, 0xdd94, 0xdd0a, 0xdc80, 0xdbf4, 0xdb68, 0xdada, 0xda4c,
        0xd9bd, 0xd92d, 0xd89c, 0xd80b, 0xd778, 0xd6e5, 0xd651, 0xd5bc, 0xd526, 0xd48f, 0xd3f8,
        0xd35f, 0xd2c6, 0xd22c, 0xd192, 0xd0f6, 0xd05a, 0xcfbd, 0xcf1f, 0xce80, 0xcde1, 0xcd41,
        0xcca0, 0xcbff, 0xcb5c, 0xcab9, 0xca16, 0xc971, 0xc8cc, 0xc826, 0xc77f, 0xc6d8, 0xc630,
        0xc588, 0xc4de, 0xc434, 0xc38a, 0xc2de, 0xc232, 0xc186, 0xc0d9, 0xc02b, 0xbf7c, 0xbecd,
        0xbe1e, 0xbd6d, 0xbcbd, 0xbc0b, 0xbb59, 0xbaa6, 0xb9f3, 0xb940, 0xb88b, 0xb7d6, 0xb721,
        0xb66b, 0xb5b5, 0xb4fe, 0xb446, 0xb38e, 0xb2d6, 0xb21d, 0xb164, 0xb0aa, 0xafef, 0xaf34,
        0xae79, 0xadbd, 0xad01, 0xac45, 0xab88, 0xaaca, 0xaa0c, 0xa94e, 0xa88f, 0xa7d0, 0xa711,
        0xa651, 0xa591, 0xa4d0, 0xa40f, 0xa34e, 0xa28c, 0xa1ca, 0xa108, 0xa045, 0x9f83, 0x9ebf,
        0x9dfc, 0x9d38, 0x9c74, 0x9bb0, 0x9aeb, 0x9a26, 0x9961, 0x989c, 0x97d6, 0x9710, 0x964a,
        0x9584, 0x94bd, 0x93f7, 0x9330, 0x9269, 0x91a1, 0x90da, 0x9012, 0x8f4b, 0x8e83, 0x8dbb,
        0x8cf3, 0x8c2a, 0x8b62, 0x8a99, 0x89d1, 0x8908, 0x883f, 0x8776, 0x86ad, 0x85e4, 0x851b,
        0x8452, 0x8389, 0x82c0, 0x81f7, 0x812d, 0x8064, 0x7f9b, 0x7ed2, 0x7e08, 0x7d3f, 0x7c76,
        0x7bad, 0x7ae4, 0x7a1b, 0x7952, 0x7889, 0x77c0, 0x76f7, 0x762e, 0x7566, 0x749d, 0x73d5,
        0x730c, 0x7244, 0x717c, 0x70b4, 0x6fed, 0x6f25, 0x6e5e, 0x6d96, 0x6ccf, 0x6c08, 0x6b42,
        0x6a7b, 0x69b5, 0x68ef, 0x6829, 0x6763, 0x669e, 0x65d9, 0x6514, 0x644f, 0x638b, 0x62c7,
        0x6203, 0x6140, 0x607c, 0x5fba, 0x5ef7, 0x5e35, 0x5d73, 0x5cb1, 0x5bf0, 0x5b2f, 0x5a6e,
        0x59ae, 0x58ee, 0x582f, 0x5770, 0x56b1, 0x55f3, 0x5535, 0x5477, 0x53ba, 0x52fe, 0x5242,
        0x5186, 0x50cb, 0x5010, 0x4f55, 0x4e9b, 0x4de2, 0x4d29, 0x4c71, 0x4bb9, 0x4b01, 0x4a4a,
        0x4994, 0x48de, 0x4829, 0x4774, 0x46bf, 0x460c, 0x4559, 0x44a6, 0x43f4, 0x4342, 0x4292,
        0x41e1, 0x4132, 0x4083, 0x3fd4, 0x3f26, 0x3e79, 0x3dcd, 0x3d21, 0x3c75, 0x3bcb, 0x3b21,
        0x3a77, 0x39cf, 0x3927, 0x3880, 0x37d9, 0x3733, 0x368e, 0x35e9, 0x3546, 0x34a3, 0x3400,
        0x335f, 0x32be, 0x321e, 0x317f, 0x30e0, 0x3042, 0x2fa5, 0x2f09, 0x2e6d, 0x2dd3, 0x2d39,
        0x2ca0, 0x2c07, 0x2b70, 0x2ad9, 0x2a43, 0x29ae, 0x291a, 0x2887, 0x27f4, 0x2763, 0x26d2,
        0x2642, 0x25b3, 0x2525, 0x2497, 0x240b, 0x237f, 0x22f5, 0x226b, 0x21e2, 0x215a, 0x20d3,
        0x204d, 0x1fc8, 0x1f43, 0x1ec0, 0x1e3e, 0x1dbc, 0x1d3c, 0x1cbc, 0x1c3e, 0x1bc0, 0x1b43,
        0x1ac8, 0x1a4d, 0x19d3, 0x195b, 0x18e3, 0x186c, 0x17f6, 0x1782, 0x170e, 0x169b, 0x1629,
        0x15b9, 0x1549, 0x14db, 0x146d, 0x1400, 0x1395, 0x132a, 0x12c1, 0x1259, 0x11f1, 0x118b,
        0x1126, 0x10c2, 0x105f, 0xffd, 0xf9c, 0xf3c, 0xede, 0xe80, 0xe24, 0xdc8, 0xd6e, 0xd15,
        0xcbd, 0xc66, 0xc10, 0xbbb, 0xb67, 0xb15, 0xac4, 0xa73, 0xa24, 0x9d6, 0x98a, 0x93e, 0x8f3,
        0x8aa, 0x862, 0x81b, 0x7d5, 0x790, 0x74d, 0x70a, 0x6c9, 0x689, 0x64a, 0x60c, 0x5d0, 0x594,
        0x55a, 0x521, 0x4e9, 0x4b3, 0x47d, 0x449, 0x416, 0x3e4, 0x3b4, 0x384, 0x356, 0x329, 0x2fd,
        0x2d2, 0x2a9, 0x281, 0x25a, 0x234, 0x20f, 0x1ec, 0x1ca, 0x1a9, 0x189, 0x16b, 0x14e, 0x131,
        0x117, 0xfd, 0xe5, 0xce, 0xb8, 0xa3, 0x90, 0x7d, 0x6c, 0x5d, 0x4e, 0x41, 0x35, 0x2a, 0x20,
        0x18, 0x11, 0xb, 0x7, 0x3, 0x1, 0x0, 0x0, 0x2, 0x5, 0x9, 0xe, 0x14, 0x1c, 0x25, 0x2f, 0x3b,
        0x47, 0x55, 0x64, 0x75, 0x86, 0x99, 0xad, 0xc3, 0xd9, 0xf1, 0x10a, 0x124, 0x13f, 0x15c,
        0x17a, 0x199, 0x1b9, 0x1db, 0x1fe, 0x221, 0x247, 0x26d, 0x295, 0x2bd, 0x2e8, 0x313, 0x33f,
        0x36d, 0x39c, 0x3cc, 0x3fd, 0x42f, 0x463, 0x498, 0x4ce, 0x505, 0x53e, 0x577, 0x5b2, 0x5ee,
        0x62b, 0x669, 0x6a9, 0x6e9, 0x72b, 0x76e, 0x7b2, 0x7f8, 0x83e, 0x886, 0x8cf, 0x919, 0x964,
        0x9b0, 0x9fd, 0xa4c, 0xa9b, 0xaec, 0xb3e, 0xb91, 0xbe5, 0xc3b, 0xc91, 0xce9, 0xd41, 0xd9b,
        0xdf6, 0xe52, 0xeaf, 0xf0d, 0xf6c, 0xfcc, 0x102e, 0x1090, 0x10f4, 0x1158, 0x11be, 0x1225,
        0x128d, 0x12f6, 0x1360, 0x13cb, 0x1437, 0x14a4, 0x1512, 0x1581, 0x15f1, 0x1662, 0x16d4,
        0x1748, 0x17bc, 0x1831, 0x18a7, 0x191f, 0x1997, 0x1a10, 0x1a8a, 0x1b05, 0x1b82, 0x1bff,
        0x1c7d, 0x1cfc, 0x1d7c, 0x1dfd, 0x1e7f, 0x1f02, 0x1f85, 0x200a, 0x2090, 0x2116, 0x219e,
        0x2226, 0x22b0, 0x233a, 0x23c5, 0x2451, 0x24de, 0x256c, 0x25fa, 0x268a, 0x271a, 0x27ab,
        0x283d, 0x28d0, 0x2964, 0x29f9, 0x2a8e, 0x2b24, 0x2bbc, 0x2c53, 0x2cec, 0x2d86, 0x2e20,
        0x2ebb, 0x2f57, 0x2ff4, 0x3091, 0x312f, 0x31ce, 0x326e, 0x330e, 0x33b0, 0x3451, 0x34f4,
        0x3598, 0x363c, 0x36e0, 0x3786, 0x382c, 0x38d3, 0x397b, 0x3a23, 0x3acc, 0x3b76, 0x3c20,
        0x3ccb, 0x3d77, 0x3e23, 0x3ed0, 0x3f7d, 0x402b, 0x40da, 0x4189, 0x4239, 0x42ea, 0x439b,
        0x444d, 0x44ff, 0x45b2, 0x4666, 0x471a, 0x47ce, 0x4883, 0x4939, 0x49ef, 0x4aa6, 0x4b5d,
        0x4c15, 0x4ccd, 0x4d85, 0x4e3f, 0x4ef8, 0x4fb2, 0x506d, 0x5128, 0x51e4, 0x52a0, 0x535c,
        0x5419, 0x54d6, 0x5594, 0x5652, 0x5710, 0x57cf, 0x588f, 0x594e, 0x5a0e, 0x5acf, 0x5b8f,
        0x5c50, 0x5d12, 0x5dd4, 0x5e96, 0x5f58, 0x601b, 0x60de, 0x61a1, 0x6265, 0x6329, 0x63ed,
        0x64b2, 0x6576, 0x663b, 0x6701, 0x67c6, 0x688c, 0x6952, 0x6a18, 0x6ade, 0x6ba5, 0x6c6c,
        0x6d33, 0x6dfa, 0x6ec1, 0x6f89, 0x7051, 0x7118, 0x71e0, 0x72a8, 0x7371, 0x7439, 0x7501,
        0x75ca, 0x7693, 0x775b, 0x7824, 0x78ed, 0x79b6, 0x7a7f, 0x7b48, 0x7c11, 0x7cdb, 0x7da4,
        0x7e6d, 0x7f36, 0x8000,
    ];
    TABLE[angle as usize]
}
