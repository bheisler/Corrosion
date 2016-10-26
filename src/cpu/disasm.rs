use memory::MemSegment;
use cpu::CPU;

pub struct Disassembler<'a> {
    pc: u16,
    cpu: &'a mut CPU,
    bytes: Vec<u8>,
    unofficial: bool,
}

struct PartialInstruction {
    pattern: String,
}

impl PartialInstruction {
    fn finish(self, instr: &str) -> String {
        self.pattern.replace("$$$", instr).clone()
    }
}

pub struct Instruction {
    pub bytes: Vec<u8>,
    pub str: String,
    pub unofficial: bool,
    pub address: u16,
}

#[cfg(feature="cputrace")]
impl<'a> Disassembler<'a> {
    // Addressing modes
    fn immediate(&mut self) -> PartialInstruction {
        PartialInstruction { pattern: format!("$$$ #${:02X}", self.read_incr_pc()) }
    }
    fn absolute(&mut self) -> PartialInstruction {
        let arg = self.read_w_incr_pc();
        PartialInstruction { pattern: format!("$$$ ${:04X} = {:02X}", arg, self.read_safe(arg)) }
    }
    fn absolute_x(&mut self) -> PartialInstruction {
        let arg = self.read_w_incr_pc();
        let target = arg.wrapping_add(self.cpu.regs.x as u16);
        PartialInstruction {
            pattern: format!("$$$ ${:04X},X @ {:04X} = {:02X}",
                             arg,
                             target,
                             self.read_safe(target)),
        }
    }
    fn absolute_y(&mut self) -> PartialInstruction {
        let arg = self.read_w_incr_pc();
        let target = arg.wrapping_add(self.cpu.regs.y as u16);
        PartialInstruction {
            pattern: format!("$$$ ${:04X},Y @ {:04X} = {:02X}",
                             arg,
                             target,
                             self.read_safe(target)),
        }
    }
    fn zero_page(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        PartialInstruction {
            pattern: format!("$$$ ${:02X} = {:02X}", arg, self.read_safe(arg as u16)),
        }
    }
    fn zero_page_x(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        let target = arg.wrapping_add(self.cpu.regs.x);
        PartialInstruction {
            pattern: format!("$$$ ${:02X},X @ {:02X} = {:02X}",
                             arg,
                             target,
                             self.read_safe(target as u16)),
        }
    }
    fn zero_page_y(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        let target = arg.wrapping_add(self.cpu.regs.y);
        PartialInstruction {
            pattern: format!("$$$ ${:02X},Y @ {:02X} = {:02X}",
                             arg,
                             target,
                             self.read_safe(target as u16)),
        }
    }
    fn indirect_x(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        let zp_idx = arg.wrapping_add(self.cpu.regs.x);
        let ptr = self.read_safe_w_zero_page(zp_idx);
        let target = self.read_safe(ptr);
        PartialInstruction {
            pattern: format!("$$$ (${:02X},X) @ {:02X} = {:04X} = {:02X}",
                             arg,
                             zp_idx,
                             ptr,
                             target),
        }
    }
    fn indirect_y(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        let base_ptr = self.read_safe_w_zero_page(arg);
        let ptr = base_ptr.wrapping_add(self.cpu.regs.y as u16);
        let target = self.read_safe(ptr);
        PartialInstruction {
            pattern: format!("$$$ (${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                             arg,
                             base_ptr,
                             ptr,
                             target),
        }
    }
    fn accumulator(&mut self) -> PartialInstruction {
        PartialInstruction { pattern: format!("$$$ A") }
    }

    fn read_safe_w_zero_page(&mut self, zp_idx: u8) -> u16 {
        let low = self.read_safe(zp_idx as u16) as u16;
        let high = self.read_safe(zp_idx.wrapping_add(1) as u16) as u16;
        (high << 8) | low
    }
}

#[cfg(not(feature="cputrace"))]
impl<'a> Disassembler<'a> {
    // Addressing modes
    fn immediate(&mut self) -> PartialInstruction {
        PartialInstruction { pattern: format!("$$$ #${:02X}", self.read_incr_pc()) }
    }
    fn absolute(&mut self) -> PartialInstruction {
        let arg = self.read_w_incr_pc();
        PartialInstruction { pattern: format!("$$$ ${:04X}", arg) }
    }
    fn absolute_x(&mut self) -> PartialInstruction {
        let arg = self.read_w_incr_pc();
        PartialInstruction { pattern: format!("$$$ ${:04X},X", arg) }
    }
    fn absolute_y(&mut self) -> PartialInstruction {
        let arg = self.read_w_incr_pc();
        PartialInstruction { pattern: format!("$$$ ${:04X},Y", arg) }
    }
    fn zero_page(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        PartialInstruction { pattern: format!("$$$ ${:02X}", arg) }
    }
    fn zero_page_x(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        PartialInstruction { pattern: format!("$$$ ${:02X},X", arg) }
    }
    fn zero_page_y(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        PartialInstruction { pattern: format!("$$$ ${:02X},Y", arg) }
    }
    fn indirect_x(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        PartialInstruction { pattern: format!("$$$ (${:02X},X)", arg) }
    }
    fn indirect_y(&mut self) -> PartialInstruction {
        let arg = self.read_incr_pc();
        PartialInstruction { pattern: format!("$$$ (${:02X}),Y", arg) }
    }
    fn accumulator(&mut self) -> PartialInstruction {
        PartialInstruction { pattern: format!("$$$ A") }
    }
}

