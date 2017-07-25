use cpu::CPU;
use memory::MemSegment;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct Analyst<'a> {
    entry_point: u16,
    pc: u16,
    current_instruction: u16,
    cpu: &'a mut CPU,
    furthest_branch: u16,
    found_exit_point: bool,

    instructions: HashMap<u16, InstructionAnalysis>,
}

pub struct InstructionAnalysis {
    pub is_branch_target: bool,
}

impl Default for InstructionAnalysis {
    fn default() -> InstructionAnalysis {
        InstructionAnalysis { is_branch_target: false }
    }
}

pub struct BlockAnalysis {
    pub entry_point: u16,
    pub exit_point: u16,

    pub instructions: HashMap<u16, InstructionAnalysis>,
}

impl<'a> Analyst<'a> {
    pub fn new(cpu: &'a mut CPU) -> Analyst<'a> {
        Analyst {
            entry_point: 0,
            pc: 0,
            current_instruction: 0,
            cpu: cpu,
            furthest_branch: 0,
            found_exit_point: false,

            instructions: HashMap::new(),
        }
    }

    #[cfg(feature = "function_disasm")]
    pub fn find_exit_point(self, entry_point: u16) -> u16 {
        self.analyze(entry_point).exit_point
    }

    pub fn analyze(mut self, entry_point: u16) -> BlockAnalysis {
        self.entry_point = entry_point;
        self.pc = entry_point;

        let mut valid_instr_addrs: HashSet<u16> = HashSet::new();

        while !self.found_exit_point {
            // Ensure that every instruction has an entry
            let temp_pc = self.pc;
            self.current_instruction = temp_pc;
            valid_instr_addrs.insert(temp_pc);
            self.get_instr_analysis(temp_pc);

            let opcode = self.read_incr_pc();
            decode_opcode!(opcode, self);
        }

        self.remove_invalid_instructions(valid_instr_addrs);

        BlockAnalysis {
            entry_point: entry_point,
            exit_point: self.pc - 1,
            instructions: self.instructions,
        }
    }

    fn remove_invalid_instructions(&mut self, valid_instr_addrs: HashSet<u16>) {
        let mut invalid_instrs: HashSet<u16> = HashSet::new();

        {
            for addr in self.instructions.keys().cloned() {
                if !valid_instr_addrs.contains(&addr) {
                    invalid_instrs.insert(addr);
                }
            }
        }

        for addr in &invalid_instrs {
            self.instructions.remove(addr);
        }
    }

    // Addressing modes
    fn immediate(&mut self) -> u8 {
        self.read_incr_pc();
        0
    }
    fn absolute(&mut self) -> u8 {
        self.read_w_incr_pc();
        0
    }
    fn absolute_x(&mut self) -> u8 {
        self.read_w_incr_pc();
        0
    }
    fn absolute_y(&mut self) -> u8 {
        self.read_w_incr_pc();
        0
    }
    fn zero_page(&mut self) -> u8 {
        self.read_incr_pc();
        0
    }
    fn zero_page_x(&mut self) -> u8 {
        self.read_incr_pc();
        0
    }
    fn zero_page_y(&mut self) -> u8 {
        self.read_incr_pc();
        0
    }
    fn indirect_x(&mut self) -> u8 {
        self.read_incr_pc();
        0
    }
    fn indirect_y(&mut self) -> u8 {
        self.read_incr_pc();
        0
    }
    fn accumulator(&mut self) -> u8 {
        0
    }

    // Instructions
    // Stores
    fn stx(&mut self, _: u8) {}
    fn sty(&mut self, _: u8) {}
    fn sta(&mut self, _: u8) {}

    // Loads
    fn ldx(&mut self, _: u8) {}
    fn lda(&mut self, _: u8) {}
    fn ldy(&mut self, _: u8) {}

    // Logic/Math Ops
    fn bit(&mut self, _: u8) {}
    fn and(&mut self, _: u8) {}
    fn ora(&mut self, _: u8) {}
    fn eor(&mut self, _: u8) {}
    fn adc(&mut self, _: u8) {}
    fn sbc(&mut self, _: u8) {}
    fn cmp(&mut self, _: u8) {}
    fn cpx(&mut self, _: u8) {}
    fn cpy(&mut self, _: u8) {}
    fn inc(&mut self, _: u8) {}
    fn iny(&mut self) {}
    fn inx(&mut self) {}
    fn dec(&mut self, _: u8) {}
    fn dey(&mut self) {}
    fn dex(&mut self) {}
    fn lsr(&mut self, _: u8) {}
    fn asl(&mut self, _: u8) {}
    fn ror(&mut self, _: u8) {}
    fn rol(&mut self, _: u8) {}

    // Jumps
    fn jmp(&mut self) {
        self.read_w_incr_pc();
        self.end_function();
    }
    fn jmpi(&mut self) {
        self.read_w_incr_pc();
        self.end_function();
    }
    fn jsr(&mut self) {
        self.read_w_incr_pc();
        self.end_function();
    }
    fn rts(&mut self) {
        self.end_function();
    }
    fn rti(&mut self) {
        self.end_function();
    }
    fn brk(&mut self) {
        self.end_function();
    }

    fn unofficial(&self) {}

    fn end_function(&mut self) {
        if self.pc > self.furthest_branch {
            self.found_exit_point = true;
        }
    }

    // Branches
    fn bcs(&mut self) {
        self.branch()
    }
    fn bcc(&mut self) {
        self.branch()
    }
    fn beq(&mut self) {
        self.branch()
    }
    fn bne(&mut self) {
        self.branch()
    }
    fn bvs(&mut self) {
        self.branch()
    }
    fn bvc(&mut self) {
        self.branch()
    }
    fn bmi(&mut self) {
        self.branch()
    }
    fn bpl(&mut self) {
        self.branch()
    }

    fn branch(&mut self) {
        let arg = self.read_incr_pc();
        let target = self.relative_addr(arg);
        self.get_instr_analysis(target).is_branch_target = true;
        if target > self.furthest_branch {
            self.furthest_branch = target;
        }
    }

    // Stack
    fn plp(&mut self) {}
    fn php(&mut self) {}
    fn pla(&mut self) {}
    fn pha(&mut self) {}

    // Misc
    fn nop(&mut self) {}
    fn sec(&mut self) {}
    fn clc(&mut self) {}
    fn sei(&mut self) {}
    fn sed(&mut self) {}
    fn cld(&mut self) {}
    fn clv(&mut self) {}
    fn tax(&mut self) {}
    fn tay(&mut self) {}
    fn tsx(&mut self) {}
    fn txa(&mut self) {}
    fn txs(&mut self) {}
    fn tya(&mut self) {}
    fn cli(&mut self) {}

    // Unofficial instructions
    fn u_nop(&mut self, _: u8) {}
    fn lax(&mut self, _: u8) {}
    fn sax(&mut self, _: u8) {}
    fn dcp(&mut self, _: u8) {}
    fn isc(&mut self, _: u8) {}
    fn slo(&mut self, _: u8) {}
    fn rla(&mut self, _: u8) {}
    fn sre(&mut self, _: u8) {}
    fn rra(&mut self, _: u8) {}
    fn kil(&mut self) {
        self.end_function();
    }
    fn unsupported(&mut self, _: u8) {
        self.end_function();
    }

    fn relative_addr(&self, disp: u8) -> u16 {
        let disp = (disp as i8) as i16; // We want to sign-extend here.
        let pc = self.pc as i16;
        pc.wrapping_add(disp) as u16
    }

    fn read_incr_pc(&mut self) -> u8 {
        let pc = self.pc;
        let val: u8 = self.read_safe(pc);
        self.pc = self.pc.wrapping_add(1);
        val
    }

    fn get_instr_analysis(&mut self, addr: u16) -> &mut InstructionAnalysis {
        let default: InstructionAnalysis = Default::default();
        self.instructions.entry(addr).or_insert(default)
    }

    fn read_w_incr_pc(&mut self) -> u16 {
        self.read_incr_pc() as u16 | ((self.read_incr_pc() as u16) << 8)
    }

    fn read_safe(&mut self, idx: u16) -> u8 {
        match idx {
            0x2000...0x3FFF => 0xFF,
            0x4000...0x401F => 0xFF,
            _ => self.cpu.read(idx),
        }
    }
}
