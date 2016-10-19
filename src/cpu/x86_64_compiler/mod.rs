#![allow(unneeded_field_pattern)]

use cpu::CPU;
use cpu::Registers;
use cpu::nes_analyst::Analyst;
use std::mem;
use memory::MemSegment;
use std::collections::HashMap;

use dynasmrt::{AssemblyOffset, DynasmApi, DynasmLabelApi, ExecutableBuffer, DynamicLabel};

const CARRY: u8 = 0b0000_0001;
const ZERO: u8 = 0b0000_0010;
const SUPPRESS_IRQ: u8 = 0b0000_0100;
const DECIMAL: u8 = 0b0000_1000;
const BREAK : u8 = 0b0001_0000;
const OVERFLOW: u8 = 0b0100_0000;
const SIGN: u8 = 0b1000_0000;

pub struct ExecutableBlock {
    offset: AssemblyOffset,
    buffer: ExecutableBuffer,
}

impl ExecutableBlock {
    pub fn call(&self, cpu: *mut CPU) {
        let offset = self.offset;
        let f: extern "win64" fn(*mut CPU, *mut Registers, *mut [u8; 0x800]) -> () =
            unsafe { mem::transmute(self.buffer.ptr(offset)) };
        let regs = unsafe { &mut (*cpu).regs };
        let ram = unsafe { &mut (*cpu).ram };
        f(cpu, regs as _, ram as _);
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

dynasm!(this
    ; .alias cpu, rbx
    ; .alias regs, rcx
    ; .alias ram, rdx
    ; .alias arg, r8b
    ; .alias n_a, r9b
    ; .alias n_x, r10b
    ; .alias n_y, r11b
    ; .alias n_p, r12b
    ; .alias n_sp, r13b
    ; .alias n_pc, r14w
    ; .alias cyc, r15
);

macro_rules! load_registers {
    ($this:ident) => {{
        dynasm!($this.asm
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
    ($this:ident) => {{
        dynasm!($this.asm
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
    ($this:ident) => {{
        dynasm!{$this.asm
            ; push rbx
            ; push r12
            ; push r13
            ; push r14
            ; push r15
            ; mov rbx, rcx //Move the CPU pointer to the CPU pointer register
            ; mov rcx, rdx //Move the registers pointer to the regs pointer register
            ; mov rdx, r8  //Move the RAM pointer to the RAM pointer register
        }
        load_registers!($this);
    }};
}

macro_rules! epilogue {
    ($this:ident) => {{
        store_registers!($this);
        dynasm!{$this.asm
            ; pop r15
            ; pop r14
            ; pop r13
            ; pop r12
            ; pop rbx
            ; ret
        }
    }};
}

macro_rules! call_naked {
    ($this:ident, $addr:expr) => {dynasm!($this.asm
        ; push rax
        ; mov rax, QWORD $addr as _
        ; call rax
        ; pop rax
    );};
}

#[naked]
extern "C" fn set_zero_flag() {
    unsafe {
        asm!("
            cmp r8b, 0
            jz 1f
            and r12b, 0FDH
            jmp 2f
        1:
            or r12b, 2H
        2:
            ret
            "
        :
        :
        : "r12"
        : "intel");
    };
}

#[naked]
extern "C" fn set_sign_flag() {
    unsafe {
        asm!("
            test r8b, 80H
            jz 1f
            or r12b, 80H
            jmp 2f
        1:
            and r12b, 7FH
        2:
            ret
            "
        :
        :
        : "r12"
        : "intel");
    };
}

mod addressing_modes;

use self::addressing_modes::AddressingMode;


struct Compiler<'a> {
    asm: ::dynasmrt::Assembler,
    cpu: &'a mut CPU,

    pc: u16,

    branch_targets: HashMap<u16, DynamicLabel>,
}

impl<'a> Compiler<'a> {
    fn new(cpu: &'a mut CPU) -> Compiler<'a> {
        Compiler {
            asm: ::dynasmrt::Assembler::new(),
            cpu: cpu,

            pc: 0,

            branch_targets: HashMap::new(),
        }
    }

    fn compile_block(mut self, addr: u16) -> ExecutableBlock {
        self.pc = addr;
        let analysis = Analyst::new(self.cpu).analyze(addr);

        let start = self.asm.offset();

        // TODO: Implement the rest of the operations

        // TODO: Count CPU cycles
        // TODO: Implement interrupts

        // TODO: Centralize the flag operations

        prologue!(self);

        while self.pc <= analysis.exit_point {
            let temp_pc = self.pc;
            if analysis.instructions.get(&temp_pc).unwrap().is_branch_target {
                let target_label = self.get_dynamic_label(temp_pc);
                dynasm!{self.asm
                    ; => target_label
                }
            }

            let opcode = self.read_incr_pc();
            decode_opcode!(opcode, self);
        }

        ExecutableBlock {
            offset: start,
            buffer: self.asm.finalize().unwrap(),
        }
    }

    // Stores
    fn stx<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ; mov arg, n_x
            ;; mode.write_from_arg(self)
        }
    }
    fn sty<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ; mov arg, n_y
            ;; mode.write_from_arg(self)
        }
    }
    fn sta<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ; mov arg, n_a
            ;; mode.write_from_arg(self)
        }
    }

    // Loads
    fn ldx<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self)
            ; mov n_x, arg
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn lda<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self)
            ; mov n_a, arg
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn ldy<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self)
            ; mov n_y, arg
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }

    // Logic/Math Ops
    fn bit<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self)

            //Set the sign flag
            ;; call_naked!(self, set_sign_flag)

            //Set the overflow flag
            ; test arg, BYTE 0b0100_0000
            ; jz >clear
            ; or n_p, BYTE OVERFLOW as _
            ; jmp >next
            ; clear:
            ; and n_p, BYTE (!OVERFLOW) as _
            ; next:

            //Set the zero flag
            ; and arg, n_a
            ;; call_naked!(self, set_zero_flag)
        }
    }
    fn and<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self)
            ; and arg, n_a
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
            ; mov n_a, arg
        }
    }
    fn ora<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self)
            ; or arg, n_a
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
            ; mov n_a, arg
        }
    }
    fn eor<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self)
            ; xor arg, n_a
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
            ; mov n_a, arg
        }
    }
    fn adc<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self)
            ;; self.do_adc()
        }
    }
    fn sbc<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self)
            ;  not arg
            ;; self.do_adc()
        }
    }
    fn do_adc(&mut self) {
        dynasm!{self.asm
            ; dec rsp
            ; mov [rsp], arg //Save original arg
            ; add r8w, r9w //Add arg + a
            ; test n_p, CARRY as _
            ; jz >next
            ; inc r8w // add the carry flag
            ; next:

            //Set carry based on result
            ; cmp r8w, 0xFF
            ; jl >next
            ; or n_p, CARRY as _
            ; next:

            //Calculate the overflow flag
            ; mov al, n_a
            ; xor al, [rsp]
            ; test al, 0x80
            ; jnz >clear_overflow
            ; mov al, n_a
            ; xor al, arg
            ; test al, 0x80
            ; jz >clear_overflow
            ; or n_p, OVERFLOW as _
            ; jmp >next
            ; clear_overflow:
            ; and n_p, (!OVERFLOW) as _
            ; next:
            ; mov n_a, arg
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
            ; inc rsp
        }
    }

    fn cmp<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self)

            ; cmp arg, n_a
            ; jnc >clear
            ; or n_p, BYTE CARRY as _
            ; jmp >next
            ; clear:
            ; and n_p, BYTE (!CARRY) as _
            ; next:

            ; sub arg, n_a
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn cpx<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self)

            ; cmp arg, n_x
            ; jnc >clear
            ; or n_p, BYTE CARRY as _
            ; jmp >next
            ; clear:
            ; and n_p, BYTE (!CARRY) as _
            ; next:

            ; sub arg, n_x
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn cpy<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self)

            ; cmp arg, n_y
            ; jnc >clear
            ; or n_p, BYTE CARRY as _
            ; jmp >next
            ; clear:
            ; and n_p, BYTE (!CARRY) as _
            ; next:

            ; sub arg, n_y
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn inc<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self)
            ; inc arg
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
            ;; mode.write_from_arg(self)
        }
    }
    fn iny(&mut self) {
        dynasm!{self.asm
            ; inc n_y
            ; mov arg, n_y
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn inx(&mut self) {
        dynasm!{self.asm
            ; inc n_x
            ; mov arg, n_x
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn dec<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self)
            ; dec arg
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
            ;; mode.write_from_arg(self)
        }
    }
    fn dey(&mut self) {
        dynasm!{self.asm
            ; dec n_y
            ; mov arg, n_y
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn dex(&mut self) {
        dynasm!{self.asm
            ; dec n_x
            ; mov arg, n_x
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn lsr<M: AddressingMode>(&mut self, _: M) {
        unimplemented!(lsr);
    }
    fn asl<M: AddressingMode>(&mut self, _: M) {
        unimplemented!(asl);
    }
    fn ror<M: AddressingMode>(&mut self, _: M) {
        unimplemented!(ror);
    }
    fn rol<M: AddressingMode>(&mut self, _: M) {
        unimplemented!(rol);
    }

    // Jumps
    fn jmp(&mut self) {
        let target = self.read_w_incr_pc();
        dynasm!(self.asm
            ; mov n_pc, WORD target as _
            ;; epilogue!(self)
        )
    }
    fn jmpi(&mut self) {
        self.read_w_incr_pc();
        unimplemented!(jmpi);
    }
    fn jsr(&mut self) {
        let target = self.read_w_incr_pc();
        let ret_addr = self.pc - 1;
        dynasm!(self.asm
            ;; self.stack_push_w(ret_addr)
            ; mov n_pc, WORD target as _
            ;; epilogue!(self)
        )
    }
    fn rts(&mut self) {
        dynasm!{self.asm
            ; add n_sp, BYTE 2
            ; mov ax, WORD [ram + r13 + 0xFF]
            ; inc ax
            ; mov n_pc, ax
            ;; epilogue!(self)
        }
    }
    fn rti(&mut self) {
        dynasm!{self.asm
            ; mov n_p, BYTE [ram + r13 + 0x101]
            ; inc n_sp
            ; or n_p, BYTE 0b0010_0000
            ; add n_sp, BYTE 2
            ; mov n_pc, WORD [ram + r13 + 0xFF]
            ;; epilogue!(self)
        }
    }
    fn brk(&mut self) {
        unimplemented!(brk);
    }

    fn unofficial(&self) {}

    // Branches
    fn bcs(&mut self) {
        let target_label = self.get_branch_target_label();
        dynasm!{self.asm
            ; test n_p, CARRY as _
            ; jnz => target_label
        }
    }
    fn bcc(&mut self) {
        let target_label = self.get_branch_target_label();
        dynasm!{self.asm
            ; test n_p, CARRY as _
            ; jz => target_label
        }
    }
    fn beq(&mut self) {
        let target_label = self.get_branch_target_label();
        dynasm!{self.asm
            ; test n_p, ZERO as _
            ; jnz => target_label
        }
    }
    fn bne(&mut self) {
        let target_label = self.get_branch_target_label();
        dynasm!{self.asm
            ; test n_p, ZERO as _
            ; jz => target_label
        }
    }
    fn bvs(&mut self) {
        let target_label = self.get_branch_target_label();
        dynasm!{self.asm
            ; test n_p, OVERFLOW as _
            ; jnz => target_label
        }
    }
    fn bvc(&mut self) {
        let target_label = self.get_branch_target_label();
        dynasm!{self.asm
            ; test n_p, OVERFLOW as _
            ; jz => target_label
        }
    }
    fn bmi(&mut self) {
        let target_label = self.get_branch_target_label();
        dynasm!{self.asm
            ; test n_p, SIGN as _
            ; jnz => target_label
        }
    }
    fn bpl(&mut self) {
        let target_label = self.get_branch_target_label();
        dynasm!{self.asm
            ; test n_p, SIGN as _
            ; jz => target_label
        }
    }

    fn branch(&mut self) {
        let arg = self.read_incr_pc();
        let target = self.relative_addr(arg);
    }

    // Stack
    fn plp(&mut self) {
        dynasm!{self.asm
            ; mov n_p, BYTE [ram + r13 + 0x101]
            ; inc n_sp
            ; or n_p, BYTE 0b0010_0000
            ; and n_p, BYTE (!BREAK) as _
        }
    }
    fn php(&mut self) {
        dynasm!{self.asm
            ; mov arg, n_p
            ; or arg, BYTE 0b0011_0000
            ; dec n_sp
            ; mov BYTE [ram + r13 + 0x101], arg
        }
    }
    fn pla(&mut self) {
        dynasm!{self.asm
            ; mov n_a, BYTE [ram + r13 + 0x101]
            ; inc n_sp
        }
    }
    fn pha(&mut self) {
        dynasm!{self.asm
            ; dec n_sp
            ; mov BYTE [ram + r13 + 0x101], n_a
        }
    }

    // Misc
    fn nop(&mut self) {}
    fn sec(&mut self) {
        dynasm!{self.asm
            ; or n_p, BYTE CARRY as _
        }
    }
    fn clc(&mut self) {
        dynasm!{self.asm
            ; and n_p, BYTE (!CARRY) as _
        }
    }
    fn sei(&mut self) {
        dynasm!{self.asm
            ; or n_p, BYTE SUPPRESS_IRQ as _
        }
    }
    fn cli(&mut self) {
        dynasm!{self.asm
            ; and n_p, BYTE (!SUPPRESS_IRQ) as _
        }
    }
    fn sed(&mut self) {
        dynasm!{self.asm
            ; or n_p, BYTE DECIMAL as _
        }
    }
    fn cld(&mut self) {
        dynasm!{self.asm
            ; and n_p, BYTE (!DECIMAL) as _
        }
    }
    fn clv(&mut self) {
        dynasm!{self.asm
            ; and n_p, BYTE (!OVERFLOW) as _
        }
    }
    fn tax(&mut self) {
        dynasm!{self.asm
            ; mov n_x, n_a
            ; mov arg, n_a
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn tay(&mut self) {
        dynasm!{self.asm
            ; mov n_y, n_a
            ; mov arg, n_a
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn tsx(&mut self) {
        dynasm!{self.asm
            ; mov n_x, n_sp
            ; mov arg, n_sp
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn txa(&mut self) {
        dynasm!{self.asm
            ; mov n_a, n_x
            ; mov arg, n_x
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn txs(&mut self) {
        dynasm!{self.asm
            ; mov n_sp, n_x
            ; mov arg, n_x
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn tya(&mut self) {
        dynasm!{self.asm
            ; mov n_a, n_y
            ; mov arg, n_y
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }

    // Unofficial instructions
    fn u_nop<M: AddressingMode>(&mut self, _: M) {
        unimplemented!(u_nop);
    }
    fn lax<M: AddressingMode>(&mut self, _: M) {
        unimplemented!(lax);
    }
    fn sax<M: AddressingMode>(&mut self, _: M) {
        unimplemented!(sax);
    }
    fn dcp<M: AddressingMode>(&mut self, _: M) {
        unimplemented!(dcp);
    }
    fn isc<M: AddressingMode>(&mut self, _: M) {
        unimplemented!(isc);
    }
    fn slo<M: AddressingMode>(&mut self, _: M) {
        unimplemented!(slo);
    }
    fn rla<M: AddressingMode>(&mut self, _: M) {
        unimplemented!(rla);
    }
    fn sre<M: AddressingMode>(&mut self, _: M) {
        unimplemented!(sre);
    }
    fn rra<M: AddressingMode>(&mut self, _: M) {
        unimplemented!(rra);
    }
    fn kil(&mut self) {
        unimplemented!(kil);
    }

    fn stack_push_w(&mut self, val: u16) {
        let low = (val & 0x00FF) as u8;
        let high = ((val & 0xFF00) >> 8) as u8;
        dynasm!( self.asm
            ; sub n_sp, BYTE 2
            ; mov BYTE [ram + r13 + 0x101], BYTE low as _
            ; mov BYTE [ram + r13 + 0x102], BYTE high as _
        )
    }

    fn relative_addr(&self, disp: u8) -> u16 {
        let disp = (disp as i8) as i16; //We want to sign-extend here.
        let pc = self.pc as i16;
        pc.wrapping_add(disp) as u16
    }

    fn read_incr_pc(&mut self) -> u8 {
        let pc = self.pc;
        let val: u8 = self.cpu.read(pc);
        self.pc = self.pc.wrapping_add(1);
        val
    }

    fn read_w_incr_pc(&mut self) -> u16 {
        self.read_incr_pc() as u16 | ((self.read_incr_pc() as u16) << 8)
    }

    fn get_branch_target_label(&mut self) -> DynamicLabel {
        let arg = self.read_incr_pc();
        let target = self.relative_addr(arg);
        self.get_dynamic_label(target)
    }

    fn get_dynamic_label(&mut self, address: u16) -> DynamicLabel {
        match self.branch_targets.get(&address).cloned() {
            Some(label) => label,
            None => {
                let label = self.asm.new_dynamic_label();
                self.branch_targets.insert( address, label.clone());
                label
            },
        }
    }
}
