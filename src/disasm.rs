use memory::MemSegment;
use cpu::CPU;

pub struct Disassembler<'a> {
    pc: u16,
    cpu: &'a mut CPU,
    bytes: Vec<u8>,
}

struct PartialInstruction {
    pattern: String,
}

impl PartialInstruction {
    fn finish(self, instr: &str) -> String {
        self.pattern.replace("$$$", instr).clone()
    }
}

pub struct Instruction {
    pub bytes: Vec<u8>,
    pub str: String,
}

impl<'a> Disassembler<'a> {
    pub fn new(cpu: &'a mut CPU) -> Disassembler {
        Disassembler {
            pc: cpu.pc,
            cpu: cpu,
            bytes: vec![],
        }
    }

    // Addressing modes
    fn immediate(&mut self) -> PartialInstruction {
        PartialInstruction { pattern: format!("$$$ #${:02X}", self.read_incr_pc()) }
    }
    fn absolute(&mut self) -> PartialInstruction {
        let arg = self.read_w_incr_pc();
        PartialInstruction {
            pattern: format!("$$$ #${:04X} = {:02X}", arg, self.cpu.mem.read(arg)),
        }
    }
    fn absolute_x(&mut self) -> PartialInstruction {
        let arg = self.read_w_incr_pc();
        let target = arg.wrapping_add(self.cpu.x as u16);
        PartialInstruction {
            pattern: format!("$$$ #${:04X},X @ {:02X} = {:02X}",
                             arg,
                             self.cpu.x,
                             self.cpu.mem.read(target)),
        }
    }
    fn absolute_y(&mut self) -> PartialInstruction {
        let arg = self.read_w_incr_pc();
        let target = arg.wrapping_add(self.cpu.y as u16);
        PartialInstruction {
            pattern: format!("$$$ #${:04X},Y @ {:02X} = {:02X}",
                             arg,
                             self.cpu.y,
                             self.cpu.mem.read(target)),
        }
    }
    fn zero_page(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        PartialInstruction {
            pattern: format!("$$$ ${:02X} = {:02X}", arg, self.cpu.mem.read(arg as u16)),
        }
    }
    fn zero_page_x(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        let target = arg.wrapping_add(self.cpu.x);
        PartialInstruction {
            pattern: format!("$$$ #${:02X},X @ {:02X} = {:02X}",
                             arg,
                             self.cpu.x,
                             self.cpu.mem.read(target as u16)),
        }
    }
    fn zero_page_y(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        let target = arg.wrapping_add(self.cpu.y);
        PartialInstruction {
            pattern: format!("$$$ #${:02X},Y @ {:02X} = {:02X}",
                             arg,
                             self.cpu.y,
                             self.cpu.mem.read(target as u16)),
        }
    }
    fn indirect_x(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        let zp_idx = arg.wrapping_add(self.cpu.x);
        let ptr = self.cpu.mem.read_w(zp_idx as u16);
        let target = self.cpu.mem.read(ptr);
        PartialInstruction {
            pattern: format!("$$$ ({:02X},X) @ {:02X} = {:04X} = {:04X}",
                             arg,
                             zp_idx,
                             ptr,
                             target),
        }
    }
    fn indirect_y(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        let base_ptr = self.cpu.mem.read_w(arg as u16);
        let ptr = base_ptr.wrapping_add(self.cpu.y as u16);
        let target = self.cpu.mem.read(ptr);
        PartialInstruction {
            pattern: format!("$$$ ({:02X}),Y = {:04X} @ {:04X} = {:04X}",
                             arg,
                             base_ptr,
                             ptr,
                             target),
        }
    }

    // Instructions
    // Stores
    fn stx(&mut self, instr: PartialInstruction) -> String {
        instr.finish("STX")
    }
    fn sta(&mut self, instr: PartialInstruction) -> String {
        instr.finish("STA")
    }

