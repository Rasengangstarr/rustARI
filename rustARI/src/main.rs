use std::env;

mod rom_read;
mod mem_load;

static mut MEM: [u8; 0x1FFF] =  [0; 0x1FFF];
static mut FLAGS: u8 = 0;

enum FlagWriter {
   NEG = 0b1000_0000,
   OVER = 0b0100_0000,
   UNUSED = 0b0010_0000,
   BRK = 0b0001_0000,
   DEC = 0b0000_1000,
   IRQD = 0b0000_0100,
   ZERO = 0b0000_0010,
   CARRY = 0b0000_0001
}

enum Flag {
   CARRY,
   ZERO,
   IRQD,
   DEC,
   BRK,
   UNUSED,
   OVER,
   NEG
}

unsafe fn read_mem(cell : usize) -> u8 {
   return MEM[cell];
}

unsafe fn write_mem(cell : usize, val : u8) {
   MEM[cell] = val;
}

unsafe fn read_flag(flag : Flag) -> bool {
   let flagu8 = flag as u8;
   return FLAGS & (1 << flagu8) != 0;
}

unsafe fn write_flag(flagWriter : FlagWriter, val : bool) {
   let fw = flagWriter as u8;
   if val {
      FLAGS &= fw;
   } else {
      FLAGS |= !fw;
   }
}

fn main() {

   let args: Vec<String> = env::args().collect();

   assert_eq!(args.len(), 2, "wrong number of arguments provided! provide a filename only");
   let filename = &args[1];
   
   println!("reading file: {}", filename);

   let rom = rom_read::get_file_as_byte_vec(filename);
   unsafe {
      MEM = mem_load::write_rom_to_mem(rom);
   
      let pc = 0x1000;
      
      match read_mem(pc) {
         0x18 => clc(),
         0x38 => sec(),
         0x58 => cli(),
         0x78 => sei(),
         0xB8 => clv(),
         0xD8 => cld(),
         0xF8 => sed(),

         _ => println!("not an instruction"),
      }
   }
   
}

unsafe fn sei() {
   println!("SEI");
   write_flag(FlagWriter::IRQD, true);
}

unsafe fn cli() {
   println!("CLI");
   write_flag(FlagWriter::IRQD, false);
}

unsafe fn cld() {
   println!("CLD");
   write_flag(FlagWriter::DEC, true);
}

unsafe fn clc() {
   println!("CLC");
   write_flag(FlagWriter::CARRY, false);
}

unsafe fn clv() {
   println!("CLV");
   write_flag(FlagWriter::OVER, true);
}

unsafe fn sed() {
   println!("SED");
   write_flag(FlagWriter::DEC, true);
}


unsafe fn sec() {
   println!("SEC");
   write_flag(FlagWriter::CARRY, true);
}
