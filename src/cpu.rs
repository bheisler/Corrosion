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
        
        0xA0 => { let mode = $this.immediate();   $this.ldy( mode ) },
        0xA4 => { let mode = $this.zero_page();   $this.ldy( mode ) },
        0xB4 => { let mode = $this.zero_page_y(); $this.ldy( mode ) },
        0xAC => { let mode = $this.absolute();    $this.ldy( mode ) },
        0xBC => { let mode = $this.absolute_y();  $this.ldy( mode ) },

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

        0x29 => { let mode = $this.immediate();   $this.and( mode ) },
        0x25 => { let mode = $this.zero_page();   $this.and( mode ) },
        0x35 => { let mode = $this.zero_page_x(); $this.and( mode ) },
        0x2D => { let mode = $this.absolute();    $this.and( mode ) },
        0x3D => { let mode = $this.absolute_x();  $this.and( mode ) },
        0x39 => { let mode = $this.absolute_y();  $this.and( mode ) },
        0x21 => { let mode = $this.indirect_x();  $this.and( mode ) },
        0x31 => { let mode = $this.indirect_y();  $this.and( mode ) },

        0x09 => { let mode = $this.immediate();   $this.ora( mode ) },
        0x05 => { let mode = $this.zero_page();   $this.ora( mode ) },
        0x15 => { let mode = $this.zero_page_x(); $this.ora( mode ) },
        0x0D => { let mode = $this.absolute();    $this.ora( mode ) },
        0x1D => { let mode = $this.absolute_x();  $this.ora( mode ) },
        0x19 => { let mode = $this.absolute_y();  $this.ora( mode ) },
        0x01 => { let mode = $this.indirect_x();  $this.ora( mode ) },
        0x11 => { let mode = $this.indirect_y();  $this.ora( mode ) },

        0x49 => { let mode = $this.immediate();   $this.eor( mode ) },
        0x45 => { let mode = $this.zero_page();   $this.eor( mode ) },
        0x55 => { let mode = $this.zero_page_x(); $this.eor( mode ) },
        0x4D => { let mode = $this.absolute();    $this.eor( mode ) },
        0x5D => { let mode = $this.absolute_x();  $this.eor( mode ) },
        0x59 => { let mode = $this.absolute_y();  $this.eor( mode ) },
        0x41 => { let mode = $this.indirect_x();  $this.eor( mode ) },
        0x51 => { let mode = $this.indirect_y();  $this.eor( mode ) },

        0x69 => { let mode = $this.immediate();   $this.adc( mode ) },
        0x65 => { let mode = $this.zero_page();   $this.adc( mode ) },
        0x75 => { let mode = $this.zero_page_x(); $this.adc( mode ) },
        0x6D => { let mode = $this.absolute();    $this.adc( mode ) },
        0x7D => { let mode = $this.absolute_x();  $this.adc( mode ) },
        0x79 => { let mode = $this.absolute_y();  $this.adc( mode ) },
        0x61 => { let mode = $this.indirect_x();  $this.adc( mode ) },
        0x71 => { let mode = $this.indirect_y();  $this.adc( mode ) },
        
        0xE9 => { let mode = $this.immediate();   $this.sbc( mode ) },
        0xE5 => { let mode = $this.zero_page();   $this.sbc( mode ) },
        0xF5 => { let mode = $this.zero_page_x(); $this.sbc( mode ) },
        0xED => { let mode = $this.absolute();    $this.sbc( mode ) },
        0xFD => { let mode = $this.absolute_x();  $this.sbc( mode ) },
        0xF9 => { let mode = $this.absolute_y();  $this.sbc( mode ) },
        0xE1 => { let mode = $this.indirect_x();  $this.sbc( mode ) },
        0xF1 => { let mode = $this.indirect_y();  $this.sbc( mode ) },

        0xC9 => { let mode = $this.immediate();   $this.cmp( mode ) },
        0xC5 => { let mode = $this.zero_page();   $this.cmp( mode ) },
        0xD5 => { let mode = $this.zero_page_x(); $this.cmp( mode ) },
        0xCD => { let mode = $this.absolute();    $this.cmp( mode ) },
        0xDD => { let mode = $this.absolute_x();  $this.cmp( mode ) },
        0xD9 => { let mode = $this.absolute_y();  $this.cmp( mode ) },
        0xC1 => { let mode = $this.indirect_x();  $this.cmp( mode ) },
        0xD1 => { let mode = $this.indirect_y();  $this.cmp( mode ) },
        
        0xE0 => { let mode = $this.immediate();   $this.cpx( mode ) },
        0xE4 => { let mode = $this.zero_page();   $this.cpx( mode ) },
        0xEC => { let mode = $this.absolute();    $this.cpx( mode ) },
        
        0xC0 => { let mode = $this.immediate();   $this.cpy( mode ) },
        0xC4 => { let mode = $this.zero_page();   $this.cpy( mode ) },
        0xCC => { let mode = $this.absolute();    $this.cpy( mode ) },
        
        0xE6 => { let mode = $this.zero_page();   $this.inc( mode ) },
        0xF6 => { let mode = $this.zero_page_x(); $this.inc( mode ) },
        0xEE => { let mode = $this.absolute();    $this.inc( mode ) },
        0xFE => { let mode = $this.absolute_x();  $this.inc( mode ) },
        
        0xE8 => $this.inx(),
        0xC8 => $this.iny(),
        
        0xC6 => { let mode = $this.zero_page();   $this.dec( mode ) },
        0xD6 => { let mode = $this.zero_page_x(); $this.dec( mode ) },
        0xCE => { let mode = $this.absolute();    $this.dec( mode ) },
        0xDE => { let mode = $this.absolute_x();  $this.dec( mode ) },
        
        0xCA => $this.dex(),
        0x88 => $this.dey(),

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

        //Stack
        0x28 => $this.plp(),
        0x08 => $this.php(),
        0x68 => $this.pla(),
        0x48 => $this.pha(),

        //Misc
        0xEA => $this.nop(),
        0x38 => $this.sec(),
        0x18 => $this.clc(),
        0x78 => $this.sei(),
        0xF8 => $this.sed(),
        0xD8 => $this.cld(),
        0xB8 => $this.clv(),
        0xAA => $this.tax(),
        0xA8 => $this.tay(),

        //Else
        x => panic!( "Unknown or unsupported opcode: {:02X}", x ),
    } }
}

