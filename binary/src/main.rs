#![no_std]
#![no_main]

extern crate interface;

#[no_mangle]
pub fn main() {


    interface::println!("Hello World");

	interface::sys::init_screen(48, 32);

	for y in 0..32{
		for x in 0..48{
			interface::sys::set_pixel_coords(x, y, x * y);
		}
	}

	interface::sys::update_screen();
	if interface::sys::rand_range(0, 3) == 1{
		interface::sys::print_isize(interface::sys::rand_range(0, 1));
		panic!("oh no we guessed 1 PANICING AAAAAAA: {}", interface::sys::rand_range(0, 1))
	}else{
        if true{
            panic!("asdlkjasdlkjasd");
        }
    }

	use crate::{platform::Interface};

    let interface = Interface{
    };

    let mut tetris = tetris::Tetris::init(interface);


    loop{
        if tetris.run_frame(){
            interface::sys::sleep_delta_mills(17);
        }else{
            break;
        }
    }
    loop{

    }
}

pub mod tetris;

trait InterfaceTrait{
    fn update_screen(&mut self);
    fn set_pixel(&mut self, x: usize, y: usize, color: util::Color);
    fn clear_screen(&mut self, color: util::Color);
    fn key_down(&mut self, key: char) -> bool;
}

pub mod platform{
    use crate::InterfaceTrait;

    pub struct Interface{
    }
    impl InterfaceTrait for Interface{
        fn update_screen(&mut self){
            interface::sys::update_screen()
        }
        fn set_pixel(&mut self, x: usize, y: usize, color: crate::util::Color){
			interface::sys::set_pixel_coords(x, y, color.into());
        }
        fn clear_screen(&mut self, color: crate::util::Color){
			interface::sys::fill_screen(color.into());
        }
        fn key_down(&mut self, key: char) -> bool{
           interface::sys::is_key_pressed(key)
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

        pub fn is_opaque(&self) -> bool {
            self.0[3] == 255
        }
    }

	impl From<Color> for usize{
    fn from(color: Color) -> Self {
		((color.0[0] as u32) | ((color.0[1] as u32) << 8) | ((color.0[2] as u32) << 16)) as usize
    }
}
}