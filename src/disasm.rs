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
            pattern: format!("$$$ #${:02X} = {:02X}", arg, self.cpu.mem.read(arg as u16)),
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

    // Instructions
    fn ldx(&mut self, instr: PartialInstruction) -> String {
        instr.finish("LDX")
    }

    fn jmp(&mut self) -> String {
        format!("JMP ${:04X}", self.read_w_incr_pc())
    }
    fn jmpi(&mut self) -> String {
        let arg = self.read_w_incr_pc();
        format!("JMP ({:04X}) = {:04X}", arg, self.cpu.mem.read_w(arg))
    }

    pub fn decode(mut self) -> Instruction {
        let opcode = self.read_incr_pc();
        let str: String = decode_opcode!(opcode, self);
        Instruction {
            bytes: self.bytes,
            str: str,
        }
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
