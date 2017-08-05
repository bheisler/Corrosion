use cpu::CPU;
use fnv::FnvHashMap;
use fnv::FnvHashSet;
use memory::MemSegment;

pub struct Analyst<'a> {
    entry_point: u16,
    pc: u16,
    current_instruction: u16,
    cpu: &'a mut CPU,
    furthest_branch: u16,
    found_exit_point: bool,

    pub last_sign_flag_set: u16,
    pub last_overflow_flag_set: u16,
    pub last_zero_flag_set: u16,
    pub last_carry_flag_set: u16,

    instructions: FnvHashMap<u16, InstructionAnalysis>,
}

#[derive(Debug, Clone)]
pub struct InstructionAnalysis {
    pub is_branch_target: bool,

    pub sign_flag_used: bool,
    pub overflow_flag_used: bool,
    pub zero_flag_used: bool,
    pub carry_flag_used: bool,
}

impl Default for InstructionAnalysis {
    fn default() -> InstructionAnalysis {
        InstructionAnalysis {
            is_branch_target: false,

            sign_flag_used: false,
            overflow_flag_used: false,
            zero_flag_used: false,
            carry_flag_used: false,
        }
    }
}

#[derive(Debug)]
pub struct BlockAnalysis {
    pub entry_point: u16,
    pub exit_point: u16,

    pub instructions: FnvHashMap<u16, InstructionAnalysis>,
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

            last_sign_flag_set: 0,
            last_carry_flag_set: 0,
            last_zero_flag_set: 0,
            last_overflow_flag_set: 0,

