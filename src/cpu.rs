#![macro_use]

macro_rules! decode_opcode {
    ($opcode:expr, $this:expr) => { match $opcode {
        //Stores
        0x86 => { let mode = $this.zero_page();   $this.stx( mode ) },
        0x96 => { let mode = $this.zero_page_y(); $this.stx( mode ) },
        0x8E => { let mode = $this.absolute();    $this.stx( mode ) },

        0x85 => { let mode = $this.zero_page();   $this.sta( mode ) },
        0x95 => { let mode = $this.zero_page_x(); $this.sta( mode ) },
        0x8D => { let mode = $this.absolute();    $this.sta( mode ) },
        0x9D => { let mode = $this.absolute_x();  $this.sta( mode ) },
        0x99 => { let mode = $this.absolute_y();  $this.sta( mode ) },
        0x81 => { let mode = $this.indirect_x();  $this.sta( mode ) },
        0x91 => { let mode = $this.indirect_y();  $this.sta( mode ) },

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

		//Logic/math operations
		0x24 => { let mode = $this.zero_page();   $this.bit( mode ) },
		0x2C => { let mode = $this.absolute();    $this.bit( mode ) },

        //Jumps
        0x4C => $this.jmp(),
        0x6C => $this.jmpi(),
        0x20 => $this.jsr(),
        0x60 => $this.rts(),

        //Branches
        0xB0 => $this.bcs(),
        0x90 => $this.bcc(),
        0xF0 => $this.beq(),
        0xD0 => $this.bne(),
        0x70 => $this.bvs(),
        0x50 => $this.bvc(),
        0x30 => $this.bmi(),
        0x10 => $this.bpl(),

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

        // println!{
        // "Stack: {:>30}",
        // (self.sp..0xFF)
        // .map(|idx| self.mem.read(0x0100 + idx as u16))
        // .map(|byte| format!("{:02X}", byte))
        // .fold("".to_string(), |left, right| left + " " + &right )
        // }
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
    fn sta<M: AddressingMode>(&mut self, mode: M) {
        let val = self.a;
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

    // Logic/Math operations
    fn bit<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        self.set_sign(arg);
        let ac = self.a;
        self.set_zero(arg & ac);
        self.set_overflow((arg & 0x40) != 0);
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
        let target = self.load_w_incr_pc();
        let return_addr = self.pc - 1;
        self.pc = target;
        self.stack_push_w(return_addr);
    }
    fn rts(&mut self) {
        self.pc = self.stack_pop_w().wrapping_add(1);
    }


    // Branches
    fn bcs(&mut self) {
        let cond = self.p.contains(C);
        self.branch(cond);
    }
    fn bcc(&mut self) {
        let cond = !self.p.contains(C);
        self.branch(cond);
    }
    fn beq(&mut self) {
        let cond = self.p.contains(Z);
        self.branch(cond);
    }
    fn bne(&mut self) {
        let cond = !self.p.contains(Z);
        self.branch(cond);
    }
    fn bvs(&mut self) {
        let cond = self.p.contains(V);
        self.branch(cond);
    }
    fn bvc(&mut self) {
        let cond = !self.p.contains(V);
        self.branch(cond);
    }
    fn bmi(&mut self) {
        let cond = self.p.contains(S);
        self.branch(cond);
    }
    fn bpl(&mut self) {
        let cond = !self.p.contains(S);
        self.branch(cond);
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
        if arg & 0b1000_0000 == 0 {
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

    fn set_overflow(&mut self, arg: bool) {
        if arg {
            self.p.insert(V);
        } else {
            self.p.remove(V);
        }
    }

    fn relative_addr(&self, disp: u8) -> u16 {
        let disp = disp as i16;
        let pc = self.pc as i16;
        pc.wrapping_add(disp) as u16
    }

    fn branch(&mut self, cond: bool) {
        let arg = self.load_incr_pc();
        if cond {
            self.pc = self.relative_addr(arg);
        }
    }

    //    fn stack_push(&mut self, val: u8) {
    //        self.sp = self.sp - 1;
    //        self.mem.write(self.sp as u16 + 0x0101, val);
    //    }
    fn stack_push_w(&mut self, val: u16) {
        self.sp = self.sp - 2;
        self.mem.write_w(self.sp as u16 + 0x0101, val);
    }
    //    fn stack_pop(&mut self) -> u8 {
    //        self.sp = self.sp.wrapping_add(1);
    //        self.mem.read(self.sp as u16 + 0x0100)
    //    }
    fn stack_pop_w(&mut self) -> u16 {
        self.sp = self.sp.wrapping_add(2);
        self.mem.read_w(self.sp as u16 + 0x00FF)
    }

    pub fn step(&mut self) {
        self.trace();
        let opcode: u8 = self.load_incr_pc();
        decode_opcode!(opcode, self);
    }
}
