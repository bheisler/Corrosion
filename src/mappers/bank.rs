use std::ops::Range;

pub struct RomBank {
    data: Box<[u8]>,
}

impl RomBank {
    pub fn new(data: Vec<u8>) -> RomBank {
        if data.len() != 0x1000 {
            panic!("Unexpected bank size {}", data.len());
        }

        RomBank { data: data.into_boxed_slice() }
    }

    pub fn read(&self, idx: u16) -> u8 {
        unsafe { *self.data.get_unchecked((idx & 0x0FFF) as usize) }
    }

    pub fn write(&mut self, _: u16, _: u8) {
        // Do Nothing
    }
}


pub struct MappingTable {
    // All banks of ROM
    banks: Box<[RomBank]>,
    // Mappings from CPU addresses to bank indexes.
    // Indexed in terms of pages starting at 0x8000.
    mappings: [usize; 8],
}

fn to_page_num(addr: u16) -> usize {
    ((addr >> 12) & 0b0111) as usize
}

impl MappingTable {
    pub fn new(rom: Vec<u8>) -> MappingTable {
        let mut banks: Vec<RomBank> = vec![];
        let bank_count = rom.len() / 0x1000;
        let mut remaining_rom = rom;
        for _ in 0..bank_count {
            let mut current_bank = remaining_rom;
            remaining_rom = current_bank.split_off(0x1000);
            banks.push(RomBank::new(current_bank));
        }

        MappingTable {
            banks: banks.into_boxed_slice(),
            mappings: [0; 8],
        }
    }

    pub fn get_bank(&self, addr: u16) -> &RomBank {
        let index = self.mappings[to_page_num(addr)];
        &self.banks[index]
    }

    pub fn get_bank_mut(&mut self, addr: u16) -> &mut RomBank {
        let index = self.mappings[to_page_num(addr)];
        &mut self.banks[index]
    }

    pub fn bank_count(&self) -> usize {
        self.banks.len()
    }

    pub fn map_page(&mut self, page: usize, bank: usize) {
        self.mappings[page] = bank;
    }

    pub fn map_pages_linear(&mut self, range: Range<usize>, starting_bank: usize) {
        let mut cur_bank = starting_bank;
        for page in range {
            self.mappings[page] = cur_bank;
            cur_bank += 1;
        }
    }
}