            instructions: FnvHashMap::default(),
        }
    }

    #[cfg(feature = "debug_features")]
    pub fn find_exit_point(self, entry_point: u16) -> u16 {
        self.analyze(entry_point).exit_point
    }

    pub fn analyze(mut self, entry_point: u16) -> BlockAnalysis {
        self.entry_point = entry_point;
        self.pc = entry_point;

        let mut valid_instr_addrs: FnvHashSet<u16> = FnvHashSet::default();

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

    fn remove_invalid_instructions(&mut self, valid_instr_addrs: FnvHashSet<u16>) {
        let mut invalid_instrs: FnvHashSet<u16> = FnvHashSet::default();

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
    fn ldx(&mut self, _: u8) {
        self.sign_set();
        self.zero_set();
    }
    fn lda(&mut self, _: u8) {
        self.sign_set();
        self.zero_set();
    }
    fn ldy(&mut self, _: u8) {
        self.sign_set();
        self.zero_set();
    }

    // Logic/Math Ops
    fn bit(&mut self, _: u8) {
        self.sign_set();
        self.carry_set();
        self.zero_set();
    }
    fn and(&mut self, _: u8) {
        self.sign_set();
        self.zero_set();
    }
    fn ora(&mut self, _: u8) {
        self.sign_set();
        self.zero_set();
    }
    fn eor(&mut self, _: u8) {
        self.sign_set();
        self.zero_set();
    }
    fn adc(&mut self, _: u8) {
        self.carry_used();
        self.carry_set();
        self.sign_set();
        self.zero_set();
        self.overflow_set();
    }
    fn sbc(&mut self, _: u8) {
        self.carry_used();
        self.carry_set();
        self.sign_set();
        self.zero_set();
        self.overflow_set();
    }
    fn cmp(&mut self, _: u8) {
        self.zero_set();
        self.sign_set();
        self.carry_set();
    }
    fn cpx(&mut self, _: u8) {
        self.zero_set();
        self.sign_set();
        self.carry_set();
    }
    fn cpy(&mut self, _: u8) {
        self.zero_set();
        self.sign_set();
        self.carry_set();
    }
    fn inc(&mut self, _: u8) {
        self.zero_set();
        self.sign_set();
    }
    fn iny(&mut self) {
        self.zero_set();
        self.sign_set();
    }
    fn inx(&mut self) {
        self.zero_set();
        self.sign_set();
    }
    fn dec(&mut self, _: u8) {
        self.zero_set();
        self.sign_set();
    }
    fn dey(&mut self) {
        self.zero_set();
        self.sign_set();
    }
    fn dex(&mut self) {
        self.zero_set();
        self.sign_set();
    }
    fn lsr(&mut self, _: u8) {
        self.zero_set();
        self.sign_set();
        self.carry_set();
    }
    fn asl(&mut self, _: u8) {
        self.zero_set();
        self.sign_set();
        self.carry_set();
    }
    fn ror(&mut self, _: u8) {
        self.carry_used();
        self.carry_set();
        self.sign_set();
        self.zero_set();
    }
    fn rol(&mut self, _: u8) {
        self.carry_used();
        self.carry_set();
        self.sign_set();
        self.zero_set();
    }

    // Jumps
    fn jmp(&mut self) {
        self.all_used();
        self.read_w_incr_pc();
        self.end_function();
    }
    fn jmpi(&mut self) {
        self.all_used();
        self.read_w_incr_pc();
        self.end_function();
    }
    fn jsr(&mut self) {
        self.all_used();
        self.read_w_incr_pc();
        self.end_function();
    }
    fn rts(&mut self) {
        self.all_used();
        self.end_function();
    }
    fn rti(&mut self) {
        self.all_used();
        self.end_function();
    }
    fn brk(&mut self) {
        self.all_used();
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
        self.all_used();
        self.branch()
    }
    fn bcc(&mut self) {
        self.all_used();
        self.branch()
    }
    fn beq(&mut self) {
        self.all_used();
        self.branch()
    }
    fn bne(&mut self) {
        self.all_used();
        self.branch()
    }
    fn bvs(&mut self) {
        self.all_used();
        self.branch()
    }
    fn bvc(&mut self) {
        self.all_used();
        self.branch()
    }
    fn bmi(&mut self) {
        self.all_used();
        self.branch()
    }
    fn bpl(&mut self) {
        self.all_used();
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
    fn plp(&mut self) {
        self.carry_set();
        self.sign_set();
        self.zero_set();
        self.overflow_set();
    }
    fn php(&mut self) {
        self.all_used()
    }
    fn pla(&mut self) {
        self.sign_set();
        self.zero_set();
    }
    fn pha(&mut self) {}

    // Misc
    fn nop(&mut self) {}
    fn sec(&mut self) {
        self.carry_set();
    }
    fn clc(&mut self) {
        self.carry_set();
    }
    fn sei(&mut self) {}
    fn sed(&mut self) {}
    fn cld(&mut self) {}
    fn clv(&mut self) {
        self.overflow_set();
    }
    fn tax(&mut self) {
        self.sign_set();
        self.zero_set();
    }
    fn tay(&mut self) {
        self.sign_set();
        self.zero_set();
    }
    fn tsx(&mut self) {
        self.sign_set();
        self.zero_set();
    }
    fn txa(&mut self) {
        self.sign_set();
        self.zero_set();
    }
    fn txs(&mut self) {}
    fn tya(&mut self) {
        self.sign_set();
        self.zero_set();
    }
    fn cli(&mut self) {}

    // Unofficial instructions
    fn u_nop(&mut self, _: u8) {}
    fn lax(&mut self, _: u8) {
        self.lda(0);
        self.ldx(0);
    }
    fn sax(&mut self, _: u8) {}
    fn dcp(&mut self, _: u8) {
        self.dec(0);
        self.cmp(0);
    }
    fn isc(&mut self, _: u8) {
        self.sbc(0);
        self.inc(0);
    }
    fn slo(&mut self, _: u8) {
        self.asl(0);
        self.ora(0);
    }
    fn rla(&mut self, _: u8) {
        self.rol(0);
        self.and(0);
    }
    fn sre(&mut self, _: u8) {
        self.lsr(0);
        self.eor(0);
    }
    fn rra(&mut self, _: u8) {
        self.adc(0);
        self.ror(0);
    }
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

    fn sign_set(&mut self) {
        self.last_sign_flag_set = self.current_instruction;
    }

    fn overflow_set(&mut self) {
        self.last_overflow_flag_set = self.current_instruction;
    }

    fn zero_set(&mut self) {
        self.last_zero_flag_set = self.current_instruction;
    }

    fn carry_set(&mut self) {
        self.last_carry_flag_set = self.current_instruction;
    }

    fn all_used(&mut self) {
        self.sign_used();
        self.overflow_used();
        self.zero_used();
        self.carry_used();
    }

    fn sign_used(&mut self) {
        if self.last_sign_flag_set == 0 {
            return;
        }
        self.instructions
            .get_mut(&self.last_sign_flag_set)
            .unwrap()
            .sign_flag_used = true;
    }

    fn overflow_used(&mut self) {
        if self.last_overflow_flag_set == 0 {
            return;
        }
        self.instructions
            .get_mut(&self.last_overflow_flag_set)
            .unwrap()
            .overflow_flag_used = true;
    }

    fn zero_used(&mut self) {
        if self.last_zero_flag_set == 0 {
            return;
        }
        self.instructions
            .get_mut(&self.last_zero_flag_set)
            .unwrap()
            .zero_flag_used = true;
    }

    fn carry_used(&mut self) {
        if self.last_carry_flag_set == 0 {
            return;
        }
        self.instructions
            .get_mut(&self.last_carry_flag_set)
            .unwrap()
            .carry_flag_used = true;
    }
}
