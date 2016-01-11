#![macro_use]

macro_rules! decode_opcode {
    ($opcode:expr, $this:expr) => { match $opcode {
        //Stores
        0x86 => { let mode = $this.zero_page();   $this.stx( mode ) },
        0x96 => { let mode = $this.zero_page_y(); $this.stx( mode ) },
        0x8E => { let mode = $this.absolute();    $this.stx( mode ) },

        0x84 => { let mode = $this.zero_page();   $this.sty( mode ) },
        0x94 => { let mode = $this.zero_page_x(); $this.sty( mode ) },
        0x8C => { let mode = $this.absolute();    $this.sty( mode ) },

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
        0xB4 => { let mode = $this.zero_page_x(); $this.ldy( mode ) },
        0xAC => { let mode = $this.absolute();    $this.ldy( mode ) },
        0xBC => { let mode = $this.absolute_x();  $this.ldy( mode ) },

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

        0x4A => { let mode = $this.accumulator(); $this.lsr( mode ) },
        0x46 => { let mode = $this.zero_page();   $this.lsr( mode ) },
        0x56 => { let mode = $this.zero_page_x(); $this.lsr( mode ) },
        0x4E => { let mode = $this.absolute();    $this.lsr( mode ) },
        0x5E => { let mode = $this.absolute_x();  $this.lsr( mode ) },

        0x0A => { let mode = $this.accumulator(); $this.asl( mode ) },
        0x06 => { let mode = $this.zero_page();   $this.asl( mode ) },
        0x16 => { let mode = $this.zero_page_x(); $this.asl( mode ) },
        0x0E => { let mode = $this.absolute();    $this.asl( mode ) },
        0x1E => { let mode = $this.absolute_x();  $this.asl( mode ) },

        0x6A => { let mode = $this.accumulator(); $this.ror( mode ) },
        0x66 => { let mode = $this.zero_page();   $this.ror( mode ) },
        0x76 => { let mode = $this.zero_page_x(); $this.ror( mode ) },
        0x6E => { let mode = $this.absolute();    $this.ror( mode ) },
        0x7E => { let mode = $this.absolute_x();  $this.ror( mode ) },

        0x2A => { let mode = $this.accumulator(); $this.rol( mode ) },
        0x26 => { let mode = $this.zero_page();   $this.rol( mode ) },
        0x36 => { let mode = $this.zero_page_x(); $this.rol( mode ) },
        0x2E => { let mode = $this.absolute();    $this.rol( mode ) },
        0x3E => { let mode = $this.absolute_x();  $this.rol( mode ) },

        //Jumps
        0x4C => $this.jmp(),
        0x6C => $this.jmpi(),
        0x20 => $this.jsr(),
        0x60 => $this.rts(),
        0x40 => $this.rti(),
        0x00 => $this.brk(),

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
        0xBA => $this.tsx(),
        0x8A => $this.txa(),
        0x9A => $this.txs(),
        0x98 => $this.tya(),

        //Unofficial NOPs
        0x04 | 0x44 | 0x64 => { $this.unofficial( ); let mode = $this.zero_page(); $this.u_nop(mode) }
        0x0C => { $this.unofficial( ); let mode = $this.absolute(); $this.u_nop(mode) }
        0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => { $this.unofficial( ); let mode = $this.zero_page_x(); $this.u_nop(mode) }
        0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => { $this.unofficial( ); $this.nop() }
        0x80 => { $this.unofficial( ); let mode = $this.immediate(); $this.u_nop(mode) }
        0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => { $this.unofficial( ); let mode = $this.absolute_x(); $this.u_nop(mode) }

        0xA3 => { $this.unofficial(); let mode = $this.indirect_x();  $this.lax(mode) }
        0xB3 => { $this.unofficial(); let mode = $this.indirect_y();  $this.lax(mode) }
        0xA7 => { $this.unofficial(); let mode = $this.zero_page();   $this.lax(mode) }
        0xB7 => { $this.unofficial(); let mode = $this.zero_page_y(); $this.lax(mode) }
        0xAF => { $this.unofficial(); let mode = $this.absolute();    $this.lax(mode) }
        0xBF => { $this.unofficial(); let mode = $this.absolute_y();  $this.lax(mode) }

        0x87 => { $this.unofficial(); let mode = $this.zero_page();   $this.sax(mode) }
        0x97 => { $this.unofficial(); let mode = $this.zero_page_y(); $this.sax(mode) }
        0x83 => { $this.unofficial(); let mode = $this.indirect_x();  $this.sax(mode) }
        0x8F => { $this.unofficial(); let mode = $this.absolute();    $this.sax(mode) }

        0xEB => { $this.unofficial(); let mode = $this.immediate();   $this.sbc(mode) }

        0xC7 => { $this.unofficial(); let mode = $this.zero_page();   $this.dcp(mode) }
        0xD7 => { $this.unofficial(); let mode = $this.zero_page_x(); $this.dcp(mode) }
        0xC3 => { $this.unofficial(); let mode = $this.indirect_x();  $this.dcp(mode) }
        0xD3 => { $this.unofficial(); let mode = $this.indirect_y();  $this.dcp(mode) }
        0xCF => { $this.unofficial(); let mode = $this.absolute();    $this.dcp(mode) }
        0xDF => { $this.unofficial(); let mode = $this.absolute_x();  $this.dcp(mode) }
        0xDB => { $this.unofficial(); let mode = $this.absolute_y();  $this.dcp(mode) }

        0xE7 => { $this.unofficial(); let mode = $this.zero_page();   $this.isc(mode) }
        0xF7 => { $this.unofficial(); let mode = $this.zero_page_x(); $this.isc(mode) }
        0xE3 => { $this.unofficial(); let mode = $this.indirect_x();  $this.isc(mode) }
        0xF3 => { $this.unofficial(); let mode = $this.indirect_y();  $this.isc(mode) }
        0xEF => { $this.unofficial(); let mode = $this.absolute();    $this.isc(mode) }
        0xFF => { $this.unofficial(); let mode = $this.absolute_x();  $this.isc(mode) }
        0xFB => { $this.unofficial(); let mode = $this.absolute_y();  $this.isc(mode) }

        0x07 => { $this.unofficial(); let mode = $this.zero_page();   $this.slo(mode) }
        0x17 => { $this.unofficial(); let mode = $this.zero_page_x(); $this.slo(mode) }
        0x03 => { $this.unofficial(); let mode = $this.indirect_x();  $this.slo(mode) }
        0x13 => { $this.unofficial(); let mode = $this.indirect_y();  $this.slo(mode) }
        0x0F => { $this.unofficial(); let mode = $this.absolute();    $this.slo(mode) }
        0x1F => { $this.unofficial(); let mode = $this.absolute_x();  $this.slo(mode) }
        0x1B => { $this.unofficial(); let mode = $this.absolute_y();  $this.slo(mode) }

        0x27 => { $this.unofficial(); let mode = $this.zero_page();   $this.rla(mode) }
        0x37 => { $this.unofficial(); let mode = $this.zero_page_x(); $this.rla(mode) }
        0x23 => { $this.unofficial(); let mode = $this.indirect_x();  $this.rla(mode) }
        0x33 => { $this.unofficial(); let mode = $this.indirect_y();  $this.rla(mode) }
        0x2F => { $this.unofficial(); let mode = $this.absolute();    $this.rla(mode) }
        0x3F => { $this.unofficial(); let mode = $this.absolute_x();  $this.rla(mode) }
        0x3B => { $this.unofficial(); let mode = $this.absolute_y();  $this.rla(mode) }

        0x47 => { $this.unofficial(); let mode = $this.zero_page();   $this.sre(mode) }
        0x57 => { $this.unofficial(); let mode = $this.zero_page_x(); $this.sre(mode) }
        0x43 => { $this.unofficial(); let mode = $this.indirect_x();  $this.sre(mode) }
        0x53 => { $this.unofficial(); let mode = $this.indirect_y();  $this.sre(mode) }
        0x4F => { $this.unofficial(); let mode = $this.absolute();    $this.sre(mode) }
        0x5F => { $this.unofficial(); let mode = $this.absolute_x();  $this.sre(mode) }
        0x5B => { $this.unofficial(); let mode = $this.absolute_y();  $this.sre(mode) }

        0x67 => { $this.unofficial(); let mode = $this.zero_page();   $this.rra(mode) }
        0x77 => { $this.unofficial(); let mode = $this.zero_page_x(); $this.rra(mode) }
        0x63 => { $this.unofficial(); let mode = $this.indirect_x();  $this.rra(mode) }
        0x73 => { $this.unofficial(); let mode = $this.indirect_y();  $this.rra(mode) }
        0x6F => { $this.unofficial(); let mode = $this.absolute();    $this.rra(mode) }
        0x7F => { $this.unofficial(); let mode = $this.absolute_x();  $this.rra(mode) }
        0x7B => { $this.unofficial(); let mode = $this.absolute_y();  $this.rra(mode) }

        0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x82 | 0x92 | 0xB2 | 0xD2 | 0xF2 => $this.kil(),

        //Else
        x => panic!( "Unknown or unsupported opcode: {:02X}", x ),
    } }
}

