#![macro_use]



macro_rules! decode_opcode {
    ($opcode:expr, $this:expr) => { match $opcode {
        //JMP
        0x4C => $this.jmp(),
        0x6C => $this.jmpi(),
        //Else
        x => panic!( "Unknown or unsupported opcode: {:02X}", x ),
    } }
}

use memory::CpuMemory;
use memory::MemSegment;
use disasm::Disassembler;

trait AddressingMode {
    fn read(&mut self, cpu: &mut CPU) -> u8;
    fn write(&mut self, cpu: &mut CPU, val: u8);
}

struct ImmediateAddressingMode;
impl AddressingMode for ImmediateAddressingMode {
    fn read(&mut self, cpu: &mut CPU) -> u8 {
        cpu.load_incr_pc()
    }
    #[allow(unused_variables)]
    fn write(&mut self, cpu: &mut CPU, val: u8) {
        panic!("Tried to write {:0X} to an immediate address.", val)
    }
}

struct MemoryAddressingMode {
    ptr: u16,
}
impl AddressingMode for MemoryAddressingMode {
    fn read(&mut self, cpu: &mut CPU) -> u8 {
        let val = cpu.read(self.ptr);
        self.ptr = self.ptr.wrapping_add(1);
        val
    }
    fn write(&mut self, cpu: &mut CPU, val: u8) {
        cpu.write(self.ptr, val);
        self.ptr = self.ptr + 1;
    }
}

pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    s: u8,
    pc: u16,

    mem: CpuMemory,
}

impl MemSegment for CPU {
    fn read(&mut self, idx: u16) -> u8 {
        self.mem.read(idx)
    }
    fn write(&mut self, idx: u16, val: u8) {
        self.mem.write(idx, val)
    }
}

impl CPU {
    fn trace(&mut self) {
        let disasm = Disassembler::new(self.pc, &mut self.mem);
        let opcode = disasm.decode();
        println!(
            "{:X} {:8}  {:30}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:3} SL:{:3}",
            self.pc,
            opcode.bytes.iter()
            	.map(|byte| format!("{:02X}", byte))
            	.fold("".to_string(), |left, right| left + " " + &right ),
            opcode.str,
            self.a,
            self.x,
            self.y,
            self.p,
            self.s,
            0, //TODO: Add cycle counting
            0, //TODO: Add scanline counting
        );
    }

    // Addressing modes
    fn immediate(&mut self) -> ImmediateAddressingMode {
        ImmediateAddressingMode
    }

    // Instructions
    fn jmp(&mut self) {
        self.pc = self.load_w_incr_pc();
    }
    fn jmpi(&mut self) {
        let arg = self.load_w_incr_pc();
        self.pc = self.mem.read_w(arg);
    }

    pub fn new(mem: CpuMemory) -> CPU {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            p: 0x24,
            s: 0xFD,
            pc: 0,

            mem: mem,
        }
    }

    pub fn init(&mut self) {
        // self.pc = self.mem.read_w(0xFFFC);
        self.pc = 0xC000;
    }

    fn load_incr_pc(&mut self) -> u8 {
        let res = self.mem.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        res
    }

    fn load_w_incr_pc(&mut self) -> u16 {
        let res = self.mem.read_w(self.pc);
        self.pc = self.pc.wrapping_add(2);
        res
    }

    pub fn step(&mut self) {
        println!("{:04X}", self.pc);
        self.trace();
        let opcode: u8 = self.load_incr_pc();
        decode_opcode!(opcode, self);
    }
}
