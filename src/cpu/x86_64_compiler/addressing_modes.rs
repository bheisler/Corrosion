use super::Compiler;
use dynasmrt::{AssemblyOffset, DynasmApi, DynasmLabelApi, ExecutableBuffer};

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
struct DummyAddressingMode;
impl AddressingMode for DummyAddressingMode {
    fn read_to_arg(&self, comp: &mut Compiler) {
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
    pub fn absolute(&mut self) -> DummyAddressingMode {
        self.read_w_incr_pc();
        unimplemented!(absolute);
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
    pub fn accumulator(&mut self) -> DummyAddressingMode {
        unimplemented!(accumulator);
    }
}
