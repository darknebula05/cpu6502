use cpu6502_macros::instructions;

pub struct Cpu6502 {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8,
    pub c: bool,
    pub z: bool,
    pub i: bool,
    pub d: bool,
    pub b: bool,
    pub v: bool,
    pub n: bool,
    pub mem: [u8; 65535],
    operand: u16,
}

impl std::fmt::Debug for Cpu6502 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Cpu6502 {{ a: 0x{:02X}, x: 0x{:02X}, y: 0x{:02X}, pc: 0x{:04X}, sp: 0x{:02X}, status: 0b{:08b} }}",
            self.a, self.x, self.y, self.pc, self.sp, self.status()
        )
    }
}

impl Default for Cpu6502 {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0xfffc,
            sp: 0,
            mem: [0; 65535],
            operand: 0,
            c: false,
            z: false,
            i: false,
            d: false,
            b: false,
            v: false,
            n: false,
        }
    }
}

type InstructionTable = [(&'static str, fn(&mut Cpu6502) -> u8, &'static str); 256];

impl Cpu6502 {
    const NMI: u16 = 0xfffa;
    const RESET: u16 = 0xfffc;
    const IRQ: u16 = 0xfffe;

    fn read_byte(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }
    fn read_word(&self, addr: u16) -> u16 {
        let lo = self.read_byte(addr);
        let hi = self.read_byte(addr + 1);
        (hi as u16) << 8 | lo as u16
    }
    fn fetch_byte(&mut self) -> u8 {
        let val = self.read_byte(self.pc);
        self.pc += 1;
        val
    }
    fn fetch_word(&mut self) -> u16 {
        let val = self.read_word(self.pc);
        self.pc += 2;
        val
    }
    fn status(&self) -> u8 {
        (self.n as u8) << 7
            | (self.v as u8) << 6
            | (self.b as u8) << 4
            | (self.d as u8) << 3
            | (self.i as u8) << 2
            | (self.z as u8) << 1
            | (self.c as u8)
    }
    fn update_n_and_z(&mut self) {
        self.n = self.a & 0x80 > 0;
        self.z = self.a == 0;
    }

    fn abs(&mut self) -> u8 {
        self.operand = self.fetch_word();
        0
    }
    fn abs_x(&mut self) -> u8 {
        self.operand = self.fetch_word() + self.x as u16;
        0
    }
    fn abs_y(&mut self) -> u8 {
        self.operand = self.fetch_word() + self.y as u16;
        0
    }
    fn abs_indirect(&mut self) -> u8 {
        let addr = self.fetch_word();
        self.operand = self.read_word(addr);
        0
    }
    fn abs_x_indirect(&mut self) -> u8 {
        let addr = self.fetch_word().wrapping_add(self.x as u16);
        self.operand = self.read_word(addr);
        0
    }
    fn acc(&mut self) -> u8 {
        self.operand = self.a as u16;
        0
    }
    fn imm(&mut self) -> u8 {
        self.operand = self.fetch_byte() as u16;
        0
    }
    fn imp(&mut self) -> u8 {
        0
    }
    fn rel(&mut self) -> u8 {
        self.operand = self.pc + self.fetch_byte() as u16;
        0
    }
    fn stack(&mut self) -> u8 {
        self.operand = 1 << 8 & self.sp as u16;
        0
    }
    fn zp(&mut self) -> u8 {
        self.operand = self.fetch_byte() as u16;
        0
    }
    fn zp_x_indirect(&mut self) -> u8 {
        let addr = self.fetch_byte().wrapping_add(self.x) as u16;
        self.operand = self.read_word(addr);
        0
    }
    fn zp_x(&mut self) -> u8 {
        self.operand = self.fetch_byte().wrapping_add(self.x) as u16;
        0
    }
    fn zp_y(&mut self) -> u8 {
        self.operand = self.fetch_byte().wrapping_add(self.y) as u16;
        0
    }
    fn zp_indirect(&mut self) -> u8 {
        let addr = self.fetch_byte() as u16;
        self.operand = self.read_word(addr);
        0
    }
    fn zp_y_indirect(&mut self) -> u8 {
        let addr = self.fetch_byte().wrapping_add(self.y) as u16;
        self.operand = self.read_word(addr);
        0
    }

