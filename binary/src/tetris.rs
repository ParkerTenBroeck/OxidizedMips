use self::{game::TetrisGame, renderer::TetrisRenderer, input::TetrisInput, sound::TetrisSound};

#[allow(unused)]
pub struct Tetris{
    renderer: TetrisRenderer,
    game: TetrisGame,
    input: TetrisInput,
    sound: TetrisSound,
    frame_counter: usize,
    interface: crate::platform::Interface
}

impl Tetris{

    #[inline(always)]
    pub fn rand_num(&self, min: usize, max: usize) -> usize{
        let mut x = self.frame_counter;
        x = ((x >> 16) ^ x).wrapping_mul(0x45d9f3b_usize);
        x = ((x >> 16) ^ x).wrapping_mul(0x45d9f3b_usize);
        x = (x >> 16) ^ x;
        let x = (x >> 1) as usize;

        let dif = (max + 1).wrapping_sub(min);
        (x % dif) + min
    }
    
    pub fn init(interface: crate::platform::Interface) -> Self{
        
        Tetris {
            interface,
            renderer: TetrisRenderer::init(),
            game: TetrisGame::init(),
            input: TetrisInput::init(),
            sound: TetrisSound::init(),
            frame_counter: 0,
        }
    }
    pub fn run_frame(&mut self) -> bool{
        self.update_input();
        self.update_sound();
        self.update_game();
        self.render_frame();
        self.frame_counter += 1;
        true
    }
}

pub mod util{

    pub fn factorial(mut num: usize) -> usize{
        if num <= 1{
            return 1;
        }
        for i in 2..num{
            num *= i
        }
        num
    }

    pub fn permutate<T>(items: &mut [T], mut permutation: usize){
        debug_assert!(permutation < factorial(items.len()));
        debug_assert!(items.len() >= 2);

        let len = items.len();
        let mut i = 0;
        //len to 2
        for n in (2..=len).rev(){
            let fact = factorial(n - 1);
            let t = permutation / fact;
            permutation -= t*fact;
            for i in (i..i+t).rev(){
                items.swap(i, i+1);
            }
            i += 1;
        }
    }

    pub fn rank_permutation<const S: usize, T: Eq>(items: &[T; S], base: &[T; S]) -> usize{
        debug_assert!(items.len() == base.len());

        let mut base_c: core::mem::MaybeUninit<[&T; S]> = core::mem::MaybeUninit::uninit();
        for i in 0..S{
            unsafe{
                let ptr = base_c.as_mut_ptr();
                (*ptr)[i] = &base[i];
            }
        }
        let mut base_c = unsafe{base_c.assume_init()};
        
        let mut result = 0;
        for i in 0..items.len()-1{
            let base_index = base_c.iter().position(|item|{ **item == items[i]}).unwrap();
            //base.remove(base_index);
            for i in base_index..items.len()-1{
                base_c.swap(i, i+1);
            }
            result += base_index * factorial(3-i);    
        }
        result
    }
}

mod game{
    use super::Tetris;

    const GRAVITY_TABLE: [u8; 30] = [48,43,38,33,28,23,18,13,8,6,5,5,5,4,4,4,3,3,3,2,2,2,2,2,2,2,2,2,2,1];

    pub struct TetrisGame{
        pub piece: Option<FallingPiece>,
        pub netx_piece: Option<Tetrominoes>,
        pub board: Board,
        pub score: usize,
        pub level: usize,
        pub lines_cleared: usize,
        pub combo_count: usize,
        pub piece_stats: [usize; 8]
    }

    impl TetrisGame{
        pub fn get_curr_piece(&self) -> Option<(u8, [Coord; 4])>{
            match &self.piece{
                Some(val) => {
                    Option::Some((val.piece_type.as_num(), val.get_coords()))
                },
                None => Option::None,
            }
        }

