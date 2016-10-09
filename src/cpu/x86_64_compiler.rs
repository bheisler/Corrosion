#![allow(dead_code)]

use cpu::CPU;
use cpu::Registers;
use cpu::nes_analyst::Analyst;
use std::mem;
use memory::MemSegment;

use dynasmrt::{AssemblyOffset, DynasmApi, DynasmLabelApi, ExecutableBuffer};

pub struct ExecutableBlock {
    offset: AssemblyOffset,
    buffer: ExecutableBuffer,
}

impl ExecutableBlock {
    pub fn call(&self, cpu: *mut CPU) {
        let offset = self.offset;
        let f: extern "win64" fn(*mut CPU, *mut Registers) -> () =
            unsafe { mem::transmute(self.buffer.ptr(offset)) };
        let regs = unsafe { &mut (*cpu).regs };
        f(cpu, regs as _);
    }
}

pub fn compile(addr: u16, cpu: &mut CPU) -> ExecutableBlock {
    Compiler::new(cpu).compile_block(addr)
}

macro_rules! unimplemented {
    ($opcode:ident) => {
        panic!(stringify!(Unknown or unimplemented operation $opcode));
    };
}

dynasm!(ops
    ; .alias cpu, rcx
    ; .alias regs, rdx
    ; .alias temp, r8b
    ; .alias n_a, r9b
    ; .alias n_x, r10b
    ; .alias n_y, r11b
    ; .alias n_p, r12b
    ; .alias n_sp, r13b
    ; .alias n_pc, r14w
    ; .alias cyc, r15
);

struct Compiler<'a> {
    asm: ::dynasmrt::Assembler,
    cpu: &'a mut CPU,

    pc: u16,
}

macro_rules! load_registers {
    ($ops:ident) => {{
        dynasm!($ops
            ; xor r8, r8
            ; xor r9, r9
            ; mov n_a, BYTE regs => Registers.a
            ; xor r10, r10
            ; mov n_x, BYTE regs => Registers.x
            ; xor r11, r11
            ; mov n_y, BYTE regs => Registers.y
            ; xor r12, r12
            ; mov n_p, BYTE regs => Registers.p
            ; xor r13, r13
            ; mov n_sp, BYTE regs => Registers.sp
            ; xor r14, r14
            ; mov n_pc, WORD regs => Registers.pc
            ; mov cyc, QWORD cpu => CPU.cycle
        );
    }};
}

macro_rules! store_registers {
    ($ops:ident) => {{
        dynasm!($ops
            ; mov BYTE regs => Registers.a, n_a
            ; mov BYTE regs => Registers.x, n_x
            ; mov BYTE regs => Registers.y, n_y
            ; mov BYTE regs => Registers.p, n_p
            ; mov BYTE regs => Registers.sp, n_sp
            ; mov WORD regs => Registers.pc, n_pc
            ; mov QWORD cpu => CPU.cycle, cyc
        );
    }};
}

macro_rules! prologue {
    ($ops:ident) => {{
        dynasm!{$ops
            ; push r12
            ; push r13
            ; push r14
            ; push r15
        }
        load_registers!($ops);
    }};
}

macro_rules! epilogue {
    ($ops:ident) => {{
        store_registers!($ops);
        dynasm!{$ops
            ; pop r15
            ; pop r14
            ; pop r13
            ; pop r12
            ; ret
        }
    }};
}

