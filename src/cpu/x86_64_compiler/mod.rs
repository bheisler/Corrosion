#![allow(unneeded_field_pattern)]

use cpu::CPU;
use cpu::Registers;
use cpu::JitInterrupt;
use cpu::nes_analyst::Analyst;
use cpu::nes_analyst::BlockAnalysis;
use cpu::nes_analyst::InstructionAnalysis;
use cpu::IRQ_VECTOR;
use cpu::CYCLE_TABLE;
use std::mem;
use memory::MemSegment;
use std::collections::HashMap;
use self::addressing_modes::NoTickMode;

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
        let f: extern "win64" fn(*mut CPU, *mut [u8; 0x800]) -> () =
            unsafe { mem::transmute(self.buffer.ptr(offset)) };
        let ram = unsafe { &mut (*cpu).ram };
        f(cpu, ram as _);
    }
}

pub fn compile(addr: u16, cpu: &mut CPU) -> (u16, ExecutableBlock) {
    let analysis = Analyst::new(cpu).analyze(addr);
    let end_addr = analysis.exit_point;
    (end_addr, Compiler::new(cpu, analysis).compile_block())
}

macro_rules! unimplemented {
    ($opcode:ident) => {
        panic!(stringify!(Unknown or unimplemented operation $opcode));
    };
}

//rcx and sub-sections thereof are the general-purpose scratch register.
//Sometimes r8 and rax are used as scratch registers as well
dynasm!(this
    ; .alias cpu, rbx
    ; .alias ram, rdx
    ; .alias arg, r8b
    ; .alias arg_w, r8w
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
            ; lea rcx, cpu => CPU.regs
            ; xor r8, r8
            ; xor r9, r9
            ; mov n_a, BYTE rcx => Registers.a
            ; xor r10, r10
            ; mov n_x, BYTE rcx => Registers.x
            ; xor r11, r11
            ; mov n_y, BYTE rcx => Registers.y
            ; xor r12, r12
            ; mov n_p, BYTE rcx => Registers.p
            ; xor r13, r13
            ; mov n_sp, BYTE rcx => Registers.sp
            ; xor r14, r14
            ; mov n_pc, WORD rcx => Registers.pc
            ; mov cyc, QWORD cpu => CPU.cycle
        );
    }};
}

macro_rules! store_registers {
    ($this:ident) => {{
        dynasm!($this.asm
            ; lea rcx, cpu => CPU.regs
            ; mov BYTE rcx => Registers.a, n_a
            ; mov BYTE rcx => Registers.x, n_x
            ; mov BYTE rcx => Registers.y, n_y
            ; mov BYTE rcx => Registers.p, n_p
            ; mov BYTE rcx => Registers.sp, n_sp
            ; mov WORD rcx => Registers.pc, n_pc
            ; mov QWORD cpu => CPU.cycle, cyc
        );
    }};
}

#[cfg(feature="cputrace")]
macro_rules! call_trace {
    ($this:ident) => {dynasm!($this.asm
        ; mov n_pc, WORD $this.pc as _
        ;; store_registers!($this)
        ; push rax
        ; push rcx
        ; push rdx
        ; push r9
        ; push r10
        ; push r11
        ; mov rax, QWORD ::cpu::x86_64_compiler::trace as _
        ; mov rcx, rbx //Pointer to CPU is first arg
        ; sub rsp, 0x20
        ; call rax
        ; add rsp, 0x20
        ; pop r11
        ; pop r10
        ; pop r9
        ; pop rdx
        ; pop rcx
        ; pop rax
    );};
}

#[cfg(not(feature="cputrace"))]
macro_rules! call_trace {
    ($this:ident) => {};
}

#[cfg(feature="cputrace")]
pub extern "win64" fn trace(cpu: *mut CPU) {
    unsafe { (*cpu).trace() }
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
            ret
        1:
            or r12b, 2H
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
            ret
        1:
            and r12b, 7FH
            "
        :
        :
        : "r12"
        : "intel");
    };
}

#[macro_use]
mod addressing_modes;

use self::addressing_modes::AddressingMode;