        pub fn get_dropped_piece(&self) -> Option<(u8, [Coord; 4], u8)>{
            match &self.piece{
                Some(val) => {
                    let mut coords = val.get_coords();
                    let mut distance = 0u8;
                    'main_loop:
                    loop{
                        for coord in coords{
                            if self.board.is_intersecting(coord + [0,1i16].into()){
                                break 'main_loop;
                            }
                        }
                        for coord in &mut coords{
                            *coord = *coord + [0,1i16].into();
                        }
                        distance += 1;
                    }
                
                    Option::Some((val.piece_type.as_num(), coords, distance))
                },
                None => Option::None,
            }
        }
    }
    #[derive(Clone, Copy)]
    pub struct FallingPiece{
        piece_type: Tetrominoes,
        frames_since_last_fall: usize,
        rotation: u8,
        coords: Coord,
    }
    impl FallingPiece{
        pub fn get_coords(&self) -> [Coord; 4]{
            let t = self.piece_type.get_coords(self.rotation);
            let mut coords = [[0,0i16].into(); 4];
            for i in 0..4{
                coords[i] = self.coords + t[i].into();
            }
            coords
        }
    }

    pub struct Board{
        data: [u32; 40]
    }

    impl Board{
        fn new() -> Self{

            Self { 
                data: [0; 40]
            }
        }
        fn is_intersecting(&self, coord: Coord) -> bool{
            if coord.x < 0 || coord.x >= 10{
                return true
            }
            if coord.y < 0 || coord.y >= 40{
                return true
            }
            self.is_full(coord)
        }
        fn is_any_intersecting(&self, coords: &[Coord]) -> bool{
            for coord in coords{
                if self.is_intersecting(*coord){
                    return true;
                }
            }
            return false
        }
        #[inline(always)]
        fn is_empty(&self, coord: Coord) -> bool{
            self.data_at_coord(coord) == 0
        }
        #[inline(always)]
        fn is_full(&self, coord: Coord) -> bool{
            !self.is_empty(coord)
        }
        #[inline(always)]
        pub fn data_at_coord(&self, coord: Coord) -> u8{
            ((self.data[coord.y as usize] >> (coord.x * 3)) & 7) as u8
        }
        #[inline(always)]
        fn set_data_at_coord(&mut self, data: u8, coord: Coord){
            let y = &mut self.data[coord.y as usize];
            *y = (*y & (!(7 << (coord.x as u32 * 3)))) | ((data as u32 & 7) << (coord.x * 3))
        }
    }

    impl TetrisGame{
        pub fn init() -> Self{

            Self { 
                piece: Option::None,
                netx_piece: Option::None,  
                board: Board::new(),
                score: 0,
                level: 0,
                lines_cleared: 0,
                combo_count: 0,
                piece_stats: [0; 8],
            }
        }
    }

    #[derive(Default, Copy, Clone)]
    pub struct Coord{
        pub x: i16,
        pub y: i16
    }

    impl Coord{
        pub fn scale(&self, scaler: i16) -> Self{
            Coord{x: self.x * scaler, y: self.y * scaler}
        }
    }

    impl From<[i16; 2]> for Coord{
        fn from(coord: [i16; 2]) -> Self {
            Coord { x: coord[0], y: coord[1] }
        }
    }

    impl From<[u8; 2]> for Coord{
        fn from(coord: [u8; 2]) -> Self {
            Coord { x: coord[0] as i16, y: coord[1] as i16 }
        }
    }
    impl core::ops::Add for Coord{
        type Output = Self;
        fn add(self, rhs: Self) -> Self::Output {
            [self.x + rhs.x, self.y + rhs.y].into()
        }
    }
    impl core::ops::Sub for Coord{
        type Output = Self;
        fn sub(self, rhs: Self) -> Self::Output {
            [self.x - rhs.x, self.y - rhs.y].into()
        }
    }
    impl From<Coord> for [i16; 2]{
        fn from(coord: Coord) -> Self {
            [coord.x, coord.y]
        }
    }
    
    #[derive(Copy, Clone)]
    pub enum Tetrominoes{
        I,
        J,
        L,
        O,
        S,
        T,
        Z,
    }
    