macro_rules! call_extern {
    ($ops:ident, $addr:expr) => {dynasm!($ops
        ; push rax
        ; push rcx
        ; push rdx
        ; push r8
        ; push r9
        ; push r10
        ; push r11
        ; sub rsp, 0x20
        ; mov rax, QWORD $addr as _
        ; call rax
        ; add rsp, 0x20
        ; pop r11
        ; pop r10
        ; pop r9
        ; pop r8
        ; pop rdx
        ; pop rcx
        ; pop rax
    );};
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
        self.pc = addr;
        let analysis = Analyst::new(self.cpu).analyze(addr);

        let start = self.asm.offset();
        let mut asm = self.asm;

        // TODO: Add way to call back to rust code
        // TODO: Implement the rest of the operations

        // TODO: Handle flags
        // TODO: Count CPU cycles
        // TODO: Implement interrupts

        prologue!(asm);

        // while self.pc < analysis.exit_point {
        // let opcode = self.read_incr_pc();
        // decode_opcode!(opcode, self);
        // }

        epilogue!(asm);

        ExecutableBlock {
            offset: start,
            buffer: asm.finalize().unwrap(),
        }
    }

    // Addressing modes
    fn immediate(&mut self) -> u8 {
        self.read_incr_pc();
        unimplemented!(immediate);
    }
    fn absolute(&mut self) -> u8 {
        self.read_w_incr_pc();
        unimplemented!(absolute);
    }
    fn absolute_x(&mut self) -> u8 {
        self.read_w_incr_pc();
        unimplemented!(absolute_x);
    }
    fn absolute_y(&mut self) -> u8 {
        self.read_w_incr_pc();
        unimplemented!(absolute_y);
    }
    fn zero_page(&mut self) -> u8 {
        self.read_incr_pc();
        unimplemented!(zero_page);
    }
    fn zero_page_x(&mut self) -> u8 {
        self.read_incr_pc();
        unimplemented!(zero_page_x);
    }
    fn zero_page_y(&mut self) -> u8 {
        self.read_incr_pc();
        unimplemented!(zero_page_y);
    }
    fn indirect_x(&mut self) -> u8 {
        self.read_incr_pc();
        unimplemented!(indirect_x);
    }
    fn indirect_y(&mut self) -> u8 {
        self.read_incr_pc();
        unimplemented!(indirect_y);
    }
    fn accumulator(&mut self) -> u8 {
        unimplemented!(accumulator);
    }

    // Instructions
    // Stores
    fn stx(&mut self, _: u8) {
        unimplemented!(stx);
    }
    fn sty(&mut self, _: u8) {
        unimplemented!(sty);
    }
    fn sta(&mut self, _: u8) {
        unimplemented!(sta);
    }

    // Loads
    fn ldx(&mut self, _: u8) {
        unimplemented!(ldx);
    }
    fn lda(&mut self, _: u8) {
        unimplemented!(lda);
    }
    fn ldy(&mut self, _: u8) {
        unimplemented!(ldy);
    }

    // Logic/Math Ops
    fn bit(&mut self, _: u8) {
        unimplemented!(bit);
    }
    fn and(&mut self, _: u8) {
        unimplemented!(and);
    }
    fn ora(&mut self, _: u8) {
        unimplemented!(ora);
    }
    fn eor(&mut self, _: u8) {
        unimplemented!(eor);
    }
    fn adc(&mut self, _: u8) {
        unimplemented!(adc);
    }
    fn sbc(&mut self, _: u8) {
        unimplemented!(sbc);
    }
    fn cmp(&mut self, _: u8) {
        unimplemented!(cmp);
    }
    fn cpx(&mut self, _: u8) {
        unimplemented!(cpx);
    }
    fn cpy(&mut self, _: u8) {
        unimplemented!(cpy);
    }
    fn inc(&mut self, _: u8) {
        unimplemented!(inc);
    }
    fn iny(&mut self) {
        unimplemented!(iny);
    }
    fn inx(&mut self) {
        unimplemented!(inx);
    }
    fn dec(&mut self, _: u8) {
        unimplemented!(dec);
    }
    fn dey(&mut self) {
        unimplemented!(dey);
    }
    fn dex(&mut self) {
        unimplemented!(dex);
    }
    fn lsr(&mut self, _: u8) {
        unimplemented!(lsr);
    }
    fn asl(&mut self, _: u8) {
        unimplemented!(asl);
    }
    fn ror(&mut self, _: u8) {
        unimplemented!(ror);
    }
    fn rol(&mut self, _: u8) {
        unimplemented!(rol);
    }

    // Jumps
    fn jmp(&mut self) {
        self.read_w_incr_pc();
        unimplemented!(jmp);
    }
    fn jmpi(&mut self) {
        self.read_w_incr_pc();
        unimplemented!(jmpi);
    }
    fn jsr(&mut self) {
        self.read_w_incr_pc();
        unimplemented!(jsr);
    }
    fn rts(&mut self) {
        unimplemented!(rts);
    }
    fn rti(&mut self) {
        unimplemented!(rti);
    }
    fn brk(&mut self) {
        unimplemented!(brk);
    }

    fn unofficial(&self) {}

    // Branches
    fn bcs(&mut self) {
        self.branch();
        unimplemented!(bcs);
    }
    fn bcc(&mut self) {
        self.branch();
        unimplemented!(bcc);
    }
    fn beq(&mut self) {
        self.branch();
        unimplemented!(beq);
    }
    fn bne(&mut self) {
        self.branch();
        unimplemented!(bne);
    }
    fn bvs(&mut self) {
        self.branch();
        unimplemented!(bvs);
    }
    fn bvc(&mut self) {
        self.branch();
        unimplemented!(bvc);
    }
    fn bmi(&mut self) {
        self.branch();
        unimplemented!(bmi);
    }
    fn bpl(&mut self) {
        self.branch();
        unimplemented!(bpl);
    }

    fn branch(&mut self) {
        let arg = self.read_incr_pc();
        let target = self.relative_addr(arg);
    }

    // Stack
    fn plp(&mut self) {
        unimplemented!(plp);
    }
    fn php(&mut self) {
        unimplemented!(php);
    }
    fn pla(&mut self) {
        unimplemented!(pla);
    }
    fn pha(&mut self) {
        unimplemented!(pha);
    }

    // Misc
    fn nop(&mut self) {
        unimplemented!(nop);
    }
    fn sec(&mut self) {
        unimplemented!(sec);
    }
    fn clc(&mut self) {
        unimplemented!(clc);
    }
    fn sei(&mut self) {
        unimplemented!(sei);
    }
    fn sed(&mut self) {
        unimplemented!(sed);
    }
    fn cld(&mut self) {
        unimplemented!(cld);
    }
    fn clv(&mut self) {
        unimplemented!(clv);
    }
    fn tax(&mut self) {
        unimplemented!(tax);
    }
    fn tay(&mut self) {
        unimplemented!(tay);
    }
    fn tsx(&mut self) {
        unimplemented!(tsx);
    }
    fn txa(&mut self) {
        unimplemented!(txa);
    }
    fn txs(&mut self) {
        unimplemented!(txs);
    }
    fn tya(&mut self) {
        unimplemented!(tya);
    }
    fn cli(&mut self) {
        unimplemented!(cli);
    }

    // Unofficial instructions
    fn u_nop(&mut self, _: u8) {
        unimplemented!(u_nop);
    }
    fn lax(&mut self, _: u8) {
        unimplemented!(lax);
    }
    fn sax(&mut self, _: u8) {
        unimplemented!(sax);
    }
    fn dcp(&mut self, _: u8) {
        unimplemented!(dcp);
    }
    fn isc(&mut self, _: u8) {
        unimplemented!(isc);
    }
    fn slo(&mut self, _: u8) {
        unimplemented!(slo);
    }
    fn rla(&mut self, _: u8) {
        unimplemented!(rla);
    }
    fn sre(&mut self, _: u8) {
        unimplemented!(sre);
    }
    fn rra(&mut self, _: u8) {
        unimplemented!(rra);
    }
    fn kil(&mut self) {
        unimplemented!(kil);
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