struct Compiler<'a> {
    asm: ::dynasmrt::Assembler,
    cpu: &'a mut CPU,
    analysis: BlockAnalysis,

    pc: u16,
    current_instruction: u16,

    branch_targets: HashMap<u16, DynamicLabel>,
}

impl<'a> Compiler<'a> {
    fn new(cpu: &'a mut CPU, analysis: BlockAnalysis) -> Compiler<'a> {
        let entry_point = analysis.entry_point;
        Compiler {
            asm: ::dynasmrt::Assembler::new(),
            cpu: cpu,

            pc: entry_point,
            current_instruction: entry_point,
            analysis: analysis,

            branch_targets: HashMap::new(),
        }
    }

    fn compile_block(mut self) -> ExecutableBlock {
        let start = self.asm.offset();

        self.prologue();

        while self.pc <= self.analysis.exit_point {
            self.current_instruction = self.pc;
            self.emit_branch_target();
            self.check_for_interrupt();

            self.do_call_trace();

            let opcode = self.read_incr_pc();
            self.emit_cycle_count(opcode);
            decode_opcode!(opcode, self);
        }

        ExecutableBlock {
            offset: start,
            buffer: self.asm.finalize().unwrap(),
        }
    }

    fn prologue(&mut self) {
        dynasm!{self.asm
            ; push rbx
            ; push r12
            ; push r13
            ; push r14
            ; push r15
            ; mov rbx, rcx //Move the CPU pointer to the CPU pointer register
            //Leave the RAM pointer in the RAM pointer register
        }
        load_registers!(self);
    }

    fn emit_branch_target(&mut self) {
        if self.get_current_instr_analysis().is_branch_target {
            let temp_pc = self.current_instruction;
            let target_label = self.get_dynamic_label(temp_pc);
            dynasm!{self.asm
                ; => target_label
            }
        }
    }

    fn emit_cycle_count(&mut self, opcode: u8) {
        let cycles = CYCLE_TABLE[opcode as usize];
        dynasm!(self.asm
            ; add cyc, cycles as _
        )
    }

    fn do_call_trace(&mut self) {
        call_trace!(self);
    }

    fn check_for_interrupt(&mut self) {
        dynasm!{self.asm
            ; lea rcx, cpu => CPU.interrupt
            ; mov rcx, rcx => JitInterrupt.next_interrupt
            ; cmp cyc, rcx
            ; jnae >next
            ; mov n_pc, WORD self.pc as _
            ;; epilogue!(self)
            ; next:
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
            ;; mode.read_to_arg(self, true)
            ; mov n_x, arg
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn lda<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self, true)
            ; mov n_a, arg
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn ldy<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self, true)
            ; mov n_y, arg
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }

