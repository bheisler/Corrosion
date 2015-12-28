use memory::MemSegment;

pub struct Disassembler<'a, M: MemSegment + 'a> {
    pc: u16,
    mem: &'a mut M,
}

struct PartialInstruction {
    bytes: Vec<u8>,
    pattern: String,
}

impl PartialInstruction {
    fn finish(&self, instr: &str) -> Instruction {
        Instruction {
            bytes: self.bytes.clone(),
            str: self.pattern.replace("$$$", instr).clone(),
        }
    }
}

pub struct Instruction {
    pub bytes: Vec<u8>,
    pub str: String,
}

impl<'a, M: MemSegment> Disassembler<'a, M> {
    pub fn new(pc: u16, mem: &'a mut M) -> Disassembler<M> {
        Disassembler { pc: pc, mem: mem }
    }

    // Addressing modes
    fn absolute(&mut self, opcode: u8) -> PartialInstruction {
        let low = self.read_incr_pc();
        let high = self.read_incr_pc();
        PartialInstruction {
            bytes: vec![opcode, low, high],
            pattern: format!("$$$ {:02X}{:02X}", high, low),
        }
    }
    fn indirect(&mut self, opcode: u8) -> PartialInstruction {
        let arg = self.read_w_incr_pc();
        PartialInstruction {
            bytes: vec![opcode, (arg & 0x00FF) as u8, (arg & 0xFF00 >> 8) as u8],
            pattern: format!("$$$ ({:04X}) = {:04X}", arg, self.mem.read_w(arg)),
        }
    }

    // Instructions
    fn jmp(&mut self, instr: &mut PartialInstruction) -> Instruction {
        instr.finish("JMP")
    }

    pub fn decode(mut self) -> Instruction {
        let opcode = self.read_incr_pc();
        decode_opcode!(opcode, self)
    }

    fn read_incr_pc(&mut self) -> u8 {
        let val: u8 = self.mem.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        val
    }

    fn read_w_incr_pc(&mut self) -> u16 {
        let val: u16 = self.mem.read_w(self.pc);
        self.pc = self.pc.wrapping_add(2);
        val
    }
}
