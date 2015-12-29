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

    // Instructions
    fn jmp(&mut self) -> String {
        format!( "JMP {:04X}", self.read_w_incr_pc() )
    }
	fn jmpi(&mut self) -> String {
	    let arg = self.read_w_incr_pc(); 
	    format!("JMP ({:04X}) = {:04X}", arg, self.mem.read_w(arg))
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
