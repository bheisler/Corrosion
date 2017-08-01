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

#[cfg(target_arch = "x86_64")]
pub struct Dispatcher {
    table: FnvHashMap<u16, Block>,
}
#[cfg(target_arch = "x86_64")]
struct Block {
    dirty: bool,
    start_addr: u16,
    end_addr: u16,
    code: ExecutableBlock,
}
#[cfg(target_arch = "x86_64")]
impl Block {
    fn overlaps_with(&self, start: usize, end: usize) -> bool {
        (self.start_addr as usize) < end || (self.end_addr as usize) >= start
    }
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

    fn put(&mut self, start_addr: u16, end_addr: u16, code: ExecutableBlock) -> &Block {
        self.table.insert(
            start_addr,
            Block {
                dirty: false,
                start_addr: start_addr,
                end_addr: end_addr,
                code: code,
            },
        );
        self.table.get(&start_addr).unwrap()
    }

    pub fn jump(&mut self, cpu: &mut CPU) {
        let addr = cpu.regs.pc;
        let executable = &self.get_block(addr, cpu).code;
        executable.call(cpu);
    }

    fn get_block(&mut self, addr: u16, cpu: &mut CPU) -> &Block {
        if self.should_compile(addr) {
            self.compile(addr, cpu)
        } else {
            self.table.get(&addr).unwrap()
        }
    }

    fn should_compile(&self, addr: u16) -> bool {
        self.table.get(&addr).map_or(true, |b| b.dirty)
    }

    fn compile(&mut self, addr: u16, cpu: &mut CPU) -> &Block {
        if cpu.settings.disassemble_functions {
            disasm_function(cpu, addr);
        }
        let (end_addr, executable) = compiler::compile(addr, cpu);
        self.put(addr, end_addr, executable)
    }

    pub fn dirty(&mut self, start: usize, end: usize) {
        for block in self.table.values_mut() {
            if block.overlaps_with(start, end) {
                block.dirty = true;
            }
        }
    }
}