use memory::CpuMemory;
use memory::MemSegment;

#[cfg(feature="cputrace")]
use disasm::Disassembler;

/// The number of cycles that each machine operation takes. Indexed by opcode
/// number.
/// Copied from FCEU & SprocketNES.
#[cfg_attr(rustfmt, rustfmt_skip)]
static CYCLE_TABLE: [u8; 256] = [
    /*0x00*/ 7,6,2,8,3,3,5,5,3,2,2,2,4,4,6,6,
    /*0x10*/ 2,5,2,8,4,4,6,6,2,4,2,7,4,4,7,7,
    /*0x20*/ 6,6,2,8,3,3,5,5,4,2,2,2,4,4,6,6,
    /*0x30*/ 2,5,2,8,4,4,6,6,2,4,2,7,4,4,7,7,
    /*0x40*/ 6,6,2,8,3,3,5,5,3,2,2,2,3,4,6,6,
    /*0x50*/ 2,5,2,8,4,4,6,6,2,4,2,7,4,4,7,7,
    /*0x60*/ 6,6,2,8,3,3,5,5,4,2,2,2,5,4,6,6,
    /*0x70*/ 2,5,2,8,4,4,6,6,2,4,2,7,4,4,7,7,
    /*0x80*/ 2,6,2,6,3,3,3,3,2,2,2,2,4,4,4,4,
    /*0x90*/ 2,6,2,6,4,4,4,4,2,5,2,5,5,5,5,5,
    /*0xA0*/ 2,6,2,6,3,3,3,3,2,2,2,2,4,4,4,4,
    /*0xB0*/ 2,5,2,5,4,4,4,4,2,4,2,4,4,4,4,4,
    /*0xC0*/ 2,6,2,8,3,3,5,5,2,2,2,2,4,4,6,6,
    /*0xD0*/ 2,5,2,8,4,4,6,6,2,4,2,7,4,4,7,7,
    /*0xE0*/ 2,6,3,8,3,3,5,5,2,2,2,2,4,4,6,6,
    /*0xF0*/ 2,5,2,8,4,4,6,6,2,4,2,7,4,4,7,7,
];


