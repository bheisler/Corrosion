use cpu::dispatcher::Dispatcher;
use std::cell::UnsafeCell;
use std::ops::Range;
use std::rc::Rc;

pub struct RomBank {
    data: Box<[u8]>,
}

const BANK_SIZE: usize = 0x1000;
const FIRST_PAGE: usize = 0x8000;

impl RomBank {
    pub fn new(data: Vec<u8>) -> RomBank {
        if data.len() != BANK_SIZE {
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

    dispatcher: Option<Rc<UnsafeCell<Dispatcher>>>,
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
            dispatcher: None,
        }
    }

    pub fn set_dispatcher(&mut self, dispatcher: Rc<UnsafeCell<Dispatcher>>) {
        if self.dispatcher.is_some() {
            panic!("Tried to set the dispatcher twice.");
        }
        self.dispatcher = Some(dispatcher);
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
        if let Some(ref mut dispatcher) = self.dispatcher {
            let start = page * BANK_SIZE + FIRST_PAGE;
            let end = start + BANK_SIZE;
            unsafe { (*dispatcher.get()).dirty(start, end) }
        }
    }

    pub fn map_pages_linear(&mut self, range: Range<usize>, starting_bank: usize) {
        let start = range.start * BANK_SIZE + FIRST_PAGE;
        let end = range.end * BANK_SIZE + FIRST_PAGE;

        let mut cur_bank = starting_bank;
        for page in range {
            self.mappings[page] = cur_bank;
            cur_bank += 1;
        }
        if let Some(ref mut dispatcher) = self.dispatcher {
            unsafe { (*dispatcher.get()).dirty(start, end) }
        }
    }
}
