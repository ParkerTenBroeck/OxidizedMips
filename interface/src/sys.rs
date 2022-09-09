
use core::{arch::asm};

pub mod external_screen{

    #[repr(C)]
    pub struct ScreenData<L1: LayerImpl = BitMapExpand, L2: LayerImpl = Dissabled, L3: LayerImpl = Dissabled, L4: LayerImpl = Dissabled>{
        pub layer1: LayerWrapper<L1>,
        pub layer2: LayerWrapper<L2>,
        pub layer3: LayerWrapper<L3>,
        pub layer4: LayerWrapper<L4>,
    }

    impl<L1: LayerImpl, L2: LayerImpl, L3: LayerImpl, L4: LayerImpl> Default for ScreenData<L1, L2, L3, L4> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<L1: LayerImpl, L2: LayerImpl, L3: LayerImpl, L4: LayerImpl> ScreenData<L1, L2, L3, L4>{
        pub fn new() -> Self{
            Self { 
                layer1: L1::create_wrapper(), 
                layer2: L2::create_wrapper(), 
                layer3: L3::create_wrapper(), 
                layer4: L4::create_wrapper() 
            }
        }
    }

    #[repr(C)]
    pub struct LayerWrapper<L: LayerImpl>{
        layer_id: u32,
        pub layer_data: L
    }

    pub trait LayerImpl{
        fn new_layer() -> Self;
        fn layer_id() -> u32;
    }
    trait LayerWrapperImpl<T: LayerImpl + Sized>{
        fn create_wrapper() -> LayerWrapper<T>;
    }
    impl<T: LayerImpl + Sized> LayerWrapperImpl<T> for T{
        fn create_wrapper() -> LayerWrapper<T> {
            LayerWrapper { 
                layer_id: T::layer_id(), 
                layer_data: T::new_layer() 
            }
        }
    }

    //--------------------------------------------------------------------------------------------------------
    pub struct Dissabled{

    }
    impl LayerImpl for Dissabled{
        fn new_layer() -> Self {
            Self {  }
        }

        fn layer_id() -> u32 {
            0
        }
    }
    //--------------------------------------------------------------------------------------------------------


    //--------------------------------------------------------------------------------------------------------
    #[repr(C)]
    pub struct BitMapExpand{
        pub size: [i16; 2],
        pub ptr: u32,
    }
    impl LayerImpl for BitMapExpand{
        fn new_layer() -> Self {
            Self {
                ptr: 0,
                size: [0,0], 
            }
        }

        fn layer_id() -> u32 { 1 }
    }
    //--------------------------------------------------------------------------------------------------------


    //--------------------------------------------------------------------------------------------------------
    #[repr(C)]
    pub struct BitMapScroll{
        pub bmp_ptr: u32,
        pub bmp_size: [i16; 2],
        pub visible_size: [i16; 2],
        pub scroll: [i16; 2],
    }
    impl LayerImpl for BitMapScroll{
        fn new_layer() -> Self {
            Self {
                bmp_ptr: 0,
                bmp_size: [0,0],
                visible_size: [0,0],
                scroll: [0,0],
            }
        }

        fn layer_id() -> u32 { 2 }
    }
    //--------------------------------------------------------------------------------------------------------

    //--------------------------------------------------------------------------------------------------------
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct Tile{
        pub index_rot: u32,
        pub tint: [u8; 4],
    }
    
    #[repr(C)]
    pub struct TileMap<const W: i16, const H: i16> where [(); W as usize * H as usize] :{
        pub tile_map_ptr: u32,
        pub visible_size: [i16; 2],
        pub scroll: [i16; 2],
        tile_map_size: [i16; 2],
        pub tiles: [Tile; W as usize * H as usize]
    }
    impl<const W: i16, const H: i16> LayerImpl for TileMap<W,H>  where [(); W as usize * H as usize] :{
        fn new_layer() -> Self {
            Self { 
                tile_map_ptr: 0, 
                tile_map_size: [W, H], 
                visible_size: [0,0], 
                scroll: [0,0], 
                tiles: [
                    Tile{ 
                        index_rot: 0, 
                        tint: [0,0,0,0] 
                    }; W as usize * H as usize
                    ]
            }
        }

        fn layer_id() -> u32 { 3 }
    }
    //--------------------------------------------------------------------------------------------------------

    //--------------------------------------------------------------------------------------------------------
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct Sprite{
        pub sp_pos: [i16; 2],
        pub screen_pos: [i16; 2],
        pub tint: [u8; 4],
        pub x_size: u16,
        pub y_size: u16,
        pub rot: i16,
    }
    
    #[repr(C)]
    pub struct Sprites<const S: i16> where [(); S as usize] :{
        pub sprite_sheet_ptr: u32,
        pub sprite_sheet_size: [i16; 2],
        pub resolition: [i16; 2],
        pub scroll: [i16; 2],
        num_sprites: i16,
        sprites: [Sprite; S as usize]
    }
    impl<const S: i16> LayerImpl for Sprites<S> where [(); S as usize] :{
        fn new_layer() -> Self {
            Self {
                sprite_sheet_ptr: 0,
                sprite_sheet_size: [0,0],
                resolition: [0,0],
                scroll: [0,0],
                num_sprites: S,
                sprites: [
                    Sprite{
                        sp_pos: [0,0],
                        screen_pos: [0,0],
                        tint: [0,0,0,0],
                        x_size: 0,
                        y_size: 0,
                        rot: 0,
                    }; S as usize
                ],
            }
        }

