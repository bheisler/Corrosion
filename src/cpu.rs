#![macro_use]

macro_rules! decode_opcode {
    ($opcode:expr, $this:expr) => { match $opcode {
        //Stores
        0x86 => { let mode = $this.zero_page();   $this.stx( mode ) },
        0x96 => { let mode = $this.zero_page_y(); $this.stx( mode ) },
        0x8E => { let mode = $this.absolute();    $this.stx( mode ) },

        //Loads
        0xA2 => { let mode = $this.immediate();   $this.ldx( mode ) },
        0xA6 => { let mode = $this.zero_page();   $this.ldx( mode ) },
        0xB6 => { let mode = $this.zero_page_y(); $this.ldx( mode ) },
        0xAE => { let mode = $this.absolute();    $this.ldx( mode ) },
        0xBE => { let mode = $this.absolute_y();  $this.ldx( mode ) },

        0xA9 => { let mode = $this.immediate();   $this.lda( mode ) },
        0xA5 => { let mode = $this.zero_page();   $this.lda( mode ) },
        0xB5 => { let mode = $this.zero_page_x(); $this.lda( mode ) },
        0xAD => { let mode = $this.absolute();    $this.lda( mode ) },
        0xBD => { let mode = $this.absolute_x();  $this.lda( mode ) },
        0xB9 => { let mode = $this.absolute_y();  $this.lda( mode ) },
        0xA1 => { let mode = $this.indirect_x();  $this.lda( mode ) },
        0xB1 => { let mode = $this.indirect_y();  $this.lda( mode ) },

        //Jumps
        0x4C => $this.jmp(),
        0x6C => $this.jmpi(),
        0x20 => $this.jsr(),

        //Branches
        0x90 => $this.bcc(),
        0xB0 => $this.bcs(),

        //Misc
        0xEA => $this.nop(),
        0x38 => $this.sec(),
        0x18 => $this.clc(),

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

bitflags! {
    flags Status : u8 {
        const C = 0b0000_0001, //Carry flag
        const Z = 0b0000_0010, //Zero flag
        const I = 0b0000_0100, //Enable Interrupts
        const D = 0b0000_1000, //Enable BCD mode
        const B = 0b0001_0000, //BRK
        const U = 0b0010_0000, //Unused, should always be 1
        const V = 0b0100_0000, //Overflow
        const S = 0b1000_0000, //Sign
    }
}

impl Status {
    fn init() -> Status {
        I | U
    }
}

pub struct CPU {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: Status,
    pub sp: u8,
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
            self.p.bits(),
            self.sp,
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
    fn indirect_x(&mut self) -> MemoryAddressingMode {
        let arg = self.load_incr_pc();
        let zp_idx = arg.wrapping_add(self.x);
        MemoryAddressingMode { ptr: self.mem.read_w(zp_idx as u16) }
    }
    fn indirect_y(&mut self) -> MemoryAddressingMode {
        let arg = self.load_incr_pc();
        let ptr_base = self.mem.read_w(arg as u16);
        let ptr = ptr_base.wrapping_add(self.y as u16);
        MemoryAddressingMode { ptr: ptr }
    }

    // Instructions
    // Stores
    fn stx<M: AddressingMode>(&mut self, mode: M) {
        let val = self.x;
        mode.write(self, val);
    }

    // Loads
    fn ldx<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        self.set_sign(arg);
        self.set_zero(arg);
        self.x = arg;
    }
    fn lda<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        self.set_sign(arg);
        self.set_zero(arg);
        self.a = arg;
    }

    // Jumps
    fn jmp(&mut self) {
        self.pc = self.load_w_incr_pc();
    }
    fn jmpi(&mut self) {
        let arg = self.load_w_incr_pc();
        self.pc = self.mem.read_w(arg);
    }
    fn jsr(&mut self) {
        let old_pc = self.pc - 1;
        self.stack_push(((old_pc >> 8) & 0xFF) as u8);
        self.stack_push(((old_pc >> 0) & 0xFF) as u8);
        self.pc = self.load_w_incr_pc();
    }

    // Branches
    fn bcc(&mut self) {
        let arg = self.load_incr_pc();
        if !self.p.contains(C) {
            self.pc = self.relative_addr(arg);
        }
    }
    fn bcs(&mut self) {
        let arg = self.load_incr_pc();
        if self.p.contains(C) {
            self.pc = self.relative_addr(arg);
        }
    }

    // Misc
    fn nop(&mut self) {}
    fn sec(&mut self) {
        self.p.insert(C);
    }
    fn clc(&mut self) {
        self.p.remove(C);
    }

    pub fn new(mem: CpuMemory) -> CPU {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            p: Status::init(),
            sp: 0xFD,
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

    fn set_sign(&mut self, arg: u8) {
        if (arg & 0b1000_0000 == 0) {
            self.p.remove(S);
        } else {
            self.p.insert(S);
        }
    }

    fn set_zero(&mut self, arg: u8) {
        if arg == 0 {
            self.p.insert(Z);
        } else {
            self.p.remove(Z);
        }
    }

    fn relative_addr(&self, disp: u8) -> u16 {
        let disp = disp as i16;
        let pc = self.pc as i16;
        pc.wrapping_add(disp) as u16
    }

    fn stack_push(&mut self, val: u8) {
        self.mem.write(self.sp as u16 + 0x0100, val);
        self.sp = self.sp - 1;
    }

    pub fn step(&mut self) {
        self.trace();
        let opcode: u8 = self.load_incr_pc();
        decode_opcode!(opcode, self);
    }
}
