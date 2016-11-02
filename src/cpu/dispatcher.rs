use cpu::CPU;

#[cfg(feature="jit")]
use cpu::compiler;

#[cfg(feature="jit")]
use cpu::compiler::ExecutableBlock;

#[cfg(not(feature="jit"))]
pub struct Dispatcher {

}
#[cfg(not(feature="jit"))]
impl Dispatcher {
    pub fn new() -> Dispatcher {
        Dispatcher {}
    }

    pub fn jump(&mut self, _: &mut CPU) {}

    pub fn dirty(&mut self, _: usize, _: usize) {}
}

#[cfg(feature="jit")]
pub struct Dispatcher {
    table: Box<[Option<Block>]>,
}
#[cfg(feature="jit")]
struct Block {
    dirty: bool,
    start_addr: u16,
    end_addr: u16,
    code: ExecutableBlock,
}
#[cfg(feature="jit")]
impl Block {
    fn overlaps_with(&self, start: usize, end: usize) -> bool {
        if (self.end_addr as usize) < start {
            false
        } else if (self.start_addr as usize) >= end {
            false
        } else {
            true
        }
    }
}

#[cfg(feature="function_disasm")]
fn disasm_function(cpu: &mut CPU, addr: u16) {
    ::cpu::disasm::Disassembler::new(cpu).disasm_function(addr);
}

#[cfg(not(feature="function_disasm"))]
fn disasm_function(_: &mut CPU, _: u16) {}

#[cfg(feature="jit")]
impl Dispatcher {
    pub fn new() -> Dispatcher {
        unsafe {
            use std::ptr;
            use std::mem;

            let mut table: Vec<Option<Block>> = vec![];
            table.reserve_exact(0x10000);
            for _ in 0..0x10000 {
                table.push(None);
            }

            Dispatcher { table: table.into_boxed_slice() }
        }
    }

    fn put(&mut self, start_addr: u16, end_addr: u16, code: ExecutableBlock) -> &Block {
        self.table[start_addr as usize] = Some(Block {
            dirty: false,
            start_addr: start_addr,
            end_addr: end_addr,
            code: code,
        });
        self.table[start_addr as usize].as_ref().unwrap()
    }

    pub fn jump(&mut self, cpu: &mut CPU) {
        let addr = cpu.regs.pc;
        let executable = &self.get_block(addr, cpu).code;
        executable.call(cpu as *mut CPU);
    }

    fn get_block(&mut self, addr: u16, cpu: &mut CPU) -> &Block {
        if self.should_compile(addr) {
            self.compile(addr, cpu)
        } else {
            self.table[addr as usize].as_ref().unwrap()
        }
    }

    fn should_compile(&self, addr: u16) -> bool {
        self.table[addr as usize]
            .as_ref()
            .map_or(true, |b| b.dirty)
    }

    fn compile(&mut self, addr: u16, cpu: &mut CPU) -> &Block {
        disasm_function(cpu, addr);
        let (end_addr, executable) = compiler::compile(addr, cpu);
        self.put(addr, end_addr, executable)
    }

    pub fn dirty(&mut self, start: usize, end: usize) {
        for opt_block in self.table.iter_mut() {
            if let Some(ref mut block) = *opt_block {
                if block.overlaps_with(start, end) {
                    block.dirty = true;
                }
            }
        }
    }
}