        fn layer_id() -> u32 { 4 }
    }
    //--------------------------------------------------------------------------------------------------------
}

#[inline(always)]
pub fn halt() -> !{
    unsafe {syscall_0_0::<0>(); }
    
    unsafe{
        core::hint::unreachable_unchecked();
    }
}

#[inline(always)]
pub fn print_i32(num: i32){
    unsafe{ syscall_1_0::<1>(num as u32); }
}

#[inline(always)]
pub fn print_zero_term_str(str: &str){
    unsafe{ syscall_1_0::<4>(str.as_ptr().addr() as u32); }
}

#[inline(always)]
pub fn print_str(str: &str){
    for char in str.chars(){
        print_char(char);
    }
}

#[inline(always)]
pub fn read_i32() -> i32{
    unsafe{ syscall_0_1::<5>() as i32 }
}

pub fn rand_range(min: i32, max: i32) -> i32{
    unsafe{ syscall_2_1::<99>(min as u32, max as u32) as i32 }
} 

#[inline(always)]
pub fn print_char(char: char){
    unsafe{ syscall_1_0::<101>(char as u32); }
}

pub fn is_key_pressed(key: char) -> bool{
    unsafe{ syscall_1_1::<104>(key as u32 as u32) == 1 }
}

#[inline(always)]
pub fn sleep_delta_mills(mills: u32){
    unsafe{ syscall_1_0::<106>(mills); }
}

#[inline(always)]
pub fn sleep_mills(mills: u32){
    unsafe{ syscall_1_0::<105>(mills); }
}

#[inline(always)]
pub fn get_micros() -> u64{
    unsafe{ syscall_0_2_s::<108>() }
}
#[inline(always)]
pub fn get_nanos() -> u64{
    unsafe{ syscall_0_2_s::<109>() }
}


#[inline(always)]
pub fn old_breakpoint(){
    unsafe{ syscall_0_0::<111>(); }
}

#[inline(always)]
pub fn init_screen(width: u32, height: u32){
    unsafe{ syscall_2_0::<150>(width, height); }
}

#[inline(always)]
pub fn set_pixel_coords(x: u32, y: u32, color: u32){
    unsafe{ syscall_3_0::<151>(x, y, color); }
}

#[inline(always)]
pub fn set_pixel_index(index: u32, color: u32){
    unsafe{ syscall_2_0::<152>(index, color); }
}

#[inline(always)]
pub fn update_screen(){
    unsafe{ syscall_0_0::<153>(); }
}

#[inline(always)]
pub fn update_screen_vsync(){
    unsafe{ syscall_0_0::<154>(); }
}

#[inline(always)]
pub fn hsv_to_rgb(hsv: u32) -> u32{
    unsafe{ syscall_1_1::<155>(hsv) }
}

#[inline(always)]
pub fn fill_screen(color: u32){
    unsafe {syscall_1_0::<156>(color); }
}

#[inline(always)]
pub unsafe fn syscall_0_0<const CALL_ID: u32>(){
    asm!(
        "syscall {0}",
        const(CALL_ID),
    )
}

#[inline(always)]
pub unsafe fn syscall_1_0<const CALL_ID: u32>(arg1: u32){
    asm!(
        "syscall {0}",
        const(CALL_ID),
        in("$4") arg1,
    )
}

#[inline(always)]
pub unsafe fn syscall_0_1<const CALL_ID: u32>() -> u32{
    let ret1;
    asm!(
        "syscall {0}",
        const(CALL_ID),
        out("$2") ret1,
    );
    ret1
}

#[inline(always)]
pub unsafe fn syscall_1_1<const CALL_ID: u32>(arg1: u32) -> u32{
    let ret1;
    asm!(
        "syscall {0}",
        const(CALL_ID),
        in("$4") arg1,
        out("$2") ret1,
    );
    ret1
}

#[inline(always)]
pub unsafe fn syscall_2_0<const CALL_ID: u32>(arg1: u32, arg2: u32){
        asm!(
            "syscall {0}",
            const(CALL_ID),
            in("$4") arg1,
            in("$5") arg2,
        );
}

#[inline(always)]
pub unsafe fn syscall_3_0<const CALL_ID: u32>(arg1: u32, arg2: u32, arg3: u32){
    asm!(
        "syscall {0}",
        const(CALL_ID),
        in("$4") arg1,
        in("$5") arg2,
        in("$6") arg3,
    );
}

#[inline(always)]
pub unsafe fn syscall_2_1<const CALL_ID: u32>(arg1: u32, arg2: u32) -> u32{
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

#[inline(always)]
pub unsafe fn syscall_0_2_s<const CALL_ID: u32>() -> u64{
    let tmp1: u32;
    let tmp2: u32;
    asm!(
        "syscall {0}",
        const(CALL_ID),
        out("$2") tmp1,
        out("$3") tmp2,
    );
    (tmp1 as u64) | ((tmp2 as u64) << 32)
}