use cpu::CPU;

#[cfg(target_arch = "x86_64")]
use cpu::compiler;

#[cfg(target_arch = "x86_64")]
use cpu::compiler::ExecutableBlock;

use fnv::FnvHashMap;
#[cfg(not(target_arch = "x86_64"))]
pub struct Dispatcher {}
#[cfg(not(target_arch = "x86_64"))]
impl Dispatcher {
    pub fn new() -> Dispatcher {
        Dispatcher {}
    }

    pub fn jump(&mut self, _: &mut CPU) {}

    pub fn dirty(&mut self, _: usize, _: usize) {}
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
struct RomAddress {
    bank: usize,
    offset: u16,
}

#[cfg(target_arch = "x86_64")]
pub struct Dispatcher {
    table: FnvHashMap<RomAddress, Block>,
}
#[cfg(target_arch = "x86_64")]
struct Block {
    code: ExecutableBlock,
}

#[cfg(feature = "debug_features")]
fn disasm_function(cpu: &mut CPU, addr: u16) {
    ::cpu::disasm::Disassembler::new(cpu).disasm_function(addr);
}

#[cfg(not(feature = "debug_features"))]
fn disasm_function(_: &mut CPU, _: u16) {}

impl Default for Dispatcher {
    fn default() -> Dispatcher {
        Dispatcher::new()
    }
}

#[cfg(target_arch = "x86_64")]
impl Dispatcher {
    pub fn new() -> Dispatcher {
        Dispatcher {
            table: FnvHashMap::default(),
        }
    }

    fn put(&mut self, start_addr: RomAddress, code: ExecutableBlock) -> &Block {
        self.table.insert(start_addr, Block { code: code });
        self.table.get(&start_addr).unwrap()
    }

    fn get_rom_addr(&self, addr: u16, cpu: &CPU) -> RomAddress {
        let rom_bank = unsafe { (*cpu.cart.get()).prg_rom_bank_id(addr) };
        RomAddress {
            bank: rom_bank,
            offset: addr & 0xFFF,
        }
    }

    pub fn jump(&mut self, cpu: &mut CPU) {
        let addr = cpu.regs.pc;
        let executable = &self.get_block(addr, cpu).code;
        executable.call(cpu);
    }

    fn get_block(&mut self, addr: u16, cpu: &mut CPU) -> &Block {
        let rom_addr = self.get_rom_addr(addr, cpu);
        if self.should_compile(rom_addr) {
            self.compile(addr, cpu)
        } else {
            self.table.get(&rom_addr).unwrap()
        }
    }

    fn should_compile(&self, addr: RomAddress) -> bool {
        !self.table.contains_key(&addr)
    }

    fn compile(&mut self, addr: u16, cpu: &mut CPU) -> &Block {
        if cpu.settings.disassemble_functions {
            disasm_function(cpu, addr);
        }
        let executable = compiler::compile(addr, cpu);
        let rom_addr = self.get_rom_addr(addr, cpu);
        self.put(rom_addr, executable)
    }
}
