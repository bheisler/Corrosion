use super::Compiler;
use dynasmrt::{AssemblyOffset, DynasmApi, DynasmLabelApi, ExecutableBuffer};
use memory::MemSegment;
use cpu::CPU;

pub extern "win64" fn read_memory(cpu: *mut CPU, addr: u16) -> u8 {
    unsafe { (*cpu).read(addr) }
}

// Expects the 6502 address in rcx and returns the byte in r8 (arg)
macro_rules! call_read {
    ($this:ident) => {dynasm!($this.asm
        ; push rax
        ; push rcx
        ; push rdx
        ; push r9
        ; push r10
        ; push r11
        ; mov rdx, rcx // Move the 6502 address to the second argument register
        ; mov rax, QWORD ::cpu::x86_64_compiler::addressing_modes::read_memory as _
        ; mov rcx, rbx //Pointer to CPU is first arg
        ; sub rsp, 0x20
        ; call rax
        ; add rsp, 0x20
        ; mov r8, rax //rax contains returned value, move it to r8 (which is arg)
        ; pop r11
        ; pop r10
        ; pop r9
        ; pop rdx
        ; pop rcx
        ; pop rax
    );};
}

// Optimized version of call_write that checks if the address is in RAM and if
// so does the
// write directly. Useful when this check can't be performed statically.
macro_rules! fast_read {
    ($this:ident) => {dynasm!($this.asm
        ; cmp cx, WORD 0x1FFF
        ; jg >slow_read
        ; and rcx, DWORD 0x07FF
        ; mov arg, [ram + rcx]
        ; jmp >next
        ; slow_read:
        ;; call_read!($this)
        ; next:
);};
}

pub extern "win64" fn write_memory(cpu: *mut CPU, addr: u16, val: u8) {
    unsafe { (*cpu).write(addr, val) }
}


