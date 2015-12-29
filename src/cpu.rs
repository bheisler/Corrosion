#![macro_use]



macro_rules! decode_opcode {
    ($opcode:expr, $this:expr) => { match $opcode {
		//LDX
		0xA2 => { let mode = $this.immediate(); $this.ldx( mode ) },
		0xA6 => { let mode = $this.zero_page(); $this.ldx( mode ) },
		0xB6 => { let mode = $this.zero_page_y(); $this.ldx( mode ) },
		0xAE => { let mode = $this.absolute(); $this.ldx( mode ) },
		0xBE => { let mode = $this.absolute_y(); $this.ldx( mode ) },
            
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
    fn read(self, cpu: &mut CPU) -> u8;
    fn write(self, cpu: &mut CPU, val: u8);
}

struct ImmediateAddressingMode;
impl AddressingMode for ImmediateAddressingMode {
    fn read(self, cpu: &mut CPU) -> u8 {
        cpu.load_incr_pc()
    }
    #[allow(unused_variables)]
    fn write(self, cpu: &mut CPU, val: u8) {
        panic!("Tried to write {:0X} to an immediate address.", val)
    }
}

struct MemoryAddressingMode {
    ptr: u16,
}
impl AddressingMode for MemoryAddressingMode {
    fn read(self, cpu: &mut CPU) -> u8 {
        cpu.read(self.ptr)
    }
    fn write(self, cpu: &mut CPU, val: u8) {
        cpu.write(self.ptr, val)
    }
}

pub struct CPU {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub s: u8,
    pub pc: u16,

    pub mem: CpuMemory,
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
        let opcode = Disassembler::new(self).decode();
        println!(
            "{:X} {:9}  {:30}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:3} SL:{:3}",
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
    fn absolute(&mut self) -> MemoryAddressingMode {
        MemoryAddressingMode { ptr: self.load_w_incr_pc() }
    }
    fn absolute_x(&mut self) -> MemoryAddressingMode {
        MemoryAddressingMode { ptr: self.load_w_incr_pc().wrapping_add(self.x as u16) }
    }
    fn absolute_y(&mut self) -> MemoryAddressingMode {
        MemoryAddressingMode { ptr: self.load_w_incr_pc().wrapping_add(self.y as u16) }
    }
    fn zero_page(&mut self) -> MemoryAddressingMode {
        MemoryAddressingMode { ptr: self.load_incr_pc() as u16 }
    }
    fn zero_page_x(&mut self) -> MemoryAddressingMode {
        MemoryAddressingMode { ptr: self.load_incr_pc().wrapping_add(self.x) as u16 }
    }
    fn zero_page_y(&mut self) -> MemoryAddressingMode {
        MemoryAddressingMode { ptr: self.load_incr_pc().wrapping_add(self.y) as u16 }
    }

    // Instructions
    fn ldx<M: AddressingMode>(&mut self, mode: M) {
        self.x = mode.read(self);
    }

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
        self.trace();
        let opcode: u8 = self.load_incr_pc();
        decode_opcode!(opcode, self);
    }
}