    impl Tetrominoes{
        pub fn get_coords(&self, rotation: u8) -> [[u8; 2]; 4]{
            let rotation_matrix:[[[[u8; 2]; 4]; 8]; 4]  = [
                [
                    [[0,1],[1,1],[2,1],[3,1]],
                    [[0,0],[0,1],[1,1],[2,1]],
                    [[0,1],[1,1],[2,1],[2,0]],
                    [[0,0],[1,0],[0,1],[1,1]],
                    [[0,1],[1,1],[1,0],[2,0]],
                    [[0,1],[1,1],[2,1],[1,0]],
                    [[0,0],[1,0],[1,1],[2,1]],
                    [[0,0],[1,1],[2,2],[3,3]],//filler
                ],
                [
                    [[2,3],[2,2],[2,1],[2,0]],
                    [[2,0],[1,0],[1,1],[1,2]],
                    [[1,0],[1,1],[1,2],[2,2]],
                    [[0,0],[1,0],[0,1],[1,1]],
                    [[1,0],[1,1],[2,1],[2,2]],
                    [[1,2],[1,1],[1,0],[2,1]],
                    [[2,0],[2,1],[1,1],[1,2]],
                    [[3,0],[2,1],[1,2],[0,3]],//filler
                ],
                [
                    [[3,2],[2,2],[1,2],[0,2]],
                    [[0,1],[1,1],[2,1],[2,2]],
                    [[0,2],[0,1],[1,1],[2,1]],
                    [[0,0],[1,0],[0,1],[1,1]],
                    [[2,1],[1,1],[1,2],[0,2]],
                    [[2,1],[1,1],[0,1],[1,2]],
                    [[0,1],[1,1],[1,2],[2,2]],
                    [[0,0],[1,1],[2,2],[3,3]],//filler
                ],
                [
                    [[1,0],[1,1],[1,2],[1,3]],
                    [[1,0],[1,1],[1,2],[0,2]],
                    [[0,0],[1,0],[1,1],[1,2]],
                    [[0,0],[1,0],[0,1],[1,1]],
                    [[0,0],[0,1],[1,1],[1,2]],
                    [[1,0],[1,1],[1,2],[0,1]],
                    [[1,0],[1,1],[0,1],[0,2]],
                    [[3,0],[2,1],[1,2],[0,3]],//filler
                ],
            ];
            rotation_matrix[rotation as usize][self.as_num() as usize]
        }

        pub fn as_num(&self) -> u8{
            match self{
                Tetrominoes::I => 0,
                Tetrominoes::J => 1,
                Tetrominoes::L => 2,
                Tetrominoes::O => 3,
                Tetrominoes::S => 4,
                Tetrominoes::T => 5,
                Tetrominoes::Z => 6,
            }
        }

        pub fn from_num(num: u8) -> Tetrominoes{
            match num{
                0 => Tetrominoes::I,
                1 => Tetrominoes::J,
                2 => Tetrominoes::L,
                3 => Tetrominoes::O,
                4 => Tetrominoes::S,
                5 => Tetrominoes::T,
                6 => Tetrominoes::Z,
                _ => {panic!()}
            }
        }
    }