    fn adc(&mut self) {
        let fetched = self.read_byte(self.operand);
        let temp = self.a as u16 + fetched as u16 + self.c as u16;
        self.c = temp > 255;
        self.v = ((!(self.a as u16 ^ fetched as u16) & (self.a as u16 ^ temp)) & 0x0080) != 0;
        self.a = temp as u8;
        self.update_n_and_z();
    }
    fn and(&mut self) {
        self.a &= self.operand as u8;
        self.update_n_and_z();
    }
    fn asl(&mut self) {
        self.c = self.a >= 0x80;
        self.a = (self.operand as u8) << 1;
        self.update_n_and_z();
    }
    fn bcc(&mut self) {
        if !self.c {
            self.pc = self.operand;
        }
    }
    fn bcs(&mut self) {
        if self.c {
            self.pc = self.operand;
        }
    }
    fn beq(&mut self) {
        if self.z {
            self.pc = self.operand;
        }
    }
    fn bit(&mut self) {
        let result = self.a & (self.operand as u8);
        self.n = self.operand & 0x80 > 0;
        self.v = self.operand & 0x40 > 0;
        self.z = result == 0;
    }
    fn bmi(&mut self) {
        if self.n {
            self.pc = self.operand;
        }
    }
    fn bne(&mut self) {
        if !self.z {
            self.pc = self.operand;
        }
    }
    fn bpl(&mut self) {
        if !self.n {
            self.pc = self.operand;
        }
    }
    fn brk(&mut self) {
        if !self.c {
            self.pc = self.operand;
        }
    }
    fn bvc(&mut self) {
        if !self.v {
            self.pc = self.operand;
        }
    }
    fn bvs(&mut self) {
        if self.v {
            self.pc = self.operand;
        }
    }
    fn clc(&mut self) {
        self.c = false;
    }
    fn cld(&mut self) {
        self.d = false;
    }
    fn cli(&mut self) {
        self.i = false;
    }
    fn clv(&mut self) {
        self.v = false;
    }
    fn cmp(&mut self) {}
    fn cpx(&mut self) {}
    fn cpy(&mut self) {}
    fn dec(&mut self) {}
    fn dex(&mut self) {}
    fn dey(&mut self) {}
    fn eor(&mut self) {}
    fn inc(&mut self) {}
    fn inx(&mut self) {}
    fn iny(&mut self) {}
    fn jmp(&mut self) {}
    fn jsr(&mut self) {}
    fn lda(&mut self) {}
    fn ldx(&mut self) {}
    fn ldy(&mut self) {}
    fn lsr(&mut self) {}
    fn nop(&mut self) {}
    fn ora(&mut self) {}
    fn pha(&mut self) {}
    fn php(&mut self) {}
    fn pla(&mut self) {}
    fn plp(&mut self) {}
    fn rol(&mut self) {}
    fn ror(&mut self) {}
    fn rti(&mut self) {}
    fn rts(&mut self) {}
    fn sbc(&mut self) {}
    fn sec(&mut self) {}
    fn sed(&mut self) {}
    fn sei(&mut self) {}
    fn sta(&mut self) {}
    fn stx(&mut self) {}
    fn sty(&mut self) {}
    fn tax(&mut self) {}
    fn tay(&mut self) {}
    fn tsx(&mut self) {}
    fn txa(&mut self) {}
    fn txs(&mut self) {}
    fn tya(&mut self) {}