    // Loads
    fn ldx(&mut self, instr: PartialInstruction) -> String {
        instr.finish("LDX")
    }
    fn lda(&mut self, instr: PartialInstruction) -> String {
        instr.finish("LDA")
    }

    // Logic/Math Ops
    fn bit(&mut self, instr: PartialInstruction) -> String {
        instr.finish("BIT")
    }
    fn and(&mut self, instr: PartialInstruction) -> String {
        instr.finish("AND")
    }
    fn ora(&mut self, instr: PartialInstruction) -> String {
        instr.finish("ORA")
    }
    fn eor(&mut self, instr: PartialInstruction) -> String {
        instr.finish("EOR")
    }
    fn adc(&mut self, instr: PartialInstruction) -> String {
        instr.finish("ADC")
    }
    fn cmp(&mut self, instr: PartialInstruction) -> String {
        instr.finish("CMP")
    }

    // Jumps
    fn jmp(&mut self) -> String {
        format!("JMP ${:04X}", self.read_w_incr_pc())
    }
    fn jmpi(&mut self) -> String {
        let arg = self.read_w_incr_pc();
        format!("JMP ({:04X}) = {:04X}", arg, self.cpu.mem.read_w(arg))
    }
    fn jsr(&mut self) -> String {
        let arg = self.read_w_incr_pc();
        format!("JSR ${:04X}", arg)
    }
    fn rts(&mut self) -> String {
        "RTS".to_string()
    }

    // Branches
    fn bcs(&mut self) -> String {
        self.branch("BCS")
    }
    fn bcc(&mut self) -> String {
        self.branch("BCC")
    }
    fn beq(&mut self) -> String {
        self.branch("BEQ")
    }
    fn bne(&mut self) -> String {
        self.branch("BNE")
    }
    fn bvs(&mut self) -> String {
        self.branch("BVS")
    }
    fn bvc(&mut self) -> String {
        self.branch("BVC")
    }
    fn bmi(&mut self) -> String {
        self.branch("BMI")
    }
    fn bpl(&mut self) -> String {
        self.branch("BPL")
    }

    fn branch(&mut self, instr: &str) -> String {
        let arg = self.read_incr_pc();
        format!("{:3} ${:04X}", instr, self.relative_addr(arg))
    }

    // Stack
    fn plp(&mut self) -> String {
        "PLP".to_string()
    }
    fn php(&mut self) -> String {
        "PHP".to_string()
    }
    fn pla(&mut self) -> String {
        "PLA".to_string()
    }
    fn pha(&mut self) -> String {
        "PHA".to_string()
    }

    // Misc
    fn nop(&mut self) -> String {
        "NOP".to_string()
    }
    fn sec(&mut self) -> String {
        "SEC".to_string()
    }
    fn clc(&mut self) -> String {
        "CLC".to_string()
    }
    fn sei(&mut self) -> String {
        "SEI".to_string()
    }
    fn sed(&mut self) -> String {
        "SED".to_string()
    }
    fn cld(&mut self) -> String {
        "CLD".to_string()
    }
    fn clv(&mut self) -> String {
        "CLV".to_string()
    }

    pub fn decode(mut self) -> Instruction {
        let opcode = self.read_incr_pc();
        let str: String = decode_opcode!(opcode, self);
        Instruction {
            bytes: self.bytes,
            str: str,
        }
    }

    fn relative_addr(&self, disp: u8) -> u16 {
        let disp = disp as i16;
        let pc = self.pc as i16;
        pc.wrapping_add(disp) as u16
    }

    fn read_incr_pc(&mut self) -> u8 {
        let val: u8 = self.cpu.mem.read(self.pc);
        self.bytes.push(val);
        self.pc = self.pc.wrapping_add(1);
        val
    }

    fn read_w_incr_pc(&mut self) -> u16 {
        ((self.read_incr_pc() as u16) << 0) | ((self.read_incr_pc() as u16) << 8)
    }
}