    impl Tetris{
        pub fn update_game(&mut self){


            let mut new = false;

            if self.input.drop_down_pressed(){
                match self.game.get_dropped_piece(){
                    Some(dropped) => {
                        for coord in dropped.1{
                            self.game.board.set_data_at_coord(dropped.0 + 1, coord);
                        }
                        self.game.score += dropped.2 as usize * 2;
                        new = true;
                    },
                    None => {

                    },
                }
            }else{

            match &mut self.game.piece{
                Some(piece) => {
                    piece.frames_since_last_fall += 1;

                    let mut grav = GRAVITY_TABLE[self.game.level.clamp(0, 29)];
                    if self.input.down_pressed(){
                        grav >>= 1;
                        self.game.score += 1;
                    }
                    if piece.frames_since_last_fall > grav as usize{
                        piece.coords = piece.coords +  [0, 1i16].into();
                        let coords = piece.get_coords();
                        if self.game.board.is_any_intersecting(&coords){
                            new = true;
                            piece.coords = piece.coords - [0, 1i16].into();
                            for coord in piece.get_coords(){
                                self.game.board.set_data_at_coord(piece.piece_type.as_num() + 1, coord);
                            }
                        }
                        
                        piece.frames_since_last_fall = 0;
                    }

                    if self.input.left_pressed(){
                        piece.coords = piece.coords - [1i16, 0].into();
                        if self.game.board.is_any_intersecting(&piece.get_coords()){
                            piece.coords = piece.coords + [1i16, 0].into();
                        }
                    }
                    if self.input.right_pressed(){
                        piece.coords = piece.coords + [1i16, 0].into();
                        if self.game.board.is_any_intersecting(&piece.get_coords()){
                            piece.coords = piece.coords - [1i16, 0].into();
                        }
                    }
                    if self.input.up_pressed(){
                        piece.rotation = (piece.rotation + 1) & 3;
                        if self.game.board.is_any_intersecting(&piece.get_coords()){
                            piece.rotation = piece.rotation.wrapping_sub(1) & 3;
                        }
                    }
                },
                None => {
                    new = true;
                },
            }
            }

            if new{
                match self.game.piece {
                    Some(piece) => {
                        self.game.piece_stats[piece.piece_type.as_num() as usize] += 1;
                        self.game.piece_stats[7] += 1;
                    },
                    None => {

                    },
                }
                self.game.piece = Option::Some(FallingPiece{
                    piece_type: Tetrominoes::from_num(self.rand_num(0, 6) as u8),
                    frames_since_last_fall: 0,
                    rotation: 0,
                    coords: [5, 22i16].into(),
                });
            }
        }

        fn drop_piece(&mut self, piece: FallingPiece){
            
        }
    }
    
}

pub mod renderer{

    pub const WIDTH: usize = (32)*8;
    pub const HEIGHT: usize = (30)*8;

    use crate::{tetris::game::Coord, InterfaceTrait, util::Color};

    use super::{Tetris};

    pub struct TetrisRenderer{

    }

    impl TetrisRenderer{
        pub fn init() -> Self {
            Self {
            }
        }   
    }

    const CHACATER_SET: &'static [u8; 768] = &[0u8; 768];//include_bytes!("../res/character-tile-set.comp");

    const TETROMINOE_PALLETE: [[Color; 5]; 8] = [
        [
            Color::from_rgb(18, 255, 255),
            Color::from_rgb(179, 255, 255),
            Color::from_rgb(0, 179, 179),
            Color::from_rgb(0, 76, 76),
            Color::from_rgb(76, 255, 255),
        ],
        [
            Color::from_rgb(0, 0, 179),
            Color::from_rgb(179, 179, 230),
            Color::from_rgb(0, 0, 119),
            Color::from_rgb(0, 0, 51),
            Color::from_rgb(76, 76, 196),
        ],
        [
            Color::from_rgb(255, 119, 0),
            Color::from_rgb(255, 214, 179),
            Color::from_rgb(179, 83, 0),
            Color::from_rgb(76, 35, 0),
            Color::from_rgb(255, 160, 76),
        ],
        [
            Color::from_rgb(255, 255, 0),
            Color::from_rgb(255, 255, 179),
            Color::from_rgb(179, 179, 0),
            Color::from_rgb(76, 76, 0),
            Color::from_rgb(255, 255, 76),
        ],
        [
            Color::from_rgb(23, 255, 0),
            Color::from_rgb(179, 255, 179),
            Color::from_rgb(13, 179, 0),
            Color::from_rgb(0, 76, 0),
            Color::from_rgb(79, 255, 79),
        ],
        [
            Color::from_rgb(204, 0, 204),
            Color::from_rgb(240, 179, 240),
            Color::from_rgb(142, 0, 142),
            Color::from_rgb(61, 0, 61),
            Color::from_rgb(219, 76, 219),
        ],
        [
            Color::from_rgb(255, 0, 0),
            Color::from_rgb(255, 179, 179),
            Color::from_rgb(179, 0, 0),
            Color::from_rgb(76, 0, 0),
            Color::from_rgb(255, 76, 76),
        ],
        [
            Color::from_rgb(119, 119, 119),
            Color::from_rgb(219, 219, 219),
            Color::from_rgb(82, 82, 82),
            Color::from_rgb(34, 34, 34),
            Color::from_rgb(160, 160, 160),
        ],
    ];

