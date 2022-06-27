#![no_std]
#![no_main]

extern crate interface;

#[no_mangle]
pub fn main() {


    let ptr: *mut i32 = unsafe{core::mem::transmute(1)};
    interface::println!("{:80x}", unsafe{ptr.read_unaligned()} as isize);

	use crate::{platform::Interface};

    let interface = Interface::new();

    let mut tetris = tetris::Tetris::init(interface);


    loop{
        if tetris.run_frame(){
            tetris.interface.sleep_delta_mills(17);

        }else{
            break;
        }
    }
    // loop{

    // }
}


#[panic_handler]
#[no_mangle]
fn panic(info: &core::panic::PanicInfo) -> ! {
    interface::println!("{}", info);
    interface::println!("STOPPING");
    interface::sys::halt()
}

pub mod tetris;

trait InterfaceTrait{
    fn new() -> Self;
    fn initialize_screen(&mut self, height: u32, width: u32);
    fn update_screen(&mut self);
    fn set_pixel(&mut self, x: usize, y: usize, color: util::Color);
    fn clear_screen(&mut self, color: util::Color);
    fn key_down(&mut self, key: char) -> bool;
    fn cpu_usage(&mut self) -> u32;
    fn sleep_delta_mills(&mut self, millies: u32);
    fn time_micros(&mut self) -> u64;
}

pub mod platform{
    use crate::InterfaceTrait;

    pub struct Interface{
        cpu_time_start: Option<u64>,
        index: usize,
        cpu_usage: [u32; 64],
    }
    impl InterfaceTrait for Interface{
        fn update_screen(&mut self){
            interface::sys::update_screen()
        }
        fn set_pixel(&mut self, x: usize, y: usize, color: crate::util::Color){
			interface::sys::set_pixel_coords(x as u32, y as u32, color.into());
        }
        fn clear_screen(&mut self, color: crate::util::Color){
			interface::sys::fill_screen(color.into());
        }
        fn key_down(&mut self, key: char) -> bool{
           interface::sys::is_key_pressed(key)
        }
        fn initialize_screen(&mut self, height: u32, width: u32) {
            interface::sys::init_screen(width, height)    
        }
        fn cpu_usage(&mut self) -> u32{
            //interface::println!("{:?}", self.stuff2);
            let mut sum = 0;
            for item in self.cpu_usage{
                sum += item;
            }
            (sum + 1) / self.cpu_usage.len() as u32
        }
        fn sleep_delta_mills(&mut self, millies: u32) {
            let start = interface::sys::get_micros();
            interface::sys::sleep_delta_mills(millies);
            let end = interface::sys::get_micros();


            let diff = (end.wrapping_sub(start) as i64).abs() as u32;
            if let Option::Some(start_stuff) = self.cpu_time_start{
                let sleep_time = diff;
                let cpu_time = (start.wrapping_sub(start_stuff) as i32).abs() as u32;
                self.cpu_usage[self.index] = (((cpu_time as u64) * 10000) / (sleep_time as u64 + cpu_time as u64)) as u32;
                self.index += 1;
                self.index = self.index % 64;
            }
            self.cpu_time_start = Option::Some(end);
        }
        fn new() -> Self {
            Self { cpu_time_start: Option::None, index: 0, cpu_usage: [0; 64] }   
        }

        fn time_micros(&mut self) -> u64 {
            interface::sys::get_micros()
        }
    }
}

mod util{

    #[allow(dead_code)]
    #[derive(Copy, Clone)]
    pub struct Color([u8; 4]);

    impl Color{
        #[inline(always)]
        pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
            Self([r, g, b, 255])
        }
    
        #[inline(always)]
        pub const fn from_rgb_additive(r: u8, g: u8, b: u8) -> Self {
            Self([r, g, b, 0])
        }

        pub fn get_inner(&self) -> &[u8; 4]{
            &self.0
        }

        //for some reason when this is inlined the LTO breaks this code
        //no idea why but thats why its never
        #[inline(never)]
        pub fn is_opaque(&self) -> bool {
            self.0[3] == 255

        }
    }

	impl From<Color> for u32{
    fn from(color: Color) -> Self {
		((color.0[0] as u32) | ((color.0[1] as u32) << 8) | ((color.0[2] as u32) << 16))
    }
}
}