    pub const LOOKUP: InstructionTable = instructions!(
        adc: (imm, 0x69, 2), (abs, 0x6d, 4), (abs_x, 0x7d, 4+p), (abs_y, 0x79, 4+p), (zp, 0x65, 3), (zp_x, 0x75, 4), (zp_x_indirect, 0x61, 6), (zp_y_indirect, 0x71, 5+p);
        and: (imm, 0x29, 2), (abs, 0x2d, 4), (abs_x, 0x3d, 4+p), (abs_y, 0x39, 4+p), (zp, 0x25, 3), (zp_x, 0x35, 4), (zp_x_indirect, 0x21, 6), (zp_y_indirect, 0x31, 5+p);
        asl: (acc, 0x0a, 2), (abs, 0x0e, 6), (abs_x, 0x1e, 7), (zp, 0x06, 5), (zp_x, 0x16, 6);
        bcc: (rel, 0x90, 2+t+p);
        bcs: (rel, 0xb0, 2+t+p);
        beq: (rel, 0xf0, 2+t+p);
        bit: (abs, 0x2c, 4), (zp, 0x24, 3);
        bmi: (rel, 0x30, 2+t+p);
        bne: (rel, 0xd0, 2+t+p);
        bpl: (rel, 0x10, 2+t+p);
        brk: (imp, 0x00, 7);
        bvc: (rel, 0x50, 2+t+p);
        bvs: (rel, 0x70, 2+t+p);
        clc: (imp, 0x18, 2);
        cld: (imp, 0xd8, 2);
        cli: (imp, 0x58, 2);
        clv: (imp, 0xb8, 2);
        cmp: (imm, 0xc9, 2), (abs, 0xcd, 4), (abs_x, 0xdd, 4+p), (abs_y, 0xd9, 4+p), (zp, 0xc5, 3), (zp_x, 0xd5, 4), (zp_x_indirect, 0xc1, 6), (zp_y_indirect, 0xd1, 5+p);
        cpx: (imm, 0xe0, 2), (abs, 0xec, 4), (zp, 0xe4, 3);
        cpy: (imm, 0xc0, 2), (abs, 0xcc, 4), (zp, 0xc4, 3);
        dec: (abs, 0xce, 6), (abs_x, 0xde, 7), (zp, 0xc6, 5), (zp_x, 0xd6, 6);
        dex: (imp, 0xca, 2);
        dey: (imp, 0x88, 2);
        eor: (imm, 0x49, 2), (abs, 0x4d, 4), (abs_x, 0x5d, 4+p), (abs_y, 0x59, 4+p), (zp, 0x45, 3), (zp_x, 0x55, 4), (zp_x_indirect, 0x41, 6), (zp_y_indirect, 0x51, 5+p);
        inc: (abs, 0xee, 6), (abs_x, 0xfe, 7), (zp, 0xe6, 5), (zp_x, 0xf6, 6);
        inx: (imp, 0xe8, 2);
        iny: (imp, 0xc8, 2);
        jmp: (abs, 0x4c, 3), (abs_indirect, 0x6c, 5);
        jsr: (abs, 0x20, 6);
        lda: (imm, 0xa9, 2), (abs, 0xad, 4), (abs_x, 0xbd, 4+p), (abs_y, 0xb9, 4+p), (zp, 0xa5, 3), (zp_x, 0xb5, 4), (zp_x_indirect, 0xa1, 6), (zp_y_indirect, 0xb1, 5+p);
        ldx: (imm, 0xa2, 2), (abs, 0xae, 4), (abs_y, 0xbe, 4+p), (zp, 0xa6, 3), (zp_y, 0xb6, 4);
        ldy: (imm, 0xa0, 2), (abs, 0xac, 4), (abs_x, 0xbc, 4+p), (zp, 0xa4, 3), (zp_x, 0xb4, 4);
        lsr: (acc, 0x4a, 2), (abs, 0x4e, 6), (abs_x, 0x5e, 7), (zp, 0x46, 5), (zp_x, 0x56, 6);
        nop: (imp, 0xea, 2);
        ora: (imm, 0x09, 2), (abs, 0x0d, 4), (abs_x, 0x1d, 4+p), (abs_y, 0x19, 4+p), (zp, 0x05, 3), (zp_x, 0x15, 4), (zp_x_indirect, 0x01, 6), (zp_y_indirect, 0x11, 5+p);
        pha: (imp, 0x48, 3);
        php: (imp, 0x08, 3);
        pla: (imp, 0x68, 4);
        plp: (imp, 0x28, 4);
        rol: (acc, 0x2a, 2), (abs, 0x2e, 6), (abs_x, 0x3e, 7), (zp, 0x26, 5), (zp_x, 0x36, 6);
        ror: (acc, 0x6a, 2), (abs, 0x6e, 6), (abs_x, 0x7e, 7), (zp, 0x66, 5), (zp_x, 0x76, 6);
        rti: (imp, 0x40, 6);
        rts: (imp, 0x60, 6);
        sbc: (imm, 0xe9, 2), (abs, 0xed, 4), (abs_x, 0xfd, 4+p), (abs_y, 0xf9, 4+p), (zp, 0xe5, 3), (zp_x, 0xf5, 4), (zp_x_indirect, 0xe1, 6), (zp_y_indirect, 0xf1, 5+p);
        sec: (imp, 0x38, 2);
        sed: (imp, 0xf8, 2);
        sei: (imp, 0x78, 2);
        sta: (abs, 0x8d, 4), (abs_x, 0x9d, 5), (abs_y, 0x99, 5), (zp, 0x85, 3), (zp_x, 0x95, 4), (zp_x_indirect, 0x81, 6), (zp_y_indirect, 0x91, 6);
        stx: (abs, 0x8e, 4), (zp, 0x86, 3), (zp_y, 0x96, 4);
        sty: (abs, 0x8c, 4), (zp, 0x84, 3), (zp_x, 0x94, 4);
        tax: (imp, 0xaa, 2);
        tay: (imp, 0xa8, 2);
        tsx: (imp, 0xba, 2);
        txa: (imp, 0x8a, 2);
        txs: (imp, 0x9a, 2);
        tya: (imp, 0x98, 2);
    );
}
