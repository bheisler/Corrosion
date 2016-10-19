use super::Compiler;
use dynasmrt::{AssemblyOffset, DynasmApi, DynasmLabelApi, ExecutableBuffer};
use memory::MemSegment;
use cpu::CPU;

extern "win64" fn read_memory(cpu: *mut CPU, addr: u16) -> u8 {
    unsafe { (*cpu).read(addr) }
}

macro_rules! call_read {
    ($this:ident, $addr:expr) => {dynasm!($this.asm
        ; push rax
        ; push rcx
        ; push rdx
        ; push r9
        ; push r10
        ; push r11
        ; mov rax, QWORD read_memory as _
        ; mov rcx, rbx //Pointer to CPU is first arg
        ; mov rdx, QWORD $addr as _ //6502 address is second arg
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

extern "win64" fn write_memory(cpu: *mut CPU, addr: u16, val: u8) {
    unsafe { (*cpu).write(addr, val) }
}

macro_rules! call_write {
    ($this:ident, $addr:expr) => {dynasm!($this.asm
        ; push rax
        ; push rcx
        ; push rdx
        ; push r8
        ; push r9
        ; push r10
        ; push r11
        ; mov rax, QWORD write_memory as _
        ; mov rcx, rbx //Pointer to CPU is first arg
        ; mov rdx, QWORD $addr as _ //6502 address is second arg
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
            call_read!(comp, self.addr)
        }
    }

    fn write_from_arg(&self, comp: &mut Compiler) {
        if (self.addr < 0x2000) {
            let ram_address = self.addr % 0x800;
            dynasm!{comp.asm
                ; mov [ram + ram_address as _], arg
            }
        } else {
            call_write!(comp, self.addr)
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
    pub fn absolute_x(&mut self) -> DummyAddressingMode {
        self.read_w_incr_pc();
        unimplemented!(absolute_x);
    }
    pub fn absolute_y(&mut self) -> DummyAddressingMode {
        self.read_w_incr_pc();
        unimplemented!(absolute_y);
    }
    pub fn zero_page(&mut self) -> ZeroPageAddressingMode {
        ZeroPageAddressingMode { addr: self.read_incr_pc() }
    }
    pub fn zero_page_x(&mut self) -> DummyAddressingMode {
        self.read_incr_pc();
        unimplemented!(zero_page_x);
    }
    pub fn zero_page_y(&mut self) -> DummyAddressingMode {
        self.read_incr_pc();
        unimplemented!(zero_page_y);
    }
    pub fn indirect_x(&mut self) -> DummyAddressingMode {
        self.read_incr_pc();
        unimplemented!(indirect_x);
    }
    pub fn indirect_y(&mut self) -> DummyAddressingMode {
        self.read_incr_pc();
        unimplemented!(indirect_y);
    }
    pub fn accumulator(&mut self) -> AccumulatorAddressingMode {
        AccumulatorAddressingMode
    }
}
