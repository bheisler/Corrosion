use cpu::CPU;

#[cfg(target_arch = "x86_64")]
use cpu::compiler;

#[cfg(target_arch = "x86_64")]
use cpu::compiler::ExecutableBlock;
use fnv::{FnvHashMap, FnvHashSet};

#[cfg(target_arch = "x86_64")]
use mappers::RomAddress;

#[cfg(not(target_arch = "x86_64"))]
pub struct Dispatcher {}
#[cfg(not(target_arch = "x86_64"))]
impl Dispatcher {
    pub fn new() -> Dispatcher {
        Dispatcher {}
    }

    pub fn jump(&mut self, _: &mut CPU) {}
}

#[cfg(target_arch = "x86_64")]
pub struct Dispatcher {
    table: FnvHashMap<RomAddress, Block>,
    compiling: FnvHashSet<RomAddress>,
}
#[cfg(target_arch = "x86_64")]
struct Block {
    code: ExecutableBlock,
    locked: bool,
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
            compiling: FnvHashSet::default(),
        }
    }

    fn get_rom_addr(&self, addr: u16, cpu: &CPU) -> RomAddress {
        unsafe { (*cpu.cart.get()).prg_rom_address(addr) }
    }

    pub fn jump(&mut self, cpu: &mut CPU) {
        let addr = cpu.regs.pc;
        let executable = &self.get_block(addr, cpu).code;
        executable.call(cpu);
    }

    pub fn lock_block(
        &mut self,
        target_addr: u16,
        caller_addr: u16,
        cpu: &mut CPU,
    ) -> Option<&ExecutableBlock> {
        if target_addr < 0x8000 {
            return None;
        }

        let target_rom_addr = self.get_rom_addr(target_addr, cpu);
        if self.compiling.contains(&target_rom_addr) {
            // Prevent infinite recursion.
            None
        } else if target_rom_addr.window_id == self.get_rom_addr(caller_addr, cpu).window_id {
            if self.should_compile(&target_rom_addr) {
                self.compile(target_addr, &target_rom_addr, cpu);
            }
            self.table.get_mut(&target_rom_addr).unwrap().locked = true;
            Some(&self.table.get(&target_rom_addr).unwrap().code)
        } else {
            None
        }
    }

    fn get_block(&mut self, addr: u16, cpu: &mut CPU) -> &Block {
        let rom_addr = self.get_rom_addr(addr, cpu);
        if self.should_compile(&rom_addr) {
            self.compile(addr, &rom_addr, cpu);
        }
        self.table.get(&rom_addr).unwrap()
    }

    fn should_compile(&self, addr: &RomAddress) -> bool {
        !self.table.contains_key(addr)
    }

    fn compile(&mut self, addr: u16, rom_addr: &RomAddress, cpu: &mut CPU) {
        if cpu.settings.disassemble_functions {
            disasm_function(cpu, addr);
        }

        self.compiling.insert(rom_addr.clone());

        let executables = compiler::compile(addr, cpu, self);
        for (addr, block) in executables {
            let rom_addr = self.get_rom_addr(addr, cpu);

            // Don't overwrite (and therefore drop) locked blocks, they're linked to other
            // blocks.
            // TODO: Track those links and patch them to the new address
            if let Some(block) = self.table.get(&rom_addr) {
                if block.locked {
                    continue;
                }
            }

            self.table.insert(
                rom_addr,
                Block {
                    code: block,
                    locked: false,
                },
            );
        }

        self.compiling.remove(rom_addr);
    }
}