use memory::CpuMemory;
use memory::MemSegment;
use disasm::Disassembler;

trait AddressingMode : Copy {
    fn read(self, cpu: &mut CPU) -> u8;
    fn write(self, cpu: &mut CPU, val: u8);
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
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

// TODO: Make this a feature?
const DUMP_STACK: bool = false;

impl CPU {
    fn trace(&mut self) {
        if DUMP_STACK {
            println!{
                "Stack: {:>30}",
                (self.sp..0xFF)
                    .map(|idx| self.mem.read(0x0100 + idx as u16))
                    .map(|byte| format!("{:02X}", byte))
                    .fold("".to_string(), |left, right| left + " " + &right )
            }
        }
        
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
    fn sta<M: AddressingMode>(&mut self, mode: M) {
        let val = self.a;
        mode.write(self, val);
    }

    // Loads
    fn ldx<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        self.x = self.set_sign_zero(arg);
    }
    fn lda<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        self.a = self.set_sign_zero(arg);
    }
    fn ldy<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        self.y = self.set_sign_zero(arg);
    }

    // Logic/Math operations
    fn bit<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        self.set_sign(arg);
        let ac = self.a;
        self.set_zero(arg & ac);
        self.set_overflow((arg & 0x40) != 0);
    }
    fn and<M: AddressingMode>(&mut self, mode: M) {
        let ac = self.a & mode.read(self);
        self.a = self.set_sign_zero(ac);
    }
    fn ora<M: AddressingMode>(&mut self, mode: M) {
        let ac = self.a | mode.read(self);
        self.a = self.set_sign_zero(ac);
    }
    fn eor<M: AddressingMode>(&mut self, mode: M) {
        let ac = self.a ^ mode.read(self);
        self.a = self.set_sign_zero(ac);
    }
    fn adc<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        self.do_adc(arg);
    }
    fn sbc<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        self.do_adc(!arg);
    }
    fn cmp<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        let ac = self.a;
        self.set_carry(!(ac < arg));
        let res = ac.wrapping_sub(arg);
        self.set_sign_zero(res);
    }
    fn cpx<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        let x = self.x;
        self.set_carry(!(x < arg));
        let res = x.wrapping_sub(arg);
        self.set_sign_zero(res);
    }
    fn cpy<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        let y = self.y;
        self.set_carry(!(y < arg));
        let res = y.wrapping_sub(arg);
        self.set_sign_zero(res);
    }
    fn inc<M : AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        let res = self.set_sign_zero(arg.wrapping_add(1));
        mode.write(self, res);
    }
    fn inx(&mut self) {
        let res = self.x.wrapping_add(1);
        self.x = self.set_sign_zero(res);
    }
    fn iny(&mut self) {
        let res = self.y.wrapping_add(1);
        self.y = self.set_sign_zero(res);
    }
    fn dec<M : AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        let res = self.set_sign_zero(arg.wrapping_sub(1));
        mode.write(self, res);
    }
    fn dex(&mut self) {
        let res = self.x.wrapping_sub(1);
        self.x = self.set_sign_zero(res);
    }
    fn dey(&mut self) {
        let res = self.y.wrapping_sub(1);
        self.y = self.set_sign_zero(res);
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

    // Stack
    fn plp(&mut self) {
        let p = self.stack_pop();
        self.p = Status::from_bits_truncate(p);
        self.p.remove(B);
        self.p.insert(U);
    }
    fn php(&mut self) {
        let p = self.p;
        self.stack_push(p.bits() | 0b0011_0000);
    }
    fn pla(&mut self) {
        let val = self.stack_pop();
        self.a = self.set_sign_zero(val);
    }
    fn pha(&mut self) {
        let a = self.a;
        self.stack_push(a);
    }

    // Misc
    fn nop(&mut self) {}
    fn sec(&mut self) {
        self.p.insert(C);
    }
    fn clc(&mut self) {
        self.p.remove(C);
    }
    fn sei(&mut self) {
        self.p.insert(I);
    }
    fn sed(&mut self) {
        self.p.insert(D);
    }
    fn cld(&mut self) {
        self.p.remove(D);
    }
    fn clv(&mut self) {
        self.p.remove(V);
    }
    fn tax(&mut self) {
        let res = self.a;
        self.x = self.set_sign_zero(res);
    }
    fn tay(&mut self) {
        let res = self.a;
        self.y = self.set_sign_zero(res);
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
    
    fn set_sign_zero(&mut self, arg: u8) -> u8 {
        self.set_sign( arg );
        self.set_zero( arg );
        arg
    }

    fn set_overflow(&mut self, arg: bool) {
        if arg {
            self.p.insert(V);
        } else {
            self.p.remove(V);
        }
    }

    fn set_carry(&mut self, arg: bool) {
        if arg {
            self.p.insert(C);
        } else {
            self.p.remove(C);
        }
    }

    fn relative_addr(&self, disp: u8) -> u16 {
        let disp = disp as i16;
        let pc = self.pc as i16;
        pc.wrapping_add(disp) as u16
    }

    fn do_adc(&mut self, arg: u8) {
        let mut result = self.a as u16 + arg as u16;
        if self.p.contains(C) {
            result += 1;
        }

        self.set_carry(result > 0xFF);

        let result = result as u8;
        let a = self.a;
        self.set_overflow((a ^ arg) & 0x80 == 0 && (a ^ result) & 0x80 == 0x80);
        self.a = self.set_sign_zero(result);
    }

    fn branch(&mut self, cond: bool) {
        let arg = self.load_incr_pc();
        if cond {
            self.pc = self.relative_addr(arg);
        }
    }

    fn stack_push(&mut self, val: u8) {
        self.sp = self.sp.wrapping_sub(1);
        self.mem.write(self.sp as u16 + 0x0101, val);
    }
    fn stack_push_w(&mut self, val: u16) {
        self.sp = self.sp.wrapping_sub(2);
        self.mem.write_w(self.sp as u16 + 0x0101, val);
    }
    fn stack_pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.mem.read(self.sp as u16 + 0x0100)
    }
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