impl<'a> Disassembler<'a> {
    pub fn new(cpu: &'a mut CPU) -> Disassembler {
        Disassembler {
            pc: cpu.regs.pc,
            cpu: cpu,
            bytes: vec![],
            unofficial: false,
        }
    }

    // Instructions
    // Stores
    fn stx(&mut self, instr: PartialInstruction) -> String {
        instr.finish("STX")
    }
    fn sty(&mut self, instr: PartialInstruction) -> String {
        instr.finish("STY")
    }
    fn sta(&mut self, instr: PartialInstruction) -> String {
        instr.finish("STA")
    }

    // Loads
    fn ldx(&mut self, instr: PartialInstruction) -> String {
        instr.finish("LDX")
    }
    fn lda(&mut self, instr: PartialInstruction) -> String {
        instr.finish("LDA")
    }
    fn ldy(&mut self, instr: PartialInstruction) -> String {
        instr.finish("LDY")
    }

    // Logic/Math Ops
    fn bit(&mut self, instr: PartialInstruction) -> String {
        instr.finish("BIT")
    }
    fn and(&mut self, instr: PartialInstruction) -> String {
        instr.finish("AND")
    }
    fn ora(&mut self, instr: PartialInstruction) -> String {
        instr.finish("ORA")
    }
    fn eor(&mut self, instr: PartialInstruction) -> String {
        instr.finish("EOR")
    }
    fn adc(&mut self, instr: PartialInstruction) -> String {
        instr.finish("ADC")
    }
    fn sbc(&mut self, instr: PartialInstruction) -> String {
        instr.finish("SBC")
    }
    fn cmp(&mut self, instr: PartialInstruction) -> String {
        instr.finish("CMP")
    }
    fn cpx(&mut self, instr: PartialInstruction) -> String {
        instr.finish("CPX")
    }
    fn cpy(&mut self, instr: PartialInstruction) -> String {
        instr.finish("CPY")
    }
    fn inc(&mut self, instr: PartialInstruction) -> String {
        instr.finish("INC")
    }
    fn iny(&mut self) -> String {
        "INY".to_string()
    }
    fn inx(&mut self) -> String {
        "INX".to_string()
    }
    fn dec(&mut self, instr: PartialInstruction) -> String {
        instr.finish("DEC")
    }
    fn dey(&mut self) -> String {
        "DEY".to_string()
    }
    fn dex(&mut self) -> String {
        "DEX".to_string()
    }
    fn lsr(&mut self, instr: PartialInstruction) -> String {
        instr.finish("LSR")
    }
    fn asl(&mut self, instr: PartialInstruction) -> String {
        instr.finish("ASL")
    }
    fn ror(&mut self, instr: PartialInstruction) -> String {
        instr.finish("ROR")
    }
    fn rol(&mut self, instr: PartialInstruction) -> String {
        instr.finish("ROL")
    }

    // Jumps
    fn jmp(&mut self) -> String {
        format!("JMP ${:04X}", self.read_w_incr_pc())
    }
    fn jmpi(&mut self) -> String {
        let arg = self.read_w_incr_pc();
        format!("JMP (${:04X}) = {:04X}", arg, self.read_safe_w(arg))
    }
    fn jsr(&mut self) -> String {
        let arg = self.read_w_incr_pc();
        format!("JSR ${:04X}", arg)
    }
    fn rts(&mut self) -> String {
        "RTS".to_string()
    }
    fn rti(&mut self) -> String {
        "RTI".to_string()
    }
    fn brk(&mut self) -> String {
        "BRK".to_string()
    }

    // Branches
    fn bcs(&mut self) -> String {
        self.branch("BCS")
    }
    fn bcc(&mut self) -> String {
        self.branch("BCC")
    }
    fn beq(&mut self) -> String {
        self.branch("BEQ")
    }
    fn bne(&mut self) -> String {
        self.branch("BNE")
    }
    fn bvs(&mut self) -> String {
        self.branch("BVS")
    }
    fn bvc(&mut self) -> String {
        self.branch("BVC")
    }
    fn bmi(&mut self) -> String {
        self.branch("BMI")
    }
    fn bpl(&mut self) -> String {
        self.branch("BPL")
    }

    fn branch(&mut self, instr: &str) -> String {
        let arg = self.read_incr_pc();
        format!("{:3} ${:04X}", instr, self.relative_addr(arg))
    }

