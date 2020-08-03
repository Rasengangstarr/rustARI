use std::env;

mod rom_read;
mod mem_load;


fn main() {

   let args: Vec<String> = env::args().collect();

   assert_eq!(args.len(), 2, "wrong number of arguments provided! provide a filename only");
   let filename = &args[1];
   
   println!("reading file: {}", filename);

   let mut flags : u8 = 0x0;
   
   let rom = rom_read::get_file_as_byte_vec(filename);
   let mem : [u8; 0x1FFF]  = mem_load::write_rom_to_mem(rom);

   let pc = 0x1000;
   
   match mem[pc] {
      0x78 => flags = sei(&flags),
      0xD8 => flags = cld(&flags),
      _ => println!("not an instruction"),
   }

   println!("{:#X?}",mem[pc]);
   
}

fn sei(flags: &u8) -> u8 {
   println!("sei");
   let mut result = *flags;
   result &= !0b0000_0100;
   return result;
}

fn cld(flags: &u8) -> u8 {
   println!("cld");
   let mut result = *flags;
   result &= !0b0000_1000;
   return result;
}

