use std::env;

mod rom_read;

fn main() {
   let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2, "wrong number of arguments provided! provide a filename only");
    let filename = &args[1];
    println!("reading file: {}", filename);
    let rom = rom_read::get_file_as_byte_vec(filename);
    for (x,byte) in rom.iter().enumerate() {
       println!("{}:{:#X?}", x, byte)
    }
}