    // Stack
    fn plp(&mut self) -> String {
        "PLP".to_string()
    }
    fn php(&mut self) -> String {
        "PHP".to_string()
    }
    fn pla(&mut self) -> String {
        "PLA".to_string()
    }
    fn pha(&mut self) -> String {
        "PHA".to_string()
    }

    // Misc
    fn nop(&mut self) -> String {
        "NOP".to_string()
    }
    fn sec(&mut self) -> String {
        "SEC".to_string()
    }
    fn clc(&mut self) -> String {
        "CLC".to_string()
    }
    fn sei(&mut self) -> String {
        "SEI".to_string()
    }
    fn sed(&mut self) -> String {
        "SED".to_string()
    }
    fn cld(&mut self) -> String {
        "CLD".to_string()
    }
    fn clv(&mut self) -> String {
        "CLV".to_string()
    }
    fn tax(&mut self) -> String {
        "TAX".to_string()
    }
    fn tay(&mut self) -> String {
        "TAY".to_string()
    }
    fn tsx(&mut self) -> String {
        "TSX".to_string()
    }
    fn txa(&mut self) -> String {
        "TXA".to_string()
    }
    fn txs(&mut self) -> String {
        "TXS".to_string()
    }
    fn tya(&mut self) -> String {
        "TYA".to_string()
    }
    fn cli(&mut self) -> String {
        "CLI".to_string()
    }

    // Unofficial instructions
    fn u_nop(&mut self, instr: PartialInstruction) -> String {
        instr.finish("NOP")
    }
    fn lax(&mut self, instr: PartialInstruction) -> String {
        instr.finish("LAX")
    }
    fn sax(&mut self, instr: PartialInstruction) -> String {
        instr.finish("SAX")
    }
    fn dcp(&mut self, instr: PartialInstruction) -> String {
        instr.finish("DCP")
    }
    fn isc(&mut self, instr: PartialInstruction) -> String {
        // Nintendulator calls this op ISB, so I'll use the same in the logs
        // at least for now
        instr.finish("ISB")
    }
    fn slo(&mut self, instr: PartialInstruction) -> String {
        instr.finish("SLO")
    }
    fn rla(&mut self, instr: PartialInstruction) -> String {
        instr.finish("RLA")
    }
    fn sre(&mut self, instr: PartialInstruction) -> String {
        instr.finish("SRE")
    }
    fn rra(&mut self, instr: PartialInstruction) -> String {
        instr.finish("RRA")
    }
    fn kil(&mut self) -> String {
        "KIL".to_string()
    }

    fn decode_instruction(&mut self) -> Instruction {
        let address = self.pc;
        let opcode = self.read_incr_pc();
        let str: String = decode_opcode!(opcode, self);
        let instr = Instruction {
            bytes: self.bytes.clone(),
            str: str,
            unofficial: self.unofficial,
            address: address,
        };
        self.bytes.clear();
        self.unofficial = false;
        instr
    }

    pub fn decode(mut self) -> Instruction {
        self.decode_instruction()
    }
    pub fn decode_function(mut self, entry_point: u16, exit_point: u16) -> Vec<Instruction> {
        let mut instructions: Vec<Instruction> = vec![];
        self.pc = entry_point;
        while self.pc <= exit_point {
            instructions.push(self.decode_instruction());
        }
        instructions
    }

    fn unofficial(&mut self) {
        self.unofficial = true;
    }

    fn relative_addr(&self, disp: u8) -> u16 {
        let disp = (disp as i8) as i16; //We want to sign-extend here.
        let pc = self.pc as i16;
        pc.wrapping_add(disp) as u16
    }

    fn read_incr_pc(&mut self) -> u8 {
        let pc = self.pc;
        let val: u8 = self.read_safe(pc);
        self.bytes.push(val);
        self.pc = self.pc.wrapping_add(1);
        val
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
    fn read_safe_w(&mut self, idx: u16) -> u16 {
        let low = self.read_safe(idx) as u16;
        let high = self.read_safe(idx + 1) as u16;
        (high << 8) | low
    }

    pub fn trace(mut self) {
        let opcode = self.decode_instruction();

        let cyc = (self.cpu.cycle * 3) % 341;
        let mut sl = ((((self.cpu.cycle as isize) * 3) / 341) + 241) % 262;
        if sl == 261 {
            sl = -1;
        }

        println!("{:04X}  {:9}{}{:30}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:3} \
                  SL:{}",
                 self.cpu.regs.pc,
                 opcode.bytes
                     .iter()
                     .map(|byte| format!("{:02X}", byte))
                     .fold(None as Option<String>, |opt, right| {
                         match opt {
                             Some(left) => Some(left + " " + &right),
                             None => Some(right),
                         }
                     })
                     .unwrap(),
                 if opcode.unofficial {
                     "*"
                 } else {
                     " "
                 },
                 opcode.str,
                 self.cpu.regs.a,
                 self.cpu.regs.x,
                 self.cpu.regs.y,
                 self.cpu.regs.p.bits(),
                 self.cpu.regs.sp,
                 cyc,
                 sl);
    }
}
