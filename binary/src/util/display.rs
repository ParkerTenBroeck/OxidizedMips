#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct Color([u8; 4]);

impl Color {
    #[inline(always)]
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b, 255])
    }
    #[inline(always)]
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }

    #[inline(always)]
    pub const fn from_rgb_additive(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b, 0])
    }

    #[inline(always)]
    pub const fn clear() -> Self {
        Self([0, 0, 0, 0])
    }

    pub fn get_inner(&self) -> &[u8; 4] {
        &self.0
    }

    #[inline(always)]
    pub fn is_opaque(&self) -> bool {
        self.0[3] == 255
    }
    #[inline(always)]
    pub fn get_alpha(&self) -> u8 {
        self.0[3]
    }
}

impl From<Color> for u32 {
    fn from(color: Color) -> Self {
        (color.0[0] as u32) | ((color.0[1] as u32) << 8) | ((color.0[2] as u32) << 16)
    }
}

impl From<u32> for Color {
    fn from(color: u32) -> Self {
        Color::from_rgb(
            (color & 255) as u8,
            ((color >> 8) & 255) as u8,
            ((color >> 16) & 255) as u8,
        )
    }
}

pub fn display_number(
    mut num: u32,
    location: impl Into<[u32; 2]>,
    leading_zeros: u32,
    forground: Color,
    background: Color,
) -> usize {
    let mut location = location.into();
    let mut iters = 0;
    let mut num_nums = 0;
    while num > 0 {
        let n = num % 10;
        num /= 10;
        //self. fill_cube(location, forground);
        draw_tiled_character(location, n + 16, forground, background);
        iters += 1;
        location[0] -= 1;
        num_nums += 1;
    }
    for _ in iters..leading_zeros {
        draw_tiled_character(location, 16, forground, background);
        location[0] -= 1;
        num_nums += 1;
    }
    num_nums
}

pub fn display_percentage<const D: u32>(
    top: u32,
    bottom: u32,
    pos: impl Into<[u32; 2]>,
    forground: Color,
    background: Color,
) {
    let mut pos = pos.into();
    const fn pow(item: u32) -> u32 {
        if item == 1 {
            10
        } else {
            pow(item - 1) * 10
        }
    }

    let usage = (top * pow(D) * 100) / bottom;
    draw_tiled_character(pos, 5, forground, background);
    pos[0] += 2;
    if usage >= pow(D + 2) {
        pos[0] += 1;
    }
    display_number(usage / pow(D), pos, 2, forground, background);
    pos[0] += 1;
    draw_tiled_character(pos, 14, forground, background);
    pos[0] += D as u32;

    display_number(usage % pow(D), pos, D, forground, background);
}

pub fn draw_string(
    string: &str,
    location: impl Into<[u32; 2]>,
    forground: Color,
    background: Color,
) {
    let mut location = location.into();
    for char in string.as_bytes().iter() {
        let char = *char as char;
        let index = match char {
            'a'..='z' => ((char as u32) - 'a' as u32) + 65,
            'A'..='Z' => ((char as u32) - 'A' as u32) + 33,
            '0'..='9' => char as u32 - '0' as u32 + 16,
            ' ' => 0,
            '#' => 95, //3,
            '/' => 15,
            _ => {
                panic!();
            }
        };
        if index != 0 {
            draw_tiled_character(location, index, forground, background);
        }
        location[0] += 1;
    }
}

const CHACATER_SET: &[u8; 768] = include_bytes!("../../res/character-tile-set.comp");

pub fn draw_tiled_character(
    location: impl Into<[u32; 2]>,
    char: u32,
    forground: Color,
    background: Color,
) {
    let mut location = location.into();
    location[1] *= 8;
    location[0] *= 8;
    draw_chacater(location, char, forground, background);
}

pub fn draw_chacater(
    location: impl Into<[u32; 2]>,
    char: u32,
    forground: Color,
    background: Color,
) {
    let location = location.into();
    let char = char << 3;
    for y in 0..8 {
        let char = unsafe { *CHACATER_SET.get_unchecked((y + char) as usize) };

        for x in 0..8 {
            //unsafe { interface::sys::syscall_0_0::<0xFE>();}

            let color = if char & (1 << x) > 0 {
                // if forground.is_opaque(){
                //     //interface::sys::old_breakpoint();
                //     //unsafe { interface::sys::syscall_0_0::<0xFF>();}
                //     interface::sys::set_pixel_coords(x + (location[0]), (y as u32) + (location[1]), forground.into());
                // }
                forground
            } else {
                // if background.is_opaque(){
                //     //interface::sys::old_breakpoint();
                //     //unsafe { interface::sys::syscall_0_0::<0xFF>();}
                //     interface::sys::set_pixel_coords(x + (location[0]), (y as u32) + (location[1]), background.into());
                // }
                background
            };
            //interface::sys::old_breakpoint();
            if color.is_opaque() {
                //interface::sys::old_breakpoint();
                //unsafe { interface::sys::syscall_1_0::<0xFF>(color.into());}
                interface::sys::set_pixel_coords(
                    x + (location[0]),
                    (y as u32) + (location[1]),
                    color.into(),
                );
            }
        }
    }
}

pub fn draw_chacater_x2(location: [u32; 2], char: u32, forground: Color, background: Color) {
    let char = char << 3;
    for y in 0..8 {
        let char = unsafe { *CHACATER_SET.get_unchecked(y + char as usize) };

        macro_rules! x_thing {
            ($x:expr) => {
                let color;
                if char & (1 << $x) > 0 {
                    color = forground;
                } else {
                    color = background;
                }
                if color.is_opaque() {
                    interface::sys::set_pixel_coords(
                        ($x << 1) + (location[0]),
                        ((y as u32) << 1) + (location[1]),
                        color.into(),
                    );
                    interface::sys::set_pixel_coords(
                        ($x << 1) + 1 + (location[0]),
                        ((y as u32) << 1) + (location[1]),
                        color.into(),
                    );
                    interface::sys::set_pixel_coords(
                        ($x << 1) + (location[0]),
                        1 + ((y as u32) << 1) + (location[1]),
                        color.into(),
                    );
                    interface::sys::set_pixel_coords(
                        ($x << 1) + 1 + (location[0]),
                        1 + ((y as u32) << 1) + (location[1]),
                        color.into(),
                    );
                }
            };
        }
        x_thing!(0);
        x_thing!(1);
        x_thing!(2);
        x_thing!(3);
        x_thing!(4);
        x_thing!(5);
        x_thing!(6);
        x_thing!(7);
    }
}
