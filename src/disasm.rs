use memory::MemSegment;

pub struct Disassembler<'a, M: MemSegment + 'a> {
    pc: u16,
    mem: &'a mut M,
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

impl<'a, M: MemSegment> Disassembler<'a, M> {
    pub fn new(pc: u16, mem: &'a mut M) -> Disassembler<M> {
        Disassembler { pc: pc, mem: mem, bytes: vec![] }
    }

    // Addressing modes
    fn absolute(&mut self) -> PartialInstruction {
        let low = self.read_incr_pc();
        let high = self.read_incr_pc();
        PartialInstruction {
            pattern: format!("$$$ {:02X}{:02X}", high, low),
        }
    }
    fn indirect(&mut self) -> PartialInstruction {
        let arg = self.read_w_incr_pc();
        PartialInstruction {
            pattern: format!("$$$ ({:04X}) = {:04X}", arg, self.mem.read_w(arg)),
        }
    }

    // Instructions
    fn jmp(&mut self, instr: PartialInstruction) -> String {
        instr.finish("JMP")
    }

    pub fn decode(mut self) -> Instruction {
        let opcode = self.read_incr_pc();
        let str : String = decode_opcode!(opcode, self);
        Instruction {
            bytes: self.bytes,
            str: str,
        }
    }

    fn read_incr_pc(&mut self) -> u8 {
        let val: u8 = self.mem.read(self.pc);
        self.bytes.push(val);
        self.pc = self.pc.wrapping_add(1);
        val
    }

    fn read_w_incr_pc(&mut self) -> u16 {
        ((self.read_incr_pc() as u16) << 0) | ((self.read_incr_pc() as u16) << 8)
    }
}
