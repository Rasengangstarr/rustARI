extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::env;
use std::string::ToString;

mod rom_read;
mod mem_load;

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

#[derive(Display, Debug)]
enum Mode {
   
   IMM,
   ZP,
   ZPX,
   ZPY,
   ABS,
   ABSX,
   ABSY,
   INDX,
   INDY
}


struct Atari {
   memory:  [u8; 0x1FFF],
   flags: u8,
   pc: usize,
   xReg: u8,
   yReg: u8,
   aReg: u8
}

impl Atari {
   /* #region Memory helpers */
   fn read_mem(&self, cell : usize) -> u8 {
      return self.memory[cell];
   }

   fn write_mem(&mut self, cell : usize, val : u8) {
      self.memory[cell] = val;
   }

   fn read_flag(&self, flag : Flag) -> bool {
      let flagu8 = flag as u8;
      return self.flags & (1 << flagu8) != 0;
   }
   
   fn write_flag(&mut self, flag_writer : FlagWriter, val : bool) {
      let fw = flag_writer as u8;
      if val {
         self.flags |= fw;
      } else {
         self.flags &= !fw;
      }
   }
   /* #endregion */

   fn execute_step(&mut self) {
      
      let pc = self.pc;

      self.pc = match self.read_mem(pc) {
         //Flag (Processor Status) Instructions
         0x18 => self.clc(pc),
         0x38 => self.sec(pc),
         0x58 => self.cli(pc),
         0x78 => self.sei(pc),
         0xB8 => self.clv(pc),
         0xD8 => self.cld(pc),
         0xF8 => self.sed(pc),

         0xA2 => self.ldx(Mode::IMM, pc),
         _ => 0,
      };

      if self.pc == 0 {
         return;
      }

      println!("{}", self.flags);
   }
   
   fn boot(&mut self) {
      loop {
         self.execute_step();
      }      
   }

   fn set_flags(&mut self, val : u8) {
      if val == 0 {
         self.write_flag(FlagWriter::ZERO, true);
      } else {
         self.write_flag(FlagWriter::ZERO, false);
      }

      if val & (1 << 1) != 0 {
         self.write_flag(FlagWriter::NEG, true);
      } else {
         self.write_flag(FlagWriter::NEG, false);
      }
   }

   /* #region Flag (Processor Status) Instructions */
   fn sei(&mut self, pc : usize) -> usize {
      println!("SEI");
      self.write_flag(FlagWriter::IRQD, true);
      return pc+1;
   }
   
   fn cli(&mut self, pc : usize) -> usize {
      println!("CLI");
      self.pc += 1;
      self.write_flag(FlagWriter::IRQD, false);
      return pc+1;
   }
   
   fn cld(&mut self, pc : usize) -> usize {
      println!("CLD");
      self.pc += 1;
      self.write_flag(FlagWriter::DEC, false);
      return pc+1;
   }
   
   fn clc(&mut self, pc : usize) -> usize {
      println!("CLC");
      self.pc += 1;
      self.write_flag(FlagWriter::CARRY, false);
      return pc+1;
   }
   
   fn clv(&mut self, pc : usize) -> usize {
      println!("CLV");
      self.pc += 1;
      self.write_flag(FlagWriter::OVER, false);
      return pc+1;
   }
   
   fn sed(&mut self, pc : usize) -> usize {
      println!("SED");
      self.pc += 1;
      self.write_flag(FlagWriter::DEC, true);
      return pc+1;
   }

   fn sec(&mut self, pc : usize) -> usize {
      println!("SEC");
      self.write_flag(FlagWriter::CARRY, true);
      return pc+1;
   }
   /* #endregion */

   fn ldx(&mut self, mode: Mode, pc : usize) -> usize {
      println!("LDX {}", mode.to_string());
      let mut pc = pc;
      match mode {
         Mode::IMM => {
            self.xReg = self.read_mem(pc+1);
            pc+=2;
         },
         Mode::ZP => {
            let target_loc = self.read_mem(pc+1) as usize;
            self.xReg = self.read_mem(target_loc);
            pc+=2;
         },
         Mode::ZPY => {
            let target_loc = (self.read_mem(pc+1) + self.yReg) as usize;
            self.xReg = self.read_mem(target_loc);
            pc+=2;
         },
         Mode::ABS => {
            let p1 : u16 = self.read_mem(pc+1) as u16;
            let p2 : u16 = self.read_mem(pc+2) as u16;
            let target_loc : u16 = p1 << 8 | p2;
            let target_loc_ind : usize = target_loc as usize;
            self.xReg = self.read_mem(target_loc_ind);
            pc+=3;
         },
         Mode::ABSY => {
            let p1 : u16 = self.read_mem(pc+1) as u16;
            let p2 : u16 = self.read_mem(pc+2) as u16;
            let target_loc : u16 = p1 << 8 | p2;
            let y_reg : u16 = self.yReg as u16;
            let target_loc_ind : usize = (target_loc + y_reg) as usize;
            self.xReg = self.read_mem(target_loc_ind);
            pc+=3;
         }
         _ => println!("MODE NOT IMPLEMENTED")
      }
      self.set_flags(self.xReg);
      return pc;
   }
}



fn main() {

   let args: Vec<String> = env::args().collect();

   assert_eq!(args.len(), 2, "wrong number of arguments provided! provide a filename only");
   let filename = &args[1];
   
   println!("reading file: {}", filename);

   let rom = rom_read::get_file_as_byte_vec(filename);
   let mut atari : Atari = Atari {memory: mem_load::write_rom_to_mem(rom),
                                    flags: 0,
                                    pc:0x1000,
                                    xReg: 0,
                                    yReg: 0,
                                    aReg: 0};   
   
   atari.boot();      

   
}