trait AddressingMode : Copy {
    fn read(self, cpu: &mut CPU) -> u8;
    fn write(self, cpu: &mut CPU, val: u8);
    fn tick_cycle(self, cpu: &mut CPU);

    // The double-instructions all have their oops cycle built in to the count
    // but they contain an instruction that ticks separately, so this is the
    // easiest way to counter that.
    fn untick_cycle(self, cpu: &mut CPU);
}

#[derive(Debug, Copy, Clone)]
struct ImmediateAddressingMode;
impl AddressingMode for ImmediateAddressingMode {
    fn read(self, cpu: &mut CPU) -> u8 {
        cpu.load_incr_pc()
    }
    fn write(self, _: &mut CPU, val: u8) {
        panic!("Tried to write {:02X} to an immediate address.", val)
    }
    fn tick_cycle(self, _: &mut CPU) {}
    fn untick_cycle(self, _: &mut CPU) {}
}

#[derive(Debug, Copy, Clone)]
struct AccumulatorAddressingMode;
impl AddressingMode for AccumulatorAddressingMode {
    fn read(self, cpu: &mut CPU) -> u8 {
        cpu.regs.a
    }
    fn write(self, cpu: &mut CPU, val: u8) {
        cpu.regs.a = val
    }
    fn tick_cycle(self, _: &mut CPU) {}
    fn untick_cycle(self, _: &mut CPU) {}
}

