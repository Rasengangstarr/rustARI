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
    INDY,
    IND
 }
 
 
 pub struct Atari {
    memory:  [u8; 0x1FFF],
    flags: u8,
    pc: usize,
    x_reg: u8,
    y_reg: u8,
    a_reg: u8,
    s_pnt: u8,
    pub cycles: usize
 }
 
 impl Atari {

    pub fn new(memory : [u8; 0x1FFF], pc: usize) -> Atari { 
        Atari { 
            memory: memory,
            flags: 0,
            pc: pc,
            x_reg: 0,
            y_reg: 0,
            a_reg: 0,
            s_pnt: 0,
            cycles: 0
        }
    }

    /* #region Utility functions */
    pub fn read_mem(&self, cell : usize) -> u8 {
       return self.memory[cell];
    }
 
    fn write_mem(&mut self, cell : usize, val : u8) {
       self.memory[cell] = val;
       let cell_bytes : u16 = cell as u16;
       let tia_addr = self.translate_for_tia(cell_bytes) as usize;
       self.memory[tia_addr] = val;
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

     
    fn set_flag_zero(&mut self, val : u8) {
      if val == 0 {
         self.write_flag(FlagWriter::ZERO, true);
      } else {
         self.write_flag(FlagWriter::ZERO, false);
      }
   }

   fn set_flag_neg(&mut self, val : u8) {
      if val >= 0x40 {
         self.write_flag(FlagWriter::NEG, true);
      } else {
         self.write_flag(FlagWriter::NEG, false);
      }
   }

   fn abs_addr (&mut self, pc : usize) -> usize {
      let p2 : u16 = self.read_mem(pc+1) as u16;
      let p1 : u16 = self.read_mem(pc+2) as u16;
      let target_loc : u16 = p1 << 8 | p2;
      return self.translate_addr(target_loc) as usize;
   }
   fn abs_addr_y (&mut self, pc : usize) -> usize {
      let p2 : u16 = self.read_mem(pc+1) as u16;
      let p1 : u16 = self.read_mem(pc+2) as u16;
      let target_loc : u16 = self.translate_addr(p1 << 8 | p2);
      let y_reg : u16 = self.y_reg as u16;
      return (target_loc + y_reg) as usize;
   }
   fn abs_addr_x (&mut self, pc : usize) -> usize {
      let p2 : u16 = self.read_mem(pc+1) as u16;
      let p1 : u16 = self.read_mem(pc+2) as u16;
      let target_loc : u16 = p1 << 8 | p2;
      let x_reg : u16 = self.x_reg as u16;
      return (target_loc + x_reg) as usize;
   }
   fn translate_addr(&mut self, mut addr : u16) -> u16
   {
      addr &= 0b0001_1111_1111_1111;
      return addr;
   }
   fn translate_for_tia(&mut self, mut addr : u16) -> u16
   {
      addr &= 0b0001_0000_1011_1111;
      return addr;
   }
    
    /* #endregion */
   
    /* #region Step Executor */

    pub fn execute_step(&mut self) {
 
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
 
          //LDX (Load X regisiter)
          0xA2 => self.ldx(Mode::IMM, pc),
          0xA6 => self.ldx(Mode::ZP, pc),
          0xB6 => self.ldx(Mode::ZPY, pc),
          0xAE => self.ldx(Mode::ABS, pc),
          0xBE => self.ldx(Mode::ABSY, pc),
 
          //LDY (Load Y register)
          0xA0 => self.ldy(Mode::IMM, pc),
          0xA4 => self.ldy(Mode::ZP, pc),
          0xB4 => self.ldy(Mode::ZPX, pc),
          0xAC => self.ldy(Mode::ABS, pc),
          0xBC => self.ldy(Mode::ABSX, pc),
 
          //LDA (Load A register) - NEEDS TESTING
          0xA9 => self.lda(Mode::IMM, pc),
          0xA5 => self.lda(Mode::ZP, pc),
          0xB5 => self.lda(Mode::ZPX, pc),
          0xAD => self.lda(Mode::ABS, pc),
          0xBD => self.lda(Mode::ABSX, pc),
          0xB9 => self.lda(Mode::ABSY, pc),
 
          //STA (Store A register) - NEEDS TESTING
          0x85 => self.sta(Mode::ZP, pc),
          0x95 => self.sta(Mode::ZPX, pc),
          0x8D => self.sta(Mode::ABS, pc),
          0x9D => self.sta(Mode::ABSX, pc),
          0x99 => self.sta(Mode::ABSY, pc),
 
          //Stack Instructions
          0x9A => self.txs(pc),
          0xBA => self.tsx(pc),
          0x48 => self.pha(pc),
 
          //Register Instructions - NEEDS TESTING
          0xAA => self.tax(pc),
          0x8A => self.txa(pc),
          0xCA => self.dex(pc),
          0xE8 => self.inx(pc),
          0xA8 => self.tay(pc),
          0x98 => self.tya(pc),
          0x88 => self.dey(pc),
          0xC8 => self.iny(pc),
 
          //Branching instructions
          0xD0 => self.bne(pc),
 
          //Jump instructions
          0x4C => self.jmp(Mode::ABS, pc),
          0x6C => self.jmp(Mode::IND, pc),
          _ => panic!("INSTRUCTION NOT IMPLEMENTED: {:X?}", self.read_mem(pc)),
       };
 
       if self.pc == 0 {
          return;
       }
    }
   /* #endregion */
 
   /* #region ADC (Add with carry) Instruction */
 
    fn adc(&mut self, value: u8) {
       let mut result: u16 = self.a_reg as u16 + value as u16;
 
       if self.read_flag(Flag::CARRY) {
           result += 1;
       }
 
       if self.read_flag(Flag::DEC) {
           // BCD Mode
           if result & 0x0f > 0x09 {
               result += 0x06;
           }
 
           if result & 0xf0 > 0x90 {
               result += 0x60;
           }
       }
 
       self.a_reg = result as u8;
       // self.flag_set_if(status_flags::NEG, self.r.a & 0x80 != 0);
       // self.flag_set_if(status_flags::ZERO, self.r.a == 0);
       // self.flag_set_if(status_flags::CARRY, result > 0xff);
       // self.flag_set_if(status_flags::OVER, result >= 128);
   }

   /* #endregion */
 
    /* #region Flag (Processor Status) Instructions */
    fn sei(&mut self, pc : usize) -> usize {
       //println!("SEI");
       self.write_flag(FlagWriter::IRQD, true);
       self.cycles+=2;
       return pc+1;
    }
 
    fn cli(&mut self, pc : usize) -> usize {
       //println!("CLI");
       self.pc += 1;
       self.write_flag(FlagWriter::IRQD, false);
       self.cycles+=2;
       return pc+1;
    }
 
    fn cld(&mut self, pc : usize) -> usize {
       //println!("CLD");
       self.pc += 1;
       self.write_flag(FlagWriter::DEC, false);
       self.cycles+=2;
       return pc+1;
    }
 
    fn clc(&mut self, pc : usize) -> usize {
       //println!("CLC");
       self.pc += 1;
       self.write_flag(FlagWriter::CARRY, false);
       self.cycles+=2;
       return pc+1;
    }
 
    fn clv(&mut self, pc : usize) -> usize {
       //println!("CLV");
       self.pc += 1;
       self.write_flag(FlagWriter::OVER, false);
       self.cycles+=2;
       return pc+1;
    }
 
    fn sed(&mut self, pc : usize) -> usize {
       //println!("SED");
       self.pc += 1;
       self.write_flag(FlagWriter::DEC, true);
       self.cycles+=2;
       return pc+1;
    }
 
    fn sec(&mut self, pc : usize) -> usize {
       //println!("SEC");
       self.write_flag(FlagWriter::CARRY, true);
       self.cycles+=2;
       return pc+1;
    }
    /* #endregion */
 
    /* #region LDX */
    fn ldx(&mut self, mode: Mode, pc : usize) -> usize {
       //println!("LDX {}", mode.to_string());
       let mut pc = pc;
 
       let target_loc = match mode {
          Mode::IMM => pc+1 as usize,
          Mode::ZP => self.read_mem(pc+1) as usize,
          Mode::ZPY => (self.read_mem(pc+1) + self.y_reg) as usize,
          Mode::ABS => self.abs_addr(pc) as usize,
          Mode::ABSY => self.abs_addr_y(pc) as usize,
          _ => panic!(INV_ADD_PANIC)
       };
 
       self.x_reg = self.read_mem(target_loc);
 
       pc += match mode {
          Mode::IMM | Mode::ZP | Mode::ZPY => 2,
          Mode::ABS | Mode::ABSY => 3,
          _ => panic!(INV_ADD_PANIC)
       };
 
       self.cycles += match mode {
          Mode::IMM => 2,
          Mode::ZP => 3,
          Mode::ABSY => (
               if target_loc < 0xFF
               {
                  4
               } else
               {
                  5
               }
            ),
          _ => 4
       };

       self.set_flag_zero(self.x_reg);
       self.set_flag_neg(self.x_reg);
       return pc;
    }
    /* #endregion */
 
    /* #region LDY */
 
     fn ldy(&mut self, mode: Mode, pc : usize) -> usize {
       //println!("LDY {}", mode.to_string());
       let mut pc = pc;
 
       let target_loc = match mode {
          Mode::IMM => pc+1 as usize,
          Mode::ZP => self.read_mem(pc+1) as usize,
          Mode::ZPX => (self.read_mem(pc+1) + self.x_reg) as usize,
          Mode::ABS => self.abs_addr(pc) as usize,
          Mode::ABSX => self.abs_addr_x(pc) as usize,
          _ => panic!(INV_ADD_PANIC)
       };
 
       self.y_reg = self.read_mem(target_loc);
 
       pc += match mode {
          Mode::IMM | Mode::ZP | Mode::ZPX => 2,
          Mode::ABS | Mode::ABSX => 3,
          _ => panic!(INV_ADD_PANIC)
       };
 
       self.cycles += match mode {
         Mode::IMM => 2,
         Mode::ZP => 3,
         Mode::ABSX => (
              if target_loc < 0xFF
              {
                 4
              } else
              {
                 5
              }
           ),
         _ => 4
      };
 
       self.set_flag_zero(self.y_reg);
       self.set_flag_neg(self.y_reg);
       return pc;
    }
    /* #endregion */
 
    /* #region LDA */
 
    fn lda(&mut self, mode: Mode, pc : usize) -> usize {
       //println!("LDA {}", mode.to_string());
       let mut pc = pc;
 
       let target_loc = match mode {
          Mode::IMM => pc+1 as usize,
          Mode::ZP => self.read_mem(pc+1) as usize,
          Mode::ZPX => (self.read_mem(pc+1) + self.x_reg) as usize,
          Mode::ABS => self.abs_addr(pc) as usize,
          Mode::ABSX => self.abs_addr_x(pc) as usize,
          Mode::ABSY => self.abs_addr_y(pc) as usize,
          _ => panic!(INV_ADD_PANIC)
       };
  
       self.a_reg = self.read_mem(target_loc);
 
       pc += match mode {
          Mode::IMM | Mode::ZP | Mode::ZPX => 2,
          Mode::ABS | Mode::ABSX | Mode::ABSY => 3,
          _ => panic!(INV_ADD_PANIC)
       };
 
       self.cycles += match mode {
          Mode::IMM => 2,
          Mode::ZP => 3,
          Mode::ABSX | Mode::ABSY => (
            if target_loc < 0xFF
            {
               4
            } else
            {
               5
            }
         ),
          _ => 4
       };
 
       self.set_flag_zero(self.a_reg);
       self.set_flag_neg(self.a_reg);
       return pc;
    }
    /* #endregion */
 
    /* #region STA */
 
    fn sta(&mut self, mode: Mode, pc : usize) -> usize {
 
       let mut pc = pc;
 
       let target_loc = match mode {
          Mode::ZP => self.read_mem(pc+1) as usize,
          Mode::ZPX => (self.read_mem(pc+1) + self.x_reg) as usize,
          Mode::ABS => self.abs_addr(pc) as usize,
          Mode::ABSX => self.abs_addr_x(pc) as usize,
          Mode::ABSY => self.abs_addr_y(pc) as usize,
          _ => panic!(INV_ADD_PANIC)
       };
 
       self.write_mem(target_loc, self.a_reg);
 
       pc += match mode {
          Mode::ZP | Mode::ZPX => 2,
          Mode::ABS | Mode::ABSX | Mode::ABSY => 3,
          _ => panic!(INV_ADD_PANIC)
       };
 
       self.cycles += match mode {
          Mode::ZP => 3,
          Mode::ABS | Mode::ZPX => 4,
          _ => 5
          
       };
 
       return pc;
    }
    /* #endregion */
 
    /* #region Stack Instructions */
    fn txs(&mut self, pc : usize) -> usize {
       //println!("TXS");
       self.s_pnt = self.x_reg;
       self.cycles+=2;
       return pc + 1;
    }
    fn tsx(&mut self, pc : usize) -> usize {
       //println!("TSX");
       self.x_reg = self.s_pnt;
       self.cycles+=2;
       return pc + 1;
    }
    fn pha(&mut self, pc: usize) -> usize {
       self.write_mem(0x100 + self.s_pnt as usize, self.a_reg);
       self.s_pnt = self.s_pnt.wrapping_sub(1);
       self.cycles += 3;
       return pc + 1;
    }
    /* #endregion */
 
   /* #region Register Instructions */

     fn tax(&mut self, pc : usize) -> usize {
       ////println!("TAX");
       self.x_reg = self.a_reg;
       self.set_flag_zero(self.x_reg);
       self.set_flag_neg(self.x_reg);
       self.cycles += 2;
       return pc + 1;
    }
    fn txa(&mut self, pc : usize) -> usize {
       ////println!("TXA");
       self.a_reg = self.x_reg;
       self.set_flag_zero(self.a_reg);
       self.set_flag_neg(self.a_reg);
       self.cycles += 2;
       return pc + 1;
    }
    fn dex(&mut self, pc : usize) -> usize {
       ////println!("DEX");
       //println!("{}",self.x_reg);
       if self.x_reg == 0 {
          self.x_reg = 0xFF;
       }
       else {
          self.x_reg -= 1;
       }
       self.set_flag_zero(self.x_reg);
       self.set_flag_neg(self.x_reg);
       self.cycles += 2;
       return pc + 1;
    }
    fn inx(&mut self, pc : usize) -> usize {
       ////println!("INX");
       self.x_reg += 1;
       //self.set_flags(self.x_reg);
       self.cycles += 2;
       return pc + 1;
    }
    fn tay(&mut self, pc : usize) -> usize {
       ////println!("TAY");
       self.y_reg = self.a_reg;
       //self.set_flags(self.y_reg);
       self.cycles += 2;
       return pc + 1;
    }
    fn tya(&mut self, pc : usize) -> usize {
       ////println!("TYA");
       self.a_reg = self.y_reg;
       //self.set_flags(self.a_reg);
       self.cycles += 2;
       return pc + 1;
    }
    fn dey(&mut self, pc : usize) -> usize {
       ////println!("DEY");
       if self.y_reg == 0 {
          self.y_reg = 0xFF;
       }
       else {
          self.y_reg -= 1;
       }
       //self.set_flags(self.y_reg);
       self.cycles += 2;
       return pc + 1;
    }
    fn iny(&mut self, pc : usize) -> usize {
       //println!("INY");
       self.y_reg += 1;
       //self.set_flags(self.y_reg);
       self.cycles += 2;
       return pc + 1;
    }
    /* #endregion */
 
    /* #region Branching Instructions */
    fn bne(&mut self, pc : usize) -> usize {
       //println!("BNE");
       if self.read_flag(Flag::ZERO) {
          self.cycles+=2;
          return pc+2;
       } else {
          let step = self.read_mem(pc+1) as i8;
          let step = step as i32;
          let pci = pc as i32;
          self.cycles += 3;
          return (pci+step+2) as usize;
       }
 
    }
    /* #endregion */
 
    /* #region Jumping Instructions */
    fn jmp(&mut self, mode: Mode, pc : usize) -> usize {
       //println!("BNE");
 
       let target_loc = match mode {
          Mode::ABS => self.abs_addr(pc) as usize,
          _ => panic!(INV_ADD_PANIC)
       };
       self.cycles += 3;
       return (target_loc) as usize;
    }
 
    /* #endregion */
 }
 

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_atari() -> Atari {
        return Atari {memory: [0; 0x1FFF],
            flags: 0,
            pc:0x1000,
            x_reg: 0,
            y_reg: 0,
            a_reg: 0,
            s_pnt: 0,
            cycles: 0};
        }
   /* #region ldx tests */
    #[test]
    fn test_ldx_imm() {
        let mut atari = setup_atari();
        let expected = 0x10;
        atari.memory[1] = expected;
        let pc = atari.ldx(Mode::IMM, 0);
        assert_eq!(atari.x_reg, expected);
        assert_eq!(pc, 3);
        assert_eq!(atari.cycles, 2);
        assert_eq!(atari.read_flag(Flag::NEG), false);
        assert_eq!(atari.read_flag(Flag::ZERO), false);
    }

    #[test]
    fn test_ldx_zp() {
        let mut atari = setup_atari();
        let expected = 0x12;
        atari.memory[0x10] = expected;
        atari.memory[1]    = 0x10;
        let pc = atari.ldx(Mode::ZP, 0);
        assert_eq!(atari.x_reg, expected);
        assert_eq!(pc, 2);
        assert_eq!(atari.cycles, 3);
        assert_eq!(atari.read_flag(Flag::NEG), false);
        assert_eq!(atari.read_flag(Flag::ZERO), false);
    }

    #[test]
    fn test_ldx_zpy() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x10+5] = expected;
        atari.memory[0x10] = 0x12;
        atari.memory[1]    = 0x10;
        atari.y_reg         = 5;
        let pc = atari.ldx(Mode::ZPY, 0);
        assert_eq!(atari.x_reg, expected);
        assert_eq!(atari.cycles, 4);
        assert_eq!(pc, 2);
        assert_eq!(atari.read_flag(Flag::NEG), false);
        assert_eq!(atari.read_flag(Flag::ZERO), false);
    }

    #[test]
    fn test_ldx_abs() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x1210] = expected;
        atari.memory[1]    = 0x10;
        atari.memory[2]    = 0x12;
        let pc = atari.ldx(Mode::ABS, 0);
        assert_eq!(atari.x_reg, expected);
        assert_eq!(atari.cycles, 4);
        assert_eq!(pc, 3);
        assert_eq!(atari.read_flag(Flag::NEG), false);
        assert_eq!(atari.read_flag(Flag::ZERO), false);
    }

    #[test]
    fn test_ldx_absy() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x0010+5] = expected;
        atari.memory[1]    = 0x10;
        atari.memory[2]    = 0x00;
        atari.y_reg = 5;
        let pc = atari.ldx(Mode::ABSY, 0);
        assert_eq!(atari.x_reg, expected);
        assert_eq!(atari.cycles, 4);
        assert_eq!(pc, 3);
        assert_eq!(atari.read_flag(Flag::NEG), false);
        assert_eq!(atari.read_flag(Flag::ZERO), false);
    }

    #[test]
    fn test_ldx_absy_page_boundary() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x1210+5] = expected;
        atari.memory[1]    = 0x10;
        atari.memory[2]    = 0x12;
        atari.y_reg = 5;
        let pc = atari.ldx(Mode::ABSY, 0);
        assert_eq!(atari.x_reg, expected);
        assert_eq!(atari.cycles, 5);
        assert_eq!(pc, 3);
        assert_eq!(atari.read_flag(Flag::NEG), false);
        assert_eq!(atari.read_flag(Flag::ZERO), false);
    }

    #[test]
    #[should_panic(expected = "INVALID ADDRESSING MODE!!!")]
    fn test_ldx_invalid_mode() {
        let mut atari = setup_atari();
        atari.ldx(Mode::ABSX, 0);
    }

    /* #endregion */

    /* #region ldy tests */
    #[test]
    fn test_ldy_imm() {
        let mut atari = setup_atari();
        let expected = 0x10;
        atari.memory[1] = expected;
        let pc = atari.ldy(Mode::IMM, 0);
        assert_eq!(atari.y_reg, expected);
        assert_eq!(pc, 2);
        assert_eq!(atari.cycles, 2);
        assert_eq!(atari.read_flag(Flag::NEG), false);
        assert_eq!(atari.read_flag(Flag::ZERO), false);
    }

    #[test]
    fn test_ldy_zp() {
        let mut atari = setup_atari();
        let expected = 0x12;
        atari.memory[0x10] = expected;
        atari.memory[1]    = 0x10;
        let pc = atari.ldy(Mode::ZP, 0);
        assert_eq!(atari.y_reg, expected);
        assert_eq!(pc, 2);
        assert_eq!(atari.cycles, 3);
        assert_eq!(atari.read_flag(Flag::NEG), false);
        assert_eq!(atari.read_flag(Flag::ZERO), false);
    }

    #[test]
    fn test_ldy_zpx() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x10+5] = expected;
        atari.memory[0x10] = 0x12;
        atari.memory[1]    = 0x10;
        atari.x_reg         = 5;
        let pc = atari.ldy(Mode::ZPX, 0);
        assert_eq!(atari.y_reg, expected);
        assert_eq!(atari.cycles, 4);
        assert_eq!(pc, 2);
        assert_eq!(atari.read_flag(Flag::NEG), false);
        assert_eq!(atari.read_flag(Flag::ZERO), false);
    }

    #[test]
    fn test_ldy_abs() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x1210] = expected;
        atari.memory[1]    = 0x10;
        atari.memory[2]    = 0x12;
        let pc = atari.ldy(Mode::ABS, 0);
        assert_eq!(atari.y_reg, expected);
        assert_eq!(atari.cycles, 4);
        assert_eq!(pc, 3);
        assert_eq!(atari.read_flag(Flag::NEG), false);
        assert_eq!(atari.read_flag(Flag::ZERO), false);
    }

    #[test]
    fn test_ldy_absx() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x0010+5] = expected;
        atari.memory[1]    = 0x10;
        atari.memory[2]    = 0x00;
        atari.x_reg = 5;
        let pc = atari.ldy(Mode::ABSX, 0);
        assert_eq!(atari.y_reg, expected);
        assert_eq!(atari.cycles, 4);
        assert_eq!(pc, 3);
        assert_eq!(atari.read_flag(Flag::NEG), false);
        assert_eq!(atari.read_flag(Flag::ZERO), false);
    }

    #[test]
    fn test_ldx_absx_page_boundary() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x1210+5] = expected;
        atari.memory[1]    = 0x10;
        atari.memory[2]    = 0x12;
        atari.x_reg = 5;
        let pc = atari.ldy(Mode::ABSX, 0);
        assert_eq!(atari.y_reg, expected);
        assert_eq!(atari.cycles, 5);
        assert_eq!(pc, 3);
        assert_eq!(atari.read_flag(Flag::NEG), false);
        assert_eq!(atari.read_flag(Flag::ZERO), false);
    }

    #[test]
    #[should_panic(expected = "INVALID ADDRESSING MODE!!!")]
    fn test_ldy_invalid_mode() {
        let mut atari = setup_atari();
        atari.ldy(Mode::ABSY, 0);
    }

   /* #endregion */

   /* #region lda tests */
   #[test]
   fn test_lda_imm() {
       let mut atari = setup_atari();
       let expected = 0x10;
       atari.memory[1] = expected;
       let pc = atari.lda(Mode::IMM, 0);
       assert_eq!(atari.a_reg, expected);
       assert_eq!(pc, 2);
       assert_eq!(atari.cycles, 2);
       assert_eq!(atari.read_flag(Flag::NEG), false);
       assert_eq!(atari.read_flag(Flag::ZERO), false);
   }

   #[test]
   fn test_lda_zp() {
       let mut atari = setup_atari();
       let expected = 0x12;
       atari.memory[0x10] = expected;
       atari.memory[1]    = 0x10;
       let pc = atari.lda(Mode::ZP, 0);
       assert_eq!(atari.a_reg, expected);
       assert_eq!(pc, 2);
       assert_eq!(atari.cycles, 3);
       assert_eq!(atari.read_flag(Flag::NEG), false);
       assert_eq!(atari.read_flag(Flag::ZERO), false);
   }

   #[test]
    fn test_lda_zpx() {
        let mut atari = setup_atari();
        let expected = 0x9;
        atari.memory[0x10+5] = expected;
        atari.memory[0x10] = 0x12;
        atari.memory[1]    = 0x10;
        atari.x_reg         = 5;
        let pc = atari.lda(Mode::ZPX, 0);
        assert_eq!(atari.a_reg, expected);
        assert_eq!(atari.cycles, 4);
        assert_eq!(pc, 2);
        assert_eq!(atari.read_flag(Flag::NEG), false);
        assert_eq!(atari.read_flag(Flag::ZERO), false);
    }

   #[test]
   fn test_lda_abs() {
       let mut atari = setup_atari();
       let expected = 0x9;
       atari.memory[0x1210] = expected;
       atari.memory[1]    = 0x10;
       atari.memory[2]    = 0x12;
       let pc = atari.lda(Mode::ABS, 0);
       assert_eq!(atari.a_reg, expected);
       assert_eq!(atari.cycles, 4);
       assert_eq!(pc, 3);
       assert_eq!(atari.read_flag(Flag::NEG), false);
       assert_eq!(atari.read_flag(Flag::ZERO), false);
   }

   #[test]
   fn test_lda_absy() {
       let mut atari = setup_atari();
       let expected = 0x9;
       atari.memory[0x0010+5] = expected;
       atari.memory[1]    = 0x10;
       atari.memory[2]    = 0x00;
       atari.y_reg = 5;
       let pc = atari.lda(Mode::ABSY, 0);
       assert_eq!(atari.a_reg, expected);
       assert_eq!(atari.cycles, 4);
       assert_eq!(pc, 3);
       assert_eq!(atari.read_flag(Flag::NEG), false);
       assert_eq!(atari.read_flag(Flag::ZERO), false);
   }

   #[test]
   fn test_lda_absy_page_boundary() {
       let mut atari = setup_atari();
       let expected = 0x9;
       atari.memory[0x1210+5] = expected;
       atari.memory[1]    = 0x10;
       atari.memory[2]    = 0x12;
       atari.y_reg = 5;
       let pc = atari.lda(Mode::ABSY, 0);
       assert_eq!(atari.a_reg, expected);
       assert_eq!(atari.cycles, 5);
       assert_eq!(pc, 3);
       assert_eq!(atari.read_flag(Flag::NEG), false);
       assert_eq!(atari.read_flag(Flag::ZERO), false);
   }

   #[test]
   fn test_lda_absx() {
       let mut atari = setup_atari();
       let expected = 0x9;
       atari.memory[0x0010+5] = expected;
       atari.memory[1]    = 0x10;
       atari.memory[2]    = 0x00;
       atari.x_reg = 5;
       let pc = atari.lda(Mode::ABSX, 0);
       assert_eq!(atari.a_reg, expected);
       assert_eq!(atari.cycles, 4);
       assert_eq!(pc, 3);
       assert_eq!(atari.read_flag(Flag::NEG), false);
       assert_eq!(atari.read_flag(Flag::ZERO), false);
   }

   #[test]
   fn test_lda_absx_page_boundary() {
       let mut atari = setup_atari();
       let expected = 0x9;
       atari.memory[0x1210+5] = expected;
       atari.memory[1]    = 0x10;
       atari.memory[2]    = 0x12;
       atari.x_reg = 5;
       let pc = atari.lda(Mode::ABSX, 0);
       assert_eq!(atari.a_reg, expected);
       assert_eq!(atari.cycles, 5);
       assert_eq!(pc, 3);
       assert_eq!(atari.read_flag(Flag::NEG), false);
       assert_eq!(atari.read_flag(Flag::ZERO), false);
   }

   #[test]
   #[should_panic(expected = "INVALID ADDRESSING MODE!!!")]
   fn test_lda_invalid_mode() {
       let mut atari = setup_atari();
       atari.lda(Mode::ZPY, 0);
   }

   /* #endregion */


   /* #region Flag (Processor Status) Instructions tests */
   #[test]
   fn test_sec() {
      let mut atari = setup_atari();
      let pc = atari.sec(0);
      assert_eq!(atari.read_flag(Flag::CARRY), true);
      assert_eq!(1, pc);
      assert_eq!(atari.cycles, 2);
   }

   #[test]
   fn test_cli() {
      let mut atari = setup_atari();
      atari.write_flag(FlagWriter::IRQD, true);
      let pc = atari.cli(0);
      assert_eq!(atari.read_flag(Flag::IRQD), false);
      assert_eq!(1, pc);
      assert_eq!(atari.cycles, 2);
   }

   #[test]
   fn test_sei() {
      let mut atari = setup_atari();
      let pc = atari.sei(0);
      assert_eq!(atari.read_flag(Flag::IRQD), true);
      assert_eq!(1, pc);
      assert_eq!(atari.cycles, 2);
   }

   #[test]
   fn test_clv() {
      let mut atari = setup_atari();
      atari.write_flag(FlagWriter::OVER, true);
      let pc = atari.clv(0);
      assert_eq!(atari.read_flag(Flag::OVER), false);
      assert_eq!(1, pc);
      assert_eq!(atari.cycles, 2);
   }

   #[test]
   fn test_cld() {
      let mut atari = setup_atari();
      atari.write_flag(FlagWriter::DEC, true);
      let pc = atari.cld(0);
      assert_eq!(atari.read_flag(Flag::DEC), false);
      assert_eq!(1, pc);
      assert_eq!(atari.cycles, 2);
   }

   #[test]
   fn test_sed() {
      let mut atari = setup_atari();
      let pc = atari.sed(0);
      assert_eq!(atari.read_flag(Flag::DEC), true);
      assert_eq!(1, pc);
      assert_eq!(atari.cycles, 2);
   }

   #[test]
   fn test_txs() {
      let mut atari = setup_atari();
      atari.x_reg = 0x12;
      atari.txs(0);
      assert_eq!(atari.s_pnt, 0x12);
   }

   #[test]
   fn test_tsx() {
      let mut atari = setup_atari();
      atari.s_pnt = 0x12;
      atari.tsx(0);
      assert_eq!(atari.x_reg, 0x12);
   }
   /* #endregion */
   
   /* #region utility functions tests */

   #[test]
   fn test_translate_for_tia() {
      let mut atari = setup_atari();
      let result = atari.translate_for_tia(0xEF3F);
      assert_eq!(result, 0x3F);
   }

   #[test]
   fn test_set_flag_neg_false() {
      let mut atari = setup_atari();
      let value = 0x12;
      atari.set_flag_neg(value);
      assert_eq!(atari.read_flag(Flag::NEG), false);
   }

   #[test]
   fn test_set_flag_neg_true() {
      let mut atari = setup_atari();
      let value = 0x40;
      atari.set_flag_neg(value);
      assert_eq!(atari.read_flag(Flag::NEG), true);
      let value = 0xFF;
      atari.set_flag_neg(value);
      assert_eq!(atari.read_flag(Flag::NEG), true);
   }

   #[test]
   fn test_set_flag_zero_false() {
      let mut atari = setup_atari();
      let value = 0x12;
      atari.set_flag_zero(value);
      assert_eq!(atari.read_flag(Flag::ZERO), false);
      let value = 0xFF;
      atari.set_flag_zero(value);
      assert_eq!(atari.read_flag(Flag::ZERO), false);
   }

   #[test]
   fn test_set_flag_zero_true() {
      let mut atari = setup_atari();
      let value = 0x00;
      atari.set_flag_zero(value);
      assert_eq!(atari.read_flag(Flag::ZERO), true);
   }

   /* #endregion */


}