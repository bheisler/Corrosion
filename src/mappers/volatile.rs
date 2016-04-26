use memory::MemSegment;

pub struct VolatileRam {
    data: Box<[u8]>,
}

impl VolatileRam {
    pub fn new(size: usize) -> VolatileRam {
        VolatileRam { data: vec![0u8; size].into_boxed_slice() }
    }

    fn wrap_addr(&self, idx: u16) -> usize {
        let idx = idx as usize;
        idx % self.data.len()
    }
}

impl MemSegment for VolatileRam {
    fn read(&mut self, idx: u16) -> u8 {
        self.data[self.wrap_addr(idx)]
    }
    fn write(&mut self, idx: u16, val: u8) {
        self.data[self.wrap_addr(idx)] = val;
    }
}
