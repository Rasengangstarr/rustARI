extern crate strum;
#[macro_use]
extern crate strum_macros;

extern crate piston_window;

extern crate find_folder;

use std::env;
use std::string::ToString;
use piston_window::*;

use std::str;

mod rom_read;
mod mem_load;

const INV_ADD_PANIC : &str = "INVALID ADDRESSING MODE!!!";

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

         //LDX (Load X register)
         0xA2 => self.ldx(Mode::IMM, pc),
         0xA6 => self.ldx(Mode::ZP, pc),
         0xB6 => self.ldx(Mode::ZPY, pc),
         0xAE => self.ldx(Mode::ABS, pc),
         0xBE => self.ldx(Mode::ABSY, pc),

         //LDY (Load Y register)
         0xA0 => self.ldx(Mode::IMM, pc),
         0xA4 => self.ldx(Mode::ZP, pc),
         0xB4 => self.ldx(Mode::ZPX, pc),
         0xAC => self.ldx(Mode::ABS, pc),
         0xBC => self.ldx(Mode::ABSX, pc),
         
         _ => panic!("INSTRUCTION NOT IMPLEMENTED: {:X?}", self.read_mem(pc)),
      };

      if self.pc == 0 {
         return;
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

   fn abs_addr (&mut self, pc : usize) -> usize {
      let p1 : u16 = self.read_mem(pc+1) as u16;
      let p2 : u16 = self.read_mem(pc+2) as u16;
      let target_loc : u16 = p1 << 8 | p2;
      return target_loc as usize;
   }
   fn abs_addr_y (&mut self, pc : usize) -> usize {
      let p1 : u16 = self.read_mem(pc+1) as u16;
      let p2 : u16 = self.read_mem(pc+2) as u16;
      let target_loc : u16 = p1 << 8 | p2;
      let y_reg : u16 = self.yReg as u16;
      return (target_loc + y_reg) as usize;
   }
   fn abs_addr_x (&mut self, pc : usize) -> usize {
      let p1 : u16 = self.read_mem(pc+1) as u16;
      let p2 : u16 = self.read_mem(pc+2) as u16;
      let target_loc : u16 = p1 << 8 | p2;
      let x_reg : u16 = self.xReg as u16;
      return (target_loc + x_reg) as usize;
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

   /* #region LDX */
   fn ldx(&mut self, mode: Mode, pc : usize) -> usize {
      println!("LDX {}", mode.to_string());
      let mut pc = pc;

      let target_loc = match mode {
         Mode::IMM => pc+1 as usize,
         Mode::ZP => self.read_mem(pc+1) as usize,
         Mode::ZPY => (self.read_mem(pc+1) + self.yReg) as usize,
         Mode::ABS => self.abs_addr(pc) as usize,
         Mode::ABSY => self.abs_addr_y(pc) as usize,
         _ => panic!(INV_ADD_PANIC)
      };

      self.xReg = self.read_mem(target_loc);

      pc += match mode {
         Mode::IMM | Mode::ZP | Mode::ZPY => 2,
         Mode::ABS | Mode::ABSY => 3,
         _ => panic!(INV_ADD_PANIC)
      };

      self.set_flags(self.xReg);
      return pc;
   }
   /* #endregion */

   /* #region LDY */
    fn ldy(&mut self, mode: Mode, pc : usize) -> usize {
      println!("LDY {}", mode.to_string());
      let mut pc = pc;

      let target_loc = match mode {
         Mode::IMM => pc+1 as usize,
         Mode::ZP => self.read_mem(pc+1) as usize,
         Mode::ZPX => (self.read_mem(pc+1) + self.xReg) as usize,
         Mode::ABS => self.abs_addr(pc) as usize,
         Mode::ABSX => self.abs_addr_x(pc) as usize,
         _ => panic!(INV_ADD_PANIC)
      };

      self.yReg = self.read_mem(target_loc);

      pc += match mode {
         Mode::IMM | Mode::ZP | Mode::ZPX => 2,
         Mode::ABS | Mode::ABSX => 3,
         _ => panic!(INV_ADD_PANIC)
      };

      self.set_flags(self.xReg);
      return pc;
   }
   /* #endregion */
}



fn main() {

   let args: Vec<String> = env::args().collect();

   assert_eq!(args.len(), 2, "wrong number of arguments provided! provide a filename only");
   let filename = &args[1];
   
   println!("reading file: {}", filename);

   let rom = rom_read::get_file_as_byte_vec(filename);
   let atari : Atari = Atari {memory: mem_load::write_rom_to_mem(rom),
                                    flags: 0,
                                    pc:0x1000,
                                    xReg: 0,
                                    yReg: 0,
                                    aReg: 0};   
   main_loop(atari);
}

fn main_loop(mut atari : Atari) {

   let mut window: PistonWindow = WindowSettings::new(
      "rustARI",
      [200, 200]
  )
  .exit_on_esc(true)
  //.opengl(OpenGL::V2_1) // Set a different OpenGl version
  .build()
  .unwrap();
   
   let assets = find_folder::Search::ParentsThenKids(3, 3)
      .for_folder("assets").unwrap();
   println!("{:?}", assets);
   let mut glyphs = window.load_font(assets.join("consola.ttf")).unwrap();

   while let Some(event) = window.next() {
      
      window.draw_2d(&event, |c, g, device| {
         clear([0.0, 0.0, 0.0, 1.0], g);
         let transform = c.transform.trans(14.0, 14.0);
         text::Text::new_color([0.0, 1.0, 0.0, 1.0], 14).draw(
             "NOUBDIZC",
             &mut glyphs,
             &c.draw_state,
             transform, g
         ).unwrap();

         let transform = c.transform.trans(14.0, 30.0);
         let flags_byte = atari.flags;
         let flags_text = &format!("{:08b}", flags_byte);
         text::Text::new_color([0.0, 1.0, 0.0, 1.0], 14).draw(
               &flags_text,
               &mut glyphs,
               &c.draw_state,
               transform, g
         ).unwrap();

         // Update glyphs before rendering.
         glyphs.factory.encoder.flush(device);
        });

      //atari.execute_step();
}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_atari() -> Atari {
        return Atari {memory: [0; 0x1FFF],
            flags: 0,
            pc:0x1000,
            xReg: 0,
            yReg: 0,
            aReg: 0};
        }
    
    #[test]
    fn test_ldx_imm() {
        let mut atari = setup_atari();
        let expected = 0x10;
        atari.memory[1] = expected;
        atari.ldx(Mode::IMM, 0);
        assert_eq!(atari.xReg, expected);
    }

    #[test]
    fn test_ldx_zp() {
        let mut atari = setup_atari();
        let expected = 0x12;
        atari.memory[0x10] = expected;
        atari.memory[1]    = 0x10;
        atari.ldx(Mode::ZP, 0);
        assert_eq!(atari.xReg, expected);
    }

    #[test]
    fn test_ldx_zpy() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x10+5] = expected;
        atari.memory[0x10] = 0x12;
        atari.memory[1]    = 0x10;
        atari.yReg         = 5;
        atari.ldx(Mode::ZPY, 0);
        assert_eq!(atari.xReg, expected);
    }

    #[test]
    fn test_ldx_abs() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x1012] = expected;
        atari.memory[1]    = 0x10;
        atari.memory[2]    = 0x12;
        atari.ldx(Mode::ABS, 0);
        assert_eq!(atari.xReg, expected);
    }

    #[test]
    fn test_ldx_absy() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x1012+5] = expected;
        atari.memory[1]    = 0x10;
        atari.memory[2]    = 0x12;
        atari.yReg = 5;
        atari.ldx(Mode::ABSY, 0);
        assert_eq!(atari.xReg, expected);
    }

    #[test]
    #[should_panic(expected = "INVALID ADDRESSING MODE!!!")]
    fn test_ldx_invalid_mode() {
        let mut atari = setup_atari();
        atari.ldx(Mode::ABSX, 0);
    }

    #[test]
    fn test_ldy_imm() {
        let mut atari = setup_atari();
        let expected = 0x10;
        atari.memory[1] = expected;
        atari.ldy(Mode::IMM, 0);
        assert_eq!(atari.yReg, expected);
    }
    

    #[test]
    fn test_ldy_zp() {
        let mut atari = setup_atari();
        let expected = 0x12;
        atari.memory[0x10] = expected;
        atari.memory[1]    = 0x10;
        atari.ldy(Mode::ZP, 0);
        assert_eq!(atari.yReg, expected);
    }

    #[test]
    fn test_ldy_zpx() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x10+5] = expected;
        atari.memory[0x10] = 0x12;
        atari.memory[1]    = 0x10;
        atari.xReg         = 5;
        atari.ldy(Mode::ZPX, 0);
        assert_eq!(atari.yReg, expected);
    }

    #[test]
    fn test_ldy_abs() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x1012] = expected;
        atari.memory[1]    = 0x10;
        atari.memory[2]    = 0x12;
        atari.ldy(Mode::ABS, 0);
        assert_eq!(atari.yReg, expected);
    }

    #[test]
    fn test_ldy_absy() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x1012+5] = expected;
        atari.memory[1]    = 0x10;
        atari.memory[2]    = 0x12;
        atari.xReg = 5;
        atari.ldy(Mode::ABSX, 0);
        assert_eq!(atari.yReg, expected);
    }

    #[test]
    #[should_panic(expected = "INVALID ADDRESSING MODE!!!")]
    fn test_ldy_invalid_mode() {
        let mut atari = setup_atari();
        atari.ldy(Mode::ABSY, 0);
    }

}