    // Logic/Math Ops
    fn bit<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self, false)

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
            ;; mode.read_to_arg(self, true)
            ; and arg, n_a
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
            ; mov n_a, arg
        }
    }
    fn ora<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self, true)
            ; or arg, n_a
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
            ; mov n_a, arg
        }
    }
    fn eor<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self, true)
            ; xor arg, n_a
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
            ; mov n_a, arg
        }
    }
    fn adc<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ; xor r8, r8
            ;; mode.read_to_arg(self, true)
            ;; self.do_adc()
        }
    }
    fn sbc<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ; xor r8, r8
            ;; mode.read_to_arg(self, true)
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
            ; ja >set_carry
            ; and n_p, (!CARRY) as _
            ; jmp >next
            ; set_carry:
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
            ; inc rsp
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }

    fn cmp<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self, true)

            ; cmp n_a, arg
            ; jb >clear
            ; or n_p, BYTE CARRY as _
            ; jmp >next
            ; clear:
            ; and n_p, BYTE (!CARRY) as _
            ; next:

            ; mov cl, n_a
            ; sub cl, arg
            ; mov arg, cl
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn cpx<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self, false)

            ; cmp n_x, arg
            ; jb >clear
            ; or n_p, BYTE CARRY as _
            ; jmp >next
            ; clear:
            ; and n_p, BYTE (!CARRY) as _
            ; next:

            ; mov cl, n_x
            ; sub cl, arg
            ; mov arg, cl
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn cpy<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self, false)

            ; cmp n_y, arg
            ; jb >clear
            ; or n_p, BYTE CARRY as _
            ; jmp >next
            ; clear:
            ; and n_p, BYTE (!CARRY) as _
            ; next:

            ; mov cl, n_y
            ; sub cl, arg
            ; mov arg, cl
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
        }
    }
    fn inc<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self, false)
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
            ;; mode.read_to_arg(self, false)
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
    fn lsr<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self, false)
            ; test arg, BYTE 0x01
            ; jz >clear_carry
            ; or n_p, CARRY as _
            ; jmp >next
            ; clear_carry:
            ; and n_p, (!CARRY) as _
            ; next:
            ; shr arg, BYTE 1
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
            ;; mode.write_from_arg(self)
        }
    }
    fn asl<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self, false)
            ; test arg, BYTE 0x80
            ; jz >clear_carry
            ; or n_p, CARRY as _
            ; jmp >next
            ; clear_carry:
            ; and n_p, (!CARRY) as _
            ; next:
            ; shl arg, BYTE 1
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
            ;; mode.write_from_arg(self)
        }
    }
    fn ror<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self, false)
            ; mov al, arg //save original arg
            ; shr arg, BYTE 1
            ; test n_p, CARRY as _
            ; jz >next
            ; or arg, BYTE 0x80
            ; next:

            ; test al, BYTE 0x01
            ; jz >clear_carry
            ; or n_p, CARRY as _
            ; jmp >next
            ; clear_carry:
            ; and n_p, (!CARRY) as _
            ; next:

            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
            ;; mode.write_from_arg(self)
        }
    }
    fn rol<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self, false)
            ; mov al, arg //save original arg
            ; shl arg, BYTE 1
            ; test n_p, CARRY as _
            ; jz >next
            ; or arg, BYTE 0x01
            ; next:

            ; test al, BYTE 0x80
            ; jz >clear_carry
            ; or n_p, CARRY as _
            ; jmp >next
            ; clear_carry:
            ; and n_p, (!CARRY) as _
            ; next:

            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
            ;; mode.write_from_arg(self)
        }
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
        let mut target = self.read_w_incr_pc();
        if target <= 0x1FFF {
            target %= 0x800;
        }
        let page = target & 0xFF00;
        let page_idx = target as u8;

        let lo_addr = target;
        let hi_addr = page | page_idx.wrapping_add(1) as u16;

        if target <= 0x1FFF {
            dynasm!{self.asm
                ; mov al, BYTE [ram + lo_addr as _]
                ; mov ah, BYTE [ram + hi_addr as _]
                ; mov n_pc, ax
            }
        }
        else {
            self.jmpi_slow( lo_addr, hi_addr );
        }
        epilogue!(self);
    }
    fn jmpi_slow(&mut self, lo_addr: u16, hi_addr: u16) {
        dynasm!{self.asm
            ; mov rdx, QWORD hi_addr as _
            ;; call_read!(self)
            ; mov al, arg
            ; mov ah, al
            ; mov rdx, QWORD lo_addr as _
            ;; call_read!(self)
            ; mov al, arg
            ; mov n_pc, ax
        }
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
        let return_addr = self.pc - 1;
        let target = self.cpu.read_w(IRQ_VECTOR);
        dynasm!{ self.asm
            ; mov n_pc, target as _
            ;; self.stack_push_w(return_addr)
            ; mov arg, n_p
            ; or arg, BYTE 0b0011_0000
            ; dec n_sp
            ; mov BYTE [ram + r13 + 0x101], arg
            ;; epilogue!(self)
        }
    }
    fn unsupported(&mut self, opcode: u8) {
        epilogue!(self);
    }

    fn unofficial(&self) {}

    // Branches
    fn bcs(&mut self) {
        dynasm!{self.asm
            ; test n_p, CARRY as _
            ; jz >next
            ;; self.branch()
            ; next:
        }
    }
    fn bcc(&mut self) {
        dynasm!{self.asm
            ; test n_p, CARRY as _
            ; jnz >next
            ;; self.branch()
            ; next:
        }
    }
    fn beq(&mut self) {
        dynasm!{self.asm
            ; test n_p, ZERO as _
            ; jz >next
            ;; self.branch()
            ; next:
        }
    }
    fn bne(&mut self) {
        dynasm!{self.asm
            ; test n_p, ZERO as _
            ; jnz >next
            ;; self.branch()
            ; next:
        }
    }
    fn bvs(&mut self) {
        dynasm!{self.asm
            ; test n_p, OVERFLOW as _
            ; jz >next
            ;; self.branch()
            ; next:
        }
    }
    fn bvc(&mut self) {
        dynasm!{self.asm
            ; test n_p, OVERFLOW as _
            ; jnz >next
            ;; self.branch()
            ; next:
        }
    }
    fn bmi(&mut self) {
        dynasm!{self.asm
            ; test n_p, SIGN as _
            ; jz >next
            ;; self.branch()
            ; next:
        }
    }
    fn bpl(&mut self) {
        dynasm!{self.asm
            ; test n_p, SIGN as _
            ; jnz >next
            ;; self.branch()
            ; next:
        }
    }

    fn branch(&mut self) {
        let (target, cycle) = self.get_branch_target();
        dynasm! {self.asm
            ; inc cyc
            ;; self.branch_page_cycle(cycle)
        }

        if self.analysis.instructions.contains_key(&target) {
            //Target is an instruction in this block
            let target_label = self.get_dynamic_label(target);
            dynasm!{self.asm
                ; jmp =>target_label
            }
        }
        else {
            //Target may be before this block, or misaligned with the instructions in this block.
            //Either way, safest to treat it as a conditional JMP.
            dynasm!{self.asm
                ; mov n_pc, target as _
                ;; epilogue!{self}
            }
        }
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
            ; mov arg, n_a
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
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
    fn u_nop<M: AddressingMode>(&mut self, mode: M) {
        mode.read_to_arg(self, true);
    }
    fn lax<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ;; mode.read_to_arg(self, true)
            ;; call_naked!(self, set_zero_flag)
            ;; call_naked!(self, set_sign_flag)
            ; mov n_a, arg
            ; mov n_x, arg
        }
    }
    fn sax<M: AddressingMode>(&mut self, mode: M) {
        dynasm!{self.asm
            ; mov arg, n_a
            ; and arg, n_x
            ;; mode.write_from_arg(self)
        }
    }
    fn dcp<M: AddressingMode>(&mut self, mode: M) {
        let mode = NoTickMode{ mode: mode };
        self.dec(mode);
        self.cmp(mode);
    }
    fn isc<M: AddressingMode>(&mut self, mode: M) {
        let mode = NoTickMode{ mode: mode };
        self.inc(mode);
        self.sbc(mode);
    }
    fn slo<M: AddressingMode>(&mut self, mode: M) {
        let mode = NoTickMode{ mode: mode };
        self.asl(mode);
        self.ora(mode);
    }
    fn rla<M: AddressingMode>(&mut self, mode: M) {
        let mode = NoTickMode{ mode: mode };
        self.rol(mode);
        self.and(mode);
    }
    fn sre<M: AddressingMode>(&mut self, mode: M) {
        let mode = NoTickMode{ mode: mode };
        self.lsr(mode);
        self.eor(mode);
    }
    fn rra<M: AddressingMode>(&mut self, mode: M) {
        let mode = NoTickMode{ mode: mode };
        self.ror(mode);
        self.adc(mode);
    }
    fn kil(&mut self) {
        dynasm!{self.asm
            ; mov cpu => CPU.halted, BYTE true as _
            ;; epilogue!(self)
        }
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

    fn get_current_instr_analysis(&mut self) -> &InstructionAnalysis {
        let temp = self.current_instruction;
        //There will always be an instruction for the current PC unless something went horribly
        //wrong.
        self.analysis.instructions.get(&temp).unwrap()
    }

    fn get_branch_target(&mut self) -> (u16, bool) {
        let arg = self.read_incr_pc();
        let target = self.relative_addr(arg);

        let do_page_cycle = (self.pc & 0xFF00) != (target & 0xFF00);
        (target, do_page_cycle)
    }
    fn branch_page_cycle(&mut self, do_page_cycle: bool) {
        if do_page_cycle {
            dynasm!{self.asm
                ; inc cyc
            }
        }
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
