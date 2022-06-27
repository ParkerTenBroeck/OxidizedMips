use std::{fs::File, io::Write};



fn main(){
    let decoder = png::Decoder::new(File::open("res/character-tile-set.png").unwrap());
    let mut reader = decoder.read_info().unwrap();
    
    let mut buf = vec![0u8; reader.output_buffer_size()];
    reader.next_frame(&mut buf).unwrap();
    
    let mut new: Vec<u8> = Vec::new();
    for y in 0..6{
        for x in 0..16{
            let index = x * 8 + y * 8 * 8 * 16;
            for ty in 0..8{
                let mut thing = 0u8;
                for tx in 0..8{
                    thing = (thing << 1) | {
                        if buf[index + tx + ty * 8 * 16] > 1{1} else{0}
                    }
                }
                new.push(thing.reverse_bits());
            }
        }
    }
    let mut file = File::create("res/character-tile-set.comp").unwrap();
    file.write(&new).unwrap();

    arch_specific();
}


#[cfg(not(target_arch = "mips"))]
fn arch_specific(){

}

#[cfg(target_arch = "mips")]
fn arch_specific(){

}