#[derive(Debug, Copy, Clone)]
struct MemoryAddressingMode {
    ptr_base: u16,
    ptr: u16,
}
impl AddressingMode for MemoryAddressingMode {
    fn read(self, cpu: &mut CPU) -> u8 {
        cpu.read(self.ptr)
    }
    fn write(self, cpu: &mut CPU, val: u8) {
        cpu.write(self.ptr, val)
    }
    fn tick_cycle(self, cpu: &mut CPU) {
        cpu.inc_page_cycle(self.ptr_base, self.ptr)
    }
    fn untick_cycle(self, cpu: &mut CPU) {
        cpu.dec_page_cycle(self.ptr_base, self.ptr)
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

pub struct Registers {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: Status,
    pub sp: u8,
    pub pc: u16,
}

pub struct CPU {
    pub regs: Registers,
    pub mem: CpuMemory,
    cycle: u64,
    halted: bool,
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
    #[cfg(feature="cputrace")]
    fn trace(&mut self) {
        let opcode = Disassembler::new(self).decode();
        println!(
            "{:04X} {:9} {}{:30}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:3} SL:{}",
            self.regs.pc,
            opcode.bytes.iter()
                .map(|byte| format!("{:02X}", byte))
                .fold("".to_string(), |left, right| left + " " + &right ),
            if opcode.unofficial { "*" } else { " " },
            opcode.str,
            self.regs.a,
            self.regs.x,
            self.regs.y,
            self.regs.p.bits(),
            self.regs.sp,
            self.mem.ppu.cycle(),
            self.mem.ppu.scanline(),
        );
    }

    #[cfg(not(feature="cputrace"))]
    fn trace(&self) {}

    #[cfg(feature="stacktrace")]
    fn stack_dump(&mut self) {
        println!{
            "Stack: {:>60}",
            (self.regs.sp..0xFF)
                .map(|idx| self.mem.read(0x0100 + idx as u16))
                .map(|byte| format!("{:02X}", byte))
                .fold("".to_string(), |left, right| left + " " + &right )
        }
    }

    #[cfg(not(feature="stacktrace"))]
    fn stack_dump(&self) {}

    // Addressing modes
    fn immediate(&mut self) -> ImmediateAddressingMode {
        ImmediateAddressingMode
    }
    fn absolute(&mut self) -> MemoryAddressingMode {
        let ptr = self.load_w_incr_pc();
        MemoryAddressingMode {
            ptr_base: ptr,
            ptr: ptr,
        }
    }
    fn absolute_x(&mut self) -> MemoryAddressingMode {
        let ptr_base = self.load_w_incr_pc();
        let ptr = ptr_base.wrapping_add(self.regs.x as u16);
        MemoryAddressingMode {
            ptr_base: ptr_base,
            ptr: ptr,
        }
    }
    fn absolute_y(&mut self) -> MemoryAddressingMode {
        let ptr_base = self.load_w_incr_pc();
        let ptr = ptr_base.wrapping_add(self.regs.y as u16);
        MemoryAddressingMode {
            ptr_base: ptr_base,
            ptr: ptr,
        }
    }
    fn zero_page(&mut self) -> MemoryAddressingMode {
        let ptr = self.load_incr_pc() as u16;
        MemoryAddressingMode {
            ptr_base: ptr,
            ptr: ptr,
        }
    }
    fn zero_page_x(&mut self) -> MemoryAddressingMode {
        let ptr = self.load_incr_pc().wrapping_add(self.regs.x) as u16;
        MemoryAddressingMode {
            ptr_base: ptr,
            ptr: ptr,
        }
    }
    fn zero_page_y(&mut self) -> MemoryAddressingMode {
        let ptr = self.load_incr_pc().wrapping_add(self.regs.y) as u16;
        MemoryAddressingMode {
            ptr_base: ptr,
            ptr: ptr,
        }
    }
    fn indirect_x(&mut self) -> MemoryAddressingMode {
        let arg = self.load_incr_pc();
        let zp_idx = arg.wrapping_add(self.regs.x);
        let ptr = self.mem.read_w_zero_page(zp_idx);
        MemoryAddressingMode {
            ptr_base: ptr,
            ptr: ptr,
        }
    }
    fn indirect_y(&mut self) -> MemoryAddressingMode {
        let arg = self.load_incr_pc();
        let ptr_base = self.mem.read_w_zero_page(arg);
        let ptr = ptr_base.wrapping_add(self.regs.y as u16);
        MemoryAddressingMode {
            ptr_base: ptr_base,
            ptr: ptr,
        }
    }
    fn accumulator(&mut self) -> AccumulatorAddressingMode {
        AccumulatorAddressingMode
    }

    // Instructions
    // Stores
    fn stx<M: AddressingMode>(&mut self, mode: M) {
        let val = self.regs.x;
        mode.write(self, val);
    }
    fn sty<M: AddressingMode>(&mut self, mode: M) {
        let val = self.regs.y;
        mode.write(self, val);
    }
    fn sta<M: AddressingMode>(&mut self, mode: M) {
        let val = self.regs.a;
        mode.write(self, val);
    }

    // Loads
    fn ldx<M: AddressingMode>(&mut self, mode: M) {
        mode.tick_cycle(self);
        let arg = mode.read(self);
        self.regs.x = self.set_sign_zero(arg);
    }
    fn lda<M: AddressingMode>(&mut self, mode: M) {
        mode.tick_cycle(self);
        let arg = mode.read(self);
        self.regs.a = self.set_sign_zero(arg);
    }
    fn ldy<M: AddressingMode>(&mut self, mode: M) {
        mode.tick_cycle(self);
        let arg = mode.read(self);
        self.regs.y = self.set_sign_zero(arg);
    }

    // Logic/Math operations
    fn bit<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        self.set_sign(arg);
        let ac = self.regs.a;
        self.set_zero(arg & ac);
        self.set_overflow((arg & 0x40) != 0);
    }
    fn and<M: AddressingMode>(&mut self, mode: M) {
        mode.tick_cycle(self);
        let ac = self.regs.a & mode.read(self);
        self.regs.a = self.set_sign_zero(ac);
    }
    fn ora<M: AddressingMode>(&mut self, mode: M) {
        mode.tick_cycle(self);
        let ac = self.regs.a | mode.read(self);
        self.regs.a = self.set_sign_zero(ac);
    }
    fn eor<M: AddressingMode>(&mut self, mode: M) {
        mode.tick_cycle(self);
        let ac = self.regs.a ^ mode.read(self);
        self.regs.a = self.set_sign_zero(ac);
    }
    fn adc<M: AddressingMode>(&mut self, mode: M) {
        mode.tick_cycle(self);
        let arg = mode.read(self);
        self.do_adc(arg);
    }
    fn sbc<M: AddressingMode>(&mut self, mode: M) {
        mode.tick_cycle(self);
        let arg = mode.read(self);
        self.do_adc(!arg);
    }
    fn cmp<M: AddressingMode>(&mut self, mode: M) {
        mode.tick_cycle(self);
        let arg = mode.read(self);
        let ac = self.regs.a;
        self.set_carry(!(ac < arg));
        let res = ac.wrapping_sub(arg);
        self.set_sign_zero(res);
    }
    fn cpx<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        let x = self.regs.x;
        self.set_carry(!(x < arg));
        let res = x.wrapping_sub(arg);
        self.set_sign_zero(res);
    }
    fn cpy<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        let y = self.regs.y;
        self.set_carry(!(y < arg));
        let res = y.wrapping_sub(arg);
        self.set_sign_zero(res);
    }
    fn inc<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        let res = self.set_sign_zero(arg.wrapping_add(1));
        mode.write(self, res);
    }
    fn inx(&mut self) {
        let res = self.regs.x.wrapping_add(1);
        self.regs.x = self.set_sign_zero(res);
    }
    fn iny(&mut self) {
        let res = self.regs.y.wrapping_add(1);
        self.regs.y = self.set_sign_zero(res);
    }
    fn dec<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        let res = self.set_sign_zero(arg.wrapping_sub(1));
        mode.write(self, res);
    }
    fn dex(&mut self) {
        let res = self.regs.x.wrapping_sub(1);
        self.regs.x = self.set_sign_zero(res);
    }
    fn dey(&mut self) {
        let res = self.regs.y.wrapping_sub(1);
        self.regs.y = self.set_sign_zero(res);
    }
    fn lsr<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        self.set_carry(arg & 0x01 != 0);
        let res = self.set_sign_zero(arg >> 1);
        mode.write(self, res);
    }
    fn asl<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        self.set_carry(arg & 0x80 != 0);
        let res = self.set_sign_zero(arg << 1);
        mode.write(self, res);
    }
    fn ror<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        let new_carry = arg & 0x01 != 0;
        let mut res = arg >> 1;
        if self.regs.p.contains(C) {
            res |= 0x80;
        }
        let res = self.set_sign_zero(res);
        self.set_carry(new_carry);
        mode.write(self, res);
    }
    fn rol<M: AddressingMode>(&mut self, mode: M) {
        let arg = mode.read(self);
        let new_carry = arg & 0x80 != 0;
        let mut res = arg << 1;
        if self.regs.p.contains(C) {
            res |= 0x01;
        }
        let res = self.set_sign_zero(res);
        self.set_carry(new_carry);
        mode.write(self, res);
    }

    // Jumps
    fn jmp(&mut self) {
        self.regs.pc = self.load_w_incr_pc();
    }
    fn jmpi(&mut self) {
        let arg = self.load_w_incr_pc();
        self.regs.pc = self.mem.read_w_same_page(arg);
    }
    fn jsr(&mut self) {
        let target = self.load_w_incr_pc();
        let return_addr = self.regs.pc - 1;
        self.regs.pc = target;
        self.stack_push_w(return_addr);
    }
    fn rts(&mut self) {
        self.regs.pc = self.stack_pop_w().wrapping_add(1);
    }
    fn rti(&mut self) {
        let status = self.stack_pop();
        self.regs.p = Status::from_bits_truncate(status);
        self.regs.p.insert(U);
        self.regs.pc = self.stack_pop_w();
    }
    fn brk(&mut self) {
        let target = self.mem.read_w(0xFFFE);
        let return_addr = self.regs.pc - 1;
        self.regs.pc = target;
        self.stack_push_w(return_addr);
    }

    // Branches
    fn bcs(&mut self) {
        let cond = self.regs.p.contains(C);
        self.branch(cond);
    }
    fn bcc(&mut self) {
        let cond = !self.regs.p.contains(C);
        self.branch(cond);
    }
    fn beq(&mut self) {
        let cond = self.regs.p.contains(Z);
        self.branch(cond);
    }
    fn bne(&mut self) {
        let cond = !self.regs.p.contains(Z);
        self.branch(cond);
    }
    fn bvs(&mut self) {
        let cond = self.regs.p.contains(V);
        self.branch(cond);
    }
    fn bvc(&mut self) {
        let cond = !self.regs.p.contains(V);
        self.branch(cond);
    }
    fn bmi(&mut self) {
        let cond = self.regs.p.contains(S);
        self.branch(cond);
    }
    fn bpl(&mut self) {
        let cond = !self.regs.p.contains(S);
        self.branch(cond);
    }

    // Stack
    fn plp(&mut self) {
        let p = self.stack_pop();
        self.regs.p = Status::from_bits_truncate(p);
        self.regs.p.remove(B);
        self.regs.p.insert(U);
    }
    fn php(&mut self) {
        let p = self.regs.p;
        self.stack_push(p.bits() | 0b0011_0000);
    }
    fn pla(&mut self) {
        let val = self.stack_pop();
        self.regs.a = self.set_sign_zero(val);
    }
    fn pha(&mut self) {
        let a = self.regs.a;
        self.stack_push(a);
    }

    // Misc
    fn nop(&mut self) {}
    fn sec(&mut self) {
        self.regs.p.insert(C);
    }
    fn clc(&mut self) {
        self.regs.p.remove(C);
    }
    fn sei(&mut self) {
        self.regs.p.insert(I);
    }
    fn sed(&mut self) {
        self.regs.p.insert(D);
    }
    fn cld(&mut self) {
        self.regs.p.remove(D);
    }
    fn clv(&mut self) {
        self.regs.p.remove(V);
    }
    fn tax(&mut self) {
        let res = self.regs.a;
        self.regs.x = self.set_sign_zero(res);
    }
    fn tay(&mut self) {
        let res = self.regs.a;
        self.regs.y = self.set_sign_zero(res);
    }
    fn tsx(&mut self) {
        let res = self.regs.sp;
        self.regs.x = self.set_sign_zero(res);
    }
    fn txa(&mut self) {
        let res = self.regs.x;
        self.regs.a = self.set_sign_zero(res);
    }
    fn txs(&mut self) {
        self.regs.sp = self.regs.x;
    }
    fn tya(&mut self) {
        let res = self.regs.y;
        self.regs.a = self.set_sign_zero(res);
    }

    // Unofficial opcodes
    fn u_nop<M: AddressingMode>(&mut self, mode: M) {
        mode.tick_cycle(self);
        mode.read(self);
    }
    fn lax<M: AddressingMode>(&mut self, mode: M) {
        mode.tick_cycle(self);
        let arg = mode.read(self);
        self.regs.a = self.set_sign_zero(arg);
        self.regs.x = self.regs.a;
    }
    fn sax<M: AddressingMode>(&mut self, mode: M) {
        let res = self.regs.a & self.regs.x;
        mode.write(self, res);
    }
    fn dcp<M: AddressingMode>(&mut self, mode: M) {
        self.dec(mode);
        self.cmp(mode);
        mode.untick_cycle(self);
    }
    fn isc<M: AddressingMode>(&mut self, mode: M) {
        self.inc(mode);
        self.sbc(mode);
        mode.untick_cycle(self);
    }
    fn slo<M: AddressingMode>(&mut self, mode: M) {
        self.asl(mode);
        self.ora(mode);
        mode.untick_cycle(self);
    }
    fn rla<M: AddressingMode>(&mut self, mode: M) {
        self.rol(mode);
        self.and(mode);
        mode.untick_cycle(self);
    }
    fn sre<M: AddressingMode>(&mut self, mode: M) {
        self.lsr(mode);
        self.eor(mode);
        mode.untick_cycle(self);
    }
    fn rra<M: AddressingMode>(&mut self, mode: M) {
        self.ror(mode);
        self.adc(mode);
        mode.untick_cycle(self);
    }
    fn kil(&mut self) {
        self.halted = true;
    }

    pub fn new(mem: CpuMemory) -> CPU {
        CPU {
            regs: Registers {
                a: 0,
                x: 0,
                y: 0,
                p: Status::init(),
                sp: 0xFD,
                pc: 0,
            },
            cycle: 0,
            mem: mem,
            halted: false,
        }
    }

    pub fn init(&mut self) {
        self.regs.pc = 0xC000;
        //self.regs.pc = self.mem.read_w(0xFFFC);
    }

    fn load_incr_pc(&mut self) -> u8 {
        let res = self.mem.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        res
    }

    fn load_w_incr_pc(&mut self) -> u16 {
        let res = self.mem.read_w(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(2);
        res
    }

    fn set_sign(&mut self, arg: u8) {
        if arg & 0b1000_0000 == 0 {
            self.regs.p.remove(S);
        } else {
            self.regs.p.insert(S);
        }
    }

    fn set_zero(&mut self, arg: u8) {
        if arg == 0 {
            self.regs.p.insert(Z);
        } else {
            self.regs.p.remove(Z);
        }
    }

    fn set_sign_zero(&mut self, arg: u8) -> u8 {
        self.set_sign(arg);
        self.set_zero(arg);
        arg
    }

    fn set_overflow(&mut self, arg: bool) {
        if arg {
            self.regs.p.insert(V);
        } else {
            self.regs.p.remove(V);
        }
    }

    fn set_carry(&mut self, arg: bool) {
        if arg {
            self.regs.p.insert(C);
        } else {
            self.regs.p.remove(C);
        }
    }

    fn relative_addr(&self, disp: u8) -> u16 {
        // Double-cast to force sign-extension
        let disp = (disp as i8) as i16;
        let pc = self.regs.pc as i16;
        pc.wrapping_add(disp) as u16
    }

    fn do_adc(&mut self, arg: u8) {
        let mut result = self.regs.a as u16 + arg as u16;
        if self.regs.p.contains(C) {
            result += 1;
        }

        self.set_carry(result > 0xFF);

        let result = result as u8;
        let a = self.regs.a;
        self.set_overflow((a ^ arg) & 0x80 == 0 && (a ^ result) & 0x80 == 0x80);
        self.regs.a = self.set_sign_zero(result);
    }

    fn branch(&mut self, cond: bool) {
        let arg = self.load_incr_pc();
        if cond {
            let target = self.relative_addr(arg);
            let pc = self.regs.pc;
            self.incr_cycle(1);
            self.inc_page_cycle(pc, target);
            self.regs.pc = self.relative_addr(arg);
        }
    }

    fn stack_push(&mut self, val: u8) {
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        self.mem.write(self.regs.sp as u16 + 0x0101, val);
    }
    fn stack_push_w(&mut self, val: u16) {
        self.regs.sp = self.regs.sp.wrapping_sub(2);
        self.mem.write_w(self.regs.sp as u16 + 0x0101, val);
    }
    fn stack_pop(&mut self) -> u8 {
        self.regs.sp = self.regs.sp.wrapping_add(1);
        self.mem.read(self.regs.sp as u16 + 0x0100)
    }
    fn stack_pop_w(&mut self) -> u16 {
        self.regs.sp = self.regs.sp.wrapping_add(2);
        self.mem.read_w(self.regs.sp as u16 + 0x00FF)
    }

    fn incr_cycle(&mut self, cycles: u64) {
        self.cycle = self.cycle.wrapping_add(cycles);
    }
    fn decr_cycle(&mut self, cycles: u64) {
        self.cycle = self.cycle.wrapping_sub(cycles);
    }
    fn inc_page_cycle(&mut self, addr1: u16, addr2: u16) {
        if addr1 & 0xFF00 != addr2 & 0xFF00 {
            self.incr_cycle(1);
        }
    }
    fn dec_page_cycle(&mut self, addr1: u16, addr2: u16) {
        if addr1 & 0xFF00 != addr2 & 0xFF00 {
            self.decr_cycle(1);
        }
    }

    fn unofficial(&self) {}

    pub fn step(&mut self) -> u64 {
        if self.halted {
            return 0;
        }
        let old_cyc = self.cycle;
        self.trace();
        self.stack_dump();
        let opcode: u8 = self.load_incr_pc();
        decode_opcode!(opcode, self);
        self.incr_cycle(CYCLE_TABLE[opcode as usize] as u64);
        self.cycle - old_cyc
    }
    
    pub fn halted(&self) -> bool {
        self.halted
    }
}