// Expects the 6502 address in rcx and the value in r8 (arg)
macro_rules! call_write {
    ($this:ident) => {dynasm!($this.asm
        ; push rax
        ; push rcx
        ; push rdx
        ; push r8
        ; push r9
        ; push r10
        ; push r11
        ; mov rdx, rcx // Move the 6502 address to the second argument register
        ; mov rax, QWORD ::cpu::x86_64_compiler::addressing_modes::write_memory as _
        ; mov rcx, rbx //Pointer to CPU is first arg
        //Conveniently, we already have the value in r8
        ; sub rsp, 0x20
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

// Optimized version of call_write that checks if the address is in RAM and if
// so does the
// write directly. Useful when this check can't be performed statically.
macro_rules! fast_write {
    ($this:ident) => {dynasm!($this.asm
        ; cmp cx, WORD 0x1FFF
        ; jg >slow_write
        ; and rcx, DWORD 0x07FF
        ; mov [ram + rcx], arg
        ; jmp >next
        ; slow_write:
        ;; call_write!($this)
        ; next:
    );};
}
pub trait AddressingMode {
    fn read_to_arg(&self, comp: &mut Compiler);
    fn write_from_arg(&self, comp: &mut Compiler);
}

#[derive(Debug, Copy, Clone)]
struct ImmediateAddressingMode;
impl AddressingMode for ImmediateAddressingMode {
    fn read_to_arg(&self, comp: &mut Compiler) {
        let imm_arg = comp.read_incr_pc() as i8;
        dynasm!{comp.asm
            ; mov arg, BYTE imm_arg
        }
    }
    fn write_from_arg(&self, _: &mut Compiler) {
        panic!("Tried to write to an immediate address.")
    }
}

#[derive(Debug, Copy, Clone)]
struct ZeroPageAddressingMode {
    addr: u8,
}
impl AddressingMode for ZeroPageAddressingMode {
    fn read_to_arg(&self, comp: &mut Compiler) {
        dynasm!{comp.asm
            ; mov arg, [ram + self.addr as _]
        }
    }
    fn write_from_arg(&self, comp: &mut Compiler) {
        let offset = self.addr as usize;
        dynasm!{comp.asm
            ; mov [ram + self.addr as _], arg
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct ZeroPageXAddressingMode {
    addr: u8,
}
impl AddressingMode for ZeroPageXAddressingMode {
    fn read_to_arg(&self, comp: &mut Compiler) {
        dynasm!{comp.asm
            ; mov rcx, DWORD self.addr as _
            ; add rcx, r10
            ; mov arg, [ram + rcx]
        }
    }
    fn write_from_arg(&self, comp: &mut Compiler) {
        let offset = self.addr as usize;
        dynasm!{comp.asm
            ; mov rcx, DWORD self.addr as _
            ; add rcx, r10
            ; mov [ram + rcx], arg
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct ZeroPageYAddressingMode {
    addr: u8,
}
impl AddressingMode for ZeroPageYAddressingMode {
    fn read_to_arg(&self, comp: &mut Compiler) {
        dynasm!{comp.asm
            ; mov rcx, DWORD self.addr as _
            ; add rcx, r11
            ; mov arg, [ram + rcx]
        }
    }
    fn write_from_arg(&self, comp: &mut Compiler) {
        let offset = self.addr as usize;
        dynasm!{comp.asm
            ; mov rcx, DWORD self.addr as _
            ; add rcx, r11
            ; mov [ram + rcx], arg
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct AbsoluteAddressingMode {
    addr: u16,
}
impl AddressingMode for AbsoluteAddressingMode {
    fn read_to_arg(&self, comp: &mut Compiler) {
        if (self.addr < 0x2000) {
            let ram_address = self.addr % 0x800;
            dynasm!{comp.asm
                ; mov arg, [ram + ram_address as _]
            }
        } else {
            dynasm!{comp.asm
                ; mov rcx, QWORD self.addr as _
                ;; call_read!(comp)
            }
        }
    }

    fn write_from_arg(&self, comp: &mut Compiler) {
        if (self.addr < 0x2000) {
            let ram_address = self.addr % 0x800;
            dynasm!{comp.asm
                ; mov [ram + ram_address as _], arg
            }
        } else {
            dynasm!{comp.asm
                ; mov rcx, QWORD self.addr as _
                ;; call_write!(comp)
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct AbsoluteXAddressingMode {
    addr: u16,
}
impl AddressingMode for AbsoluteXAddressingMode {
    fn read_to_arg(&self, comp: &mut Compiler) {
        // adding X might make it step outside of RAM
        if (self.addr < 0x1F00) {
            let ram_address = self.addr % 0x800;
            dynasm!{comp.asm
                ; mov rcx, ram_address as _
                ; add rcx, r10
                ; mov arg, [ram + rcx]
            }
        } else {
            dynasm!{comp.asm
                ; mov rcx, QWORD self.addr as _
                ; add rcx, r10
                ;; call_read!(comp)
            }
        }
    }

    fn write_from_arg(&self, comp: &mut Compiler) {
        // adding X might make it step outside of RAM
        if (self.addr < 0x1F00) {
            let ram_address = self.addr % 0x800;
            dynasm!{comp.asm
                    ; mov rcx, ram_address as _
                    ; add rcx, r10
                    ; mov [ram + rcx], arg
                }
        } else {
            dynasm!{comp.asm
                    ; mov rcx, QWORD self.addr as _
                    ; add rcx, r10
                    ;; call_write!(comp)
                }
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct AbsoluteYAddressingMode {
    addr: u16,
}
impl AddressingMode for AbsoluteYAddressingMode {
    fn read_to_arg(&self, comp: &mut Compiler) {
        // adding Y might make it step outside of RAM
        if (self.addr < 0x1F00) {
            let ram_address = self.addr % 0x800;
            dynasm!{comp.asm
                ; mov rcx, ram_address as _
                ; add rcx, r11
                ; mov arg, [ram + rcx]
            }
        } else {
            dynasm!{comp.asm
                ; mov rcx, QWORD self.addr as _
                ; add rcx, r11
                ;; call_read!(comp)
            }
        }
    }

    fn write_from_arg(&self, comp: &mut Compiler) {
        // adding Y might make it step outside of RAM
        if (self.addr < 0x1F00) {
            let ram_address = self.addr % 0x800;
            dynasm!{comp.asm
                    ; mov rcx, ram_address as _
                    ; add rcx, r11
                    ; mov [ram + rcx], arg
                }
        } else {
            dynasm!{comp.asm
                    ; mov rcx, QWORD self.addr as _
                    ; add rcx, r11
                    ;; call_write!(comp)
                }
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct AccumulatorAddressingMode;
impl AddressingMode for AccumulatorAddressingMode {
    fn read_to_arg(&self, comp: &mut Compiler) {
        dynasm!{comp.asm
            ; mov arg, n_a
        }
    }
    fn write_from_arg(&self, comp: &mut Compiler) {
        dynasm!{comp.asm
            ; mov n_a, arg
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct IndirectXAddressingMode {
    addr: u8,
}
impl IndirectXAddressingMode {
    fn calc_addr(&self, comp: &mut Compiler) {
        dynasm!{comp.asm
            ; xor rcx, rcx
            ; mov cl, n_x
            ; add cl, self.addr as _
            ; mov al, BYTE [ram + rcx]
            ; inc cl
            ; mov ah, BYTE [ram + rcx]
            ; mov cx, ax
        }
    }
}
impl AddressingMode for IndirectXAddressingMode {
    fn read_to_arg(&self, comp: &mut Compiler) {
        dynasm!{comp.asm
            ;; self.calc_addr(comp)
            ;; fast_read!(comp)
        }
    }
    fn write_from_arg(&self, comp: &mut Compiler) {
        dynasm!{comp.asm
            ;; self.calc_addr(comp)
            ;; fast_write!(comp)
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct IndirectYAddressingMode {
    addr: u8,
}
impl IndirectYAddressingMode {
    fn calc_addr(&self, comp: &mut Compiler) {
        dynasm!{comp.asm
            ; xor rcx, rcx
            ; mov cl, self.addr as _
            ; mov al, BYTE [ram + rcx]
            ; inc cl
            ; mov ah, BYTE [ram + rcx]
            ; mov cx, ax
            ; add cx, self.addr as _
        }
    }
}
impl AddressingMode for IndirectYAddressingMode {
    fn read_to_arg(&self, comp: &mut Compiler) {
        dynasm!{comp.asm
            ;; self.calc_addr(comp)
            ;; fast_read!(comp)
        }
    }
    fn write_from_arg(&self, comp: &mut Compiler) {
        dynasm!{comp.asm
            ;; self.calc_addr(comp)
            ;; fast_write!(comp)
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct DummyAddressingMode;
impl AddressingMode for DummyAddressingMode {
    fn read_to_arg(&self, _: &mut Compiler) {
        panic!("Tried to use DummyAddressingMode")
    }
    fn write_from_arg(&self, _: &mut Compiler) {
        panic!("Tried to use DummyAddressingMode")
    }
}

impl<'a> Compiler<'a> {
    pub fn immediate(&mut self) -> ImmediateAddressingMode {
        ImmediateAddressingMode
    }
    pub fn absolute(&mut self) -> AbsoluteAddressingMode {
        AbsoluteAddressingMode { addr: self.read_w_incr_pc() }
    }
    pub fn absolute_x(&mut self) -> AbsoluteXAddressingMode {
        AbsoluteXAddressingMode { addr: self.read_w_incr_pc() }
    }
    pub fn absolute_y(&mut self) -> AbsoluteXAddressingMode {
        AbsoluteXAddressingMode { addr: self.read_w_incr_pc() }
    }
    pub fn zero_page(&mut self) -> ZeroPageAddressingMode {
        ZeroPageAddressingMode { addr: self.read_incr_pc() }
    }
    pub fn zero_page_x(&mut self) -> ZeroPageXAddressingMode {
        ZeroPageXAddressingMode { addr: self.read_incr_pc() }
    }
    pub fn zero_page_y(&mut self) -> ZeroPageYAddressingMode {
        ZeroPageYAddressingMode { addr: self.read_incr_pc() }
    }
    pub fn indirect_x(&mut self) -> IndirectXAddressingMode {
        IndirectXAddressingMode { addr: self.read_incr_pc() }
    }
    pub fn indirect_y(&mut self) -> IndirectYAddressingMode {
        IndirectYAddressingMode { addr: self.read_incr_pc() }
    }
    pub fn accumulator(&mut self) -> AccumulatorAddressingMode {
        AccumulatorAddressingMode
    }
}
