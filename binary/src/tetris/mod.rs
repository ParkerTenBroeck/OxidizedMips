use crate::util::display::Color;

pub mod tetris;



pub fn run_tetris(){
    use crate::tetris::{platform::Interface};

    let interface = Interface::new();

    let mut tetris = tetris::Tetris::init(interface);


    loop{
        if tetris.run_frame(){
            tetris.interface.update_screen();
        }else{
            break;
        }
    }
}
trait InterfaceTrait{
    fn new() -> Self;
    fn initialize_screen(&mut self, height: u32, width: u32);
    fn update_screen(&mut self);
    fn set_pixel(&mut self, x: usize, y: usize, color: Color);
    fn clear_screen(&mut self, color: Color);
    fn key_down(&mut self, key: char) -> bool;
    fn cpu_usage(&mut self) -> u32;
    fn fps(&mut self) -> u32;
    fn time_micros(&mut self) -> u64;
}

pub mod platform{
    use crate::util::display::Color;

    use super::InterfaceTrait;

    pub struct Interface{
        cpu_time_start: Option<u64>,
        index: usize,
        cpu_usage: [u32; 64],
        fps: [u32; 64],
    }
    impl InterfaceTrait for Interface{
        fn update_screen(&mut self){

            let start = interface::sys::get_micros();

            if self.key_down('e'){
                interface::sys::update_screen()
            }else{
                interface::sys::update_screen_vsync();
            }

            let end = interface::sys::get_micros();


            let diff = (end.wrapping_sub(start) as i64).abs() as u32;
            if let Option::Some(start_stuff) = self.cpu_time_start{
                let sleep_time = diff;
                let cpu_time = (start.wrapping_sub(start_stuff) as i32).abs() as u32;
                self.cpu_usage[self.index] = (((cpu_time as u64) * 10000) / (sleep_time as u64 + cpu_time as u64)) as u32;
                self.fps[self.index] = sleep_time + cpu_time;
                self.index += 1;
                self.index = self.index % 64;
            }
            self.cpu_time_start = Option::Some(end);
        }
        fn set_pixel(&mut self, x: usize, y: usize, color: Color){
			interface::sys::set_pixel_coords(x as u32, y as u32, color.into());
        }
        fn clear_screen(&mut self, color: Color){
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
        fn new() -> Self {
            Self { cpu_time_start: Option::None, index: 0, cpu_usage: [0; 64], fps: [0; 64] }   
        }

        fn time_micros(&mut self) -> u64 {
            interface::sys::get_micros()
        }

        fn fps(&mut self) -> u32 {
            let mut sum = 0;
            for item in self.fps{
                sum += item;
            }
            10000000u32.checked_div((sum) / self.fps.len() as u32).unwrap_or_default()
        }
    }
}