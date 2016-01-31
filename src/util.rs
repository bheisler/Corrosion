pub struct ShiftRegister8 {
    bits: u8,
}

impl ShiftRegister8 {
    pub fn new(init: u8) -> ShiftRegister8 {
        ShiftRegister8 {
            bits: init,
        }
    }
    
    pub fn shift(&mut self) -> u8 {
        let result = self.bits & 0x01;
        self.bits =  self.bits >> 1;
        result
    }
    
    pub fn load(&mut self, val: u8) {
        self.bits = val;
    }
}