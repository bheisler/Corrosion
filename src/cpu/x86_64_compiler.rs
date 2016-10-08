#![allow(dead_code)]

use cpu::CPU;
use cpu::nes_analyst::Analyst;
use std::mem;
use memory::MemSegment;

use dynasmrt::{DynasmApi, DynasmLabelApi};

pub type ExecutableBlock = extern "win64" fn(*mut CPU) -> ();

pub fn compile(addr: u16, cpu: &mut CPU) -> ExecutableBlock {
    Compiler::new(cpu).compile_block(addr)
}

extern "win64" fn do_nothing(_: *mut CPU) -> () {
    // Do Nothing
}

struct Compiler<'a> {
    asm: ::dynasmrt::Assembler,
    cpu: &'a mut CPU,

    pc: u16,
}

impl<'a> Compiler<'a> {
    fn new(cpu: &'a mut CPU) -> Compiler<'a> {
        Compiler {
            asm: ::dynasmrt::Assembler::new(),
            cpu: cpu,

            pc: 0,
        }
    }

    fn compile_block(mut self, addr: u16) -> ExecutableBlock {
        let analysis = Analyst::new(self.cpu).analyze(addr);

        let mut test: u8 = 12;

        let start = self.asm.offset();
        dynasm!{self.asm
            ; mov [rcx], BYTE 0
            ; ret
        }

        let buf = self.asm.finalize().unwrap();
        let test_fn: extern "win64" fn(*mut u8) -> () = unsafe { mem::transmute(buf.ptr(start)) };

        test_fn(&mut test as _);
        assert_eq!(0, test);
        do_nothing
    }

    // Addressing modes
    fn immediate(&mut self) -> u8 {
        self.read_incr_pc();
        unimplemented!();
    }
    fn absolute(&mut self) -> u8 {
        self.read_w_incr_pc();
        unimplemented!();
    }
    fn absolute_x(&mut self) -> u8 {
        self.read_w_incr_pc();
        unimplemented!();
    }
    fn absolute_y(&mut self) -> u8 {
        self.read_w_incr_pc();
        unimplemented!();
    }
    fn zero_page(&mut self) -> u8 {
        self.read_incr_pc();
        unimplemented!();
    }
    fn zero_page_x(&mut self) -> u8 {
        self.read_incr_pc();
        unimplemented!();
    }
    fn zero_page_y(&mut self) -> u8 {
        self.read_incr_pc();
        unimplemented!();
    }
    fn indirect_x(&mut self) -> u8 {
        self.read_incr_pc();
        unimplemented!();
    }
    fn indirect_y(&mut self) -> u8 {
        self.read_incr_pc();
        unimplemented!();
    }
    fn accumulator(&mut self) -> u8 {
        unimplemented!();
    }

    // Instructions
    // Stores
    fn stx(&mut self, _: u8) {
        unimplemented!();
    }
    fn sty(&mut self, _: u8) {
        unimplemented!();
    }
    fn sta(&mut self, _: u8) {
        unimplemented!();
    }

    // Loads
    fn ldx(&mut self, _: u8) {
        unimplemented!();
    }
    fn lda(&mut self, _: u8) {
        unimplemented!();
    }
    fn ldy(&mut self, _: u8) {
        unimplemented!();
    }

    // Logic/Math Ops
    fn bit(&mut self, _: u8) {
        unimplemented!();
    }
    fn and(&mut self, _: u8) {
        unimplemented!();
    }
    fn ora(&mut self, _: u8) {
        unimplemented!();
    }
    fn eor(&mut self, _: u8) {
        unimplemented!();
    }
    fn adc(&mut self, _: u8) {
        unimplemented!();
    }
    fn sbc(&mut self, _: u8) {
        unimplemented!();
    }
    fn cmp(&mut self, _: u8) {
        unimplemented!();
    }
    fn cpx(&mut self, _: u8) {
        unimplemented!();
    }
    fn cpy(&mut self, _: u8) {
        unimplemented!();
    }
    fn inc(&mut self, _: u8) {
        unimplemented!();
    }
    fn iny(&mut self) {
        unimplemented!();
    }
    fn inx(&mut self) {
        unimplemented!();
    }
    fn dec(&mut self, _: u8) {
        unimplemented!();
    }
    fn dey(&mut self) {
        unimplemented!();
    }
    fn dex(&mut self) {
        unimplemented!();
    }
    fn lsr(&mut self, _: u8) {
        unimplemented!();
    }
    fn asl(&mut self, _: u8) {
        unimplemented!();
    }
    fn ror(&mut self, _: u8) {
        unimplemented!();
    }
    fn rol(&mut self, _: u8) {
        unimplemented!();
    }

    // Jumps
    fn jmp(&mut self) {
        self.read_w_incr_pc();
        unimplemented!();
    }
    fn jmpi(&mut self) {
        self.read_w_incr_pc();
        unimplemented!();
    }
    fn jsr(&mut self) {
        self.read_w_incr_pc();
        unimplemented!();
    }
    fn rts(&mut self) {
        unimplemented!();
    }
    fn rti(&mut self) {
        unimplemented!();
    }
    fn brk(&mut self) {
        unimplemented!();
    }

    fn unofficial(&self) {}

    // Branches
    fn bcs(&mut self) {
        self.branch()
    }
    fn bcc(&mut self) {
        self.branch()
    }
    fn beq(&mut self) {
        self.branch()
    }
    fn bne(&mut self) {
        self.branch()
    }
    fn bvs(&mut self) {
        self.branch()
    }
    fn bvc(&mut self) {
        self.branch()
    }
    fn bmi(&mut self) {
        self.branch()
    }
    fn bpl(&mut self) {
        self.branch()
    }

    fn branch(&mut self) {
        let arg = self.read_incr_pc();
        let target = self.relative_addr(arg);
        unimplemented!();
    }

    // Stack
    fn plp(&mut self) {
        unimplemented!();
    }
    fn php(&mut self) {
        unimplemented!();
    }
    fn pla(&mut self) {
        unimplemented!();
    }
    fn pha(&mut self) {
        unimplemented!();
    }

    // Misc
    fn nop(&mut self) {
        unimplemented!();
    }
    fn sec(&mut self) {
        unimplemented!();
    }
    fn clc(&mut self) {
        unimplemented!();
    }
    fn sei(&mut self) {
        unimplemented!();
    }
    fn sed(&mut self) {
        unimplemented!();
    }
    fn cld(&mut self) {
        unimplemented!();
    }
    fn clv(&mut self) {
        unimplemented!();
    }
    fn tax(&mut self) {
        unimplemented!();
    }
    fn tay(&mut self) {
        unimplemented!();
    }
    fn tsx(&mut self) {
        unimplemented!();
    }
    fn txa(&mut self) {
        unimplemented!();
    }
    fn txs(&mut self) {
        unimplemented!();
    }
    fn tya(&mut self) {
        unimplemented!();
    }
    fn cli(&mut self) {
        unimplemented!();
    }

    // Unofficial instructions
    fn u_nop(&mut self, _: u8) {
        unimplemented!();
    }
    fn lax(&mut self, _: u8) {
        unimplemented!();
    }
    fn sax(&mut self, _: u8) {
        unimplemented!();
    }
    fn dcp(&mut self, _: u8) {
        unimplemented!();
    }
    fn isc(&mut self, _: u8) {
        unimplemented!();
    }
    fn slo(&mut self, _: u8) {
        unimplemented!();
    }
    fn rla(&mut self, _: u8) {
        unimplemented!();
    }
    fn sre(&mut self, _: u8) {
        unimplemented!();
    }
    fn rra(&mut self, _: u8) {
        unimplemented!();
    }
    fn kil(&mut self) {
        unimplemented!();
    }

    fn relative_addr(&self, disp: u8) -> u16 {
        let disp = (disp as i8) as i16; //We want to sign-extend here.
        let pc = self.pc as i16;
        pc.wrapping_add(disp) as u16
    }

    fn read_incr_pc(&mut self) -> u8 {
        let pc = self.pc;
        let val: u8 = self.read_safe(pc);
        self.pc = self.pc.wrapping_add(1);
        val
    }

    fn read_w_incr_pc(&mut self) -> u16 {
        self.read_incr_pc() as u16 | ((self.read_incr_pc() as u16) << 8)
    }

    fn read_safe(&mut self, idx: u16) -> u8 {
        match idx {
            0x2000...0x3FFF => 0xFF,
            0x4000...0x401F => 0xFF,
            _ => self.cpu.read(idx),
        }
    }
}