    impl Tetris{
        pub fn render_frame(&mut self){

            self.interface.clear_screen(Color::from_rgb(0, 0, 0));

            for x in 0..12{
                self.draw_cube([x,0i16].into(), &TETROMINOE_PALLETE[7]);
            }
            for x in 0..12{
                self.draw_cube([x,21i16].into(), &TETROMINOE_PALLETE[7]);
            }
            for y in 1..21{
                self.draw_cube([0i16,y].into(), &TETROMINOE_PALLETE[7]);
            }
            for y in 1..21{
                self.draw_cube([11i16,y].into(), &TETROMINOE_PALLETE[7]);
            }

            for x in 0u8..10{
                for y in 0u8..20{
                    let data = self.game.board.data_at_coord([x,y + 20].into());
                    if data == 0{
                        self.fill_cube([x+1,y+1].into(), Color::from_rgb(0, 0, 0));
                    }else{
                        self.draw_cube([x+1, y+1].into(), &TETROMINOE_PALLETE[data as usize - 1]);
                    }
                }
            }

            match self.game.get_dropped_piece(){
                Some(piece) => {
                    for coord in piece.1{
                        self.ghost_cube(coord - [-1,19i16].into(), TETROMINOE_PALLETE[piece.0 as usize][0])
                    }
                },
                None => {},
            }

            match self.game.get_curr_piece(){
                Some(piece) => {
                    for coord in piece.1{
                        self.draw_cube(coord - [-1,19i16].into(), &TETROMINOE_PALLETE[piece.0 as usize])
                    }
                },
                None => {},
            }
            
            for i in 0..8{
                self.display_number(self.game.piece_stats[i], [15i16, i as i16].into(), 3, Color::from_rgb(255, 255, 255), Color::from_rgb_additive(0, 0, 0))
            }

            self.display_number(self.frame_counter, [10i16, 12].into(), 4, Color::from_rgb(255, 255, 255), Color::from_rgb_additive(0, 0, 0));

            self.interface.update_screen();
            self.frame_counter += 1;
        }

        fn draw_cube(&mut self, coords: Coord, cube_pallete: &[Color; 5]){
            let start_x = coords.x as usize * 8;
            let start_y = coords.y as usize * 8;

            for x in 0..8{
                for y in 0..8{
                    let color = match (x,y){
                        (0..=6, 0) => {
                            //top light light
                            cube_pallete[1]
                        }
                        (1..=7, 7) => {
                            //bottom dark dark
                            cube_pallete[3]
                        }
                        (0, 1..=7) => {
                            //left light
                            cube_pallete[4]
                        }
                        (7, 0..=6) => {
                            //right dark
                            cube_pallete[2]
                        }
                        _ => {
                            cube_pallete[0]
                        }
                    };
                    if color.is_opaque(){
                        self.interface.set_pixel(x + start_x, y + start_y, color);
                    }
                }
            }
        }

        fn ghost_cube(&mut self, location: Coord, color: Color){

            if !color.is_opaque(){
                return;
            }
            let location = location.scale(8);
            for x in 0..8{
                self.interface.set_pixel(location.x as usize + x, location.y as usize, color);
                self.interface.set_pixel(location.x as usize + x, location.y as usize + 7, color);
            }
            for y in 1..7{
                self.interface.set_pixel(location.x as usize, location.y as usize + y, color);
                self.interface.set_pixel(location.x as usize + 7, location.y as usize + y, color);
            }
        }

        fn display_number(&mut self, mut num: usize, mut location: Coord, leading_zeros: usize, forground: Color, background: Color){
            let mut iters = 0;
            while num > 0{
                let n = num % 10;
                num /= 10;
                //self. fill_cube(location, forground);
                self.draw_chacater(location, n + 16, forground, background);
                iters += 1;
                location.x -= 1;
            }
            for _ in iters..leading_zeros{
                self.draw_chacater(location, 16, forground, background);
                location.x -= 1;
            }
        }

