use std::env;

mod rom_read;
mod mem_load;


fn main() {

   let args: Vec<String> = env::args().collect();

   assert_eq!(args.len(), 2, "wrong number of arguments provided! provide a filename only");
   let filename = &args[1];
   
   println!("reading file: {}", filename);

   let rom = rom_read::get_file_as_byte_vec(filename);
   let mut mem : [u8; 0x1FFF]  = mem_load::write_rom_to_mem(rom);

   
   
}