        fn draw_chacater(&mut self, location: Coord, char: usize, forground: Color, background: Color){
            let char = char * 8;
            for y in 0..8{
                let mut char = CHACATER_SET[char + y];
                for x in 0..8{
                    
                    let color = if char & 1 == 1{
                        forground
                    }else{
                        background
                    };
                    if color.is_opaque(){
                        self.interface.set_pixel(x + (location.x*8) as usize, y + (location.y*8) as usize, color);
                    }
                    char >>= 1;
                }
            }
        }

        fn draw_tile(&mut self, location: Coord, tile: &[u8; 256], pallet: &[Color]){
            for x in 0..8{
                for y in 0..8{
                    let color = pallet[tile[x + y*8] as usize];
                    if color.is_opaque(){
                        self.interface.set_pixel(x + (location.x*8) as usize, y + (location.y*8) as usize, color);
                    }
                }
            }
        }

        fn fill_cube(&mut self, coords: Coord, color: Color){
            let start_x = coords.x as usize * 8;
            let start_y = coords.y as usize * 8;

            for x in 0..8{
                for y in 0..8{
                    if color.is_opaque(){
                        self.interface.set_pixel(x + start_x, y + start_y, color);
                    }
                }
            }
        }
    }
}


mod input{
    use crate::InterfaceTrait;

    use super::Tetris;

    pub struct TetrisInput{
        up: KeyState,
        left: KeyState,
        right: KeyState,
        down: KeyState,
        drop_down: KeyState,
        save: KeyState,
    }

    struct KeyState{
        key_down: bool,
        frames_down: usize,
        key_pressed: bool,
    }

    impl KeyState{
        fn new() -> Self{
            Self { 
                key_down: false, 
                frames_down: 0, 
                key_pressed: true 
            }
        }

        fn update(&mut self, down: bool){
            self.key_pressed = false;

            match (down, self.key_down){
                (true, true) => {
                    self.frames_down += 1;
                },
                (true, false) => {
                    self.key_down = true;
                    self.frames_down = 0;
                    self.key_pressed = true;
                },
                (false, true) => {
                    self.key_down = false;
                    self.frames_down = 0;
                },
                (false, false) => {

                },
            }
        }
    }

    impl TetrisInput{
        pub fn init() -> Self {
            Self {
                up: KeyState::new(),
                left: KeyState::new(),
                right: KeyState::new(),
                down: KeyState::new(),
                drop_down: KeyState::new(),
                save: KeyState::new(),
            }
        } 
    
        pub fn up_pressed(&self) -> bool{
            self.up.key_pressed
        }
        pub fn left_pressed(&self) -> bool{
            self.left.key_pressed
        }
        pub fn down_pressed(&self) -> bool{
            self.down.key_pressed
        }
        pub fn right_pressed(&self) -> bool{
            self.right.key_pressed
        }
        pub fn save_pressed(&self) -> bool{
            self.save.key_pressed
        }
        pub fn drop_down_pressed(&self) -> bool{
            self.drop_down.key_pressed
        }
    }

    impl Tetris{
        pub fn update_input(&mut self){

            self.input.up.update(self.interface.key_down('w') | self.interface.key_down('\x26'));
            self.input.left.update(self.interface.key_down('a') | self.interface.key_down('\x25'));
            self.input.down.update(self.interface.key_down('s') | self.interface.key_down('\x27'));
            self.input.right.update(self.interface.key_down('d') | self.interface.key_down('\x28'));
            self.input.save.update(self.interface.key_down('c'));
            self.input.drop_down.update(self.interface.key_down(' '));
            
            self.input.down.key_pressed = self.input.down.key_down;

            if self.input.left.frames_down > 20{
                self.input.left.key_pressed = self.input.left.frames_down % 8 == 0;
            }
            if self.input.right.frames_down > 20{
                self.input.right.key_pressed = self.input.right.frames_down % 8 == 0;
            }
        }
    }
}

mod sound{
    use super::Tetris;

    pub struct TetrisSound{
        
    }

    impl TetrisSound{
        pub fn init() -> Self {
            Self {
            }
        }
    
        
    }

    impl Tetris{
        pub fn update_sound(&self){
            
        }
    }
}


