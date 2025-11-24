use std::backtrace;

pub struct Flags(u8);

impl Flags {
    const CARRY: u8 = 0b0000_0001;
    const PARITY: u8 = 0b0000_0100;
    const AUX: u8 = 0b0001_0000;
    const ZERO: u8 = 0b0100_0000;
    const SIGN: u8 = 0b1000_0000;

    fn set_carry(&mut self, value: bool) {
        if value {
            self.0 |= Self::CARRY;
        } else {
            self.0 &= !Self::CARRY;
        }
    }

    fn clear_carry(&mut self) {
        self.0 &= !Self::CARRY;
    }

    fn get_carry(&self) -> bool {
        (self.0 & Self::CARRY) != 0
    }

    fn set_zero(&mut self, value: bool) {
        if value {
            self.0 |= Self::ZERO;
        } else {
            self.0 &= !Self::ZERO;
        }
    }

    fn clear_zero(&mut self) {
        self.0 &= !Self::ZERO;
    }

    fn get_zero(&self) -> bool {
        (self.0 & Self::ZERO) != 0
    }

    fn set_parity(&mut self, value: bool) {
        if value {
            self.0 |= Self::PARITY;
        } else {
            self.0 &= !Self::PARITY;
        }
    }

    fn clear_parity(&mut self) {
        self.0 &= !Self::PARITY;
    }

    fn get_parity(&self) -> bool {
        (self.0 & Self::PARITY) != 0
    }

    fn set_aux(&mut self, value: bool) {
        if value {
            self.0 |= Self::AUX;
        } else {
            self.0 &= !Self::AUX;
        }
    }

    fn clear_aux(&mut self) {
        self.0 &= !Self::AUX;
    }

    fn get_aux(&self) -> bool {
        (self.0 & Self::AUX) != 0
    }

    fn set_sign(&mut self, value: bool) {
        if value {
            self.0 |= Self::SIGN;
        } else {
            self.0 &= !Self::SIGN;
        }
    }

    fn clear_sign(&mut self) {
        self.0 &= !Self::SIGN;
    }

    fn get_sign(&self) -> bool {
        (self.0 & Self::SIGN) != 0
    }
}

pub struct CpuMetadata {
    pub cycles: u64,
    pub instructions_executed: u64,
}

pub struct Cpu8080 {
    // registers
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    // flags
    flags: Flags,

    pc: u16,
    sp: u16,

    interrupts_enabled: bool,

    bus: [u8; 0x10000], // 64KB memory

    halted: bool,

    metadata: CpuMetadata,
}

impl Cpu8080 {
    pub fn new() -> Self {
        Cpu8080 {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            flags: Flags(0),
            pc: 0,
            sp: 0,
            interrupts_enabled: false,
            bus: [0; 0x10000],
            halted: false,
            metadata: CpuMetadata {
                cycles: 0,
                instructions_executed: 0,
            },
        }
    }

    fn update_metadata(&mut self, cycles: u64) {
        self.metadata.cycles += cycles;
        self.metadata.instructions_executed += 1;
    }

    pub fn print_debug(&self) {
        println!("<-----------------------cpu state----------------------->");
        println!("A: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X}",
                 self.a, self.b, self.c, self.d, self.e, self.h, self.l);
        println!("Flags: {:08b}", self.flags.0);
        println!("PC: {:04X} SP: {:04X}", self.pc, self.sp);
        println!("Interrupts Enabled: {}", self.interrupts_enabled);
        println!("Cycles: {} Instructions Executed: {}\n",
                 self.metadata.cycles, self.metadata.instructions_executed);
    }

    pub fn load_program(&mut self, program: &[u8], start_address: u16) {
        let start = start_address as usize;
        let end = start + program.len();
        self.bus[start..end].copy_from_slice(program);
        self.pc = start_address;
        self.sp = 0xFFFF; // Initialize stack pointer to top of memory
    }

    pub fn increment_pc(&mut self, count: u8) {
        // increment pc by the count
        self.pc += (16 * count) as u16;
    }

    pub fn is_halted(&self) -> bool {
        return self.halted
    }

    pub fn step(&mut self) {
        if self.is_halted() {
            return;
        }
        
        let opcode = self.fetch_byte();
        match opcode {
            0x00 => {
                // NOP
                self.update_metadata(4);
            }
            
            // Load Immediate Instructions for Register Pairs
            0x01 => {
                // LXI B, D16
                let data = self.fetch_word();
                self.set_bc(data);
                self.update_metadata(10);
            }
            0x11 => {
                // LXI D, D16
                let data = self.fetch_word();
                self.set_de(data);
                self.update_metadata(10);
            }
            0x21 => {
                // LXI H, D16
                let data = self.fetch_word();
                self.h = (data >> 8) as u8;
                self.l = (data & 0xFF) as u8;
                self.update_metadata(10);
            }
            0x31 => {
                // LXI SP, D16
                let data = self.fetch_word();
                self.sp = data;
                self.update_metadata(10);
            }

            // immediate loads (MOV r_dest, r_src)
            b if (b & 0b1100_0000) == 0b0100_0000 => {
                let dest_code = (b >> 3) & 0b0000_0111;
                let src_code = b & 0b0000_0111; 
                let src_val = self.get_register_by_code(src_code);
                let dest_val = self.get_register_ref_mut_by_code(dest_code);
                *dest_val = src_val;

                self.update_metadata(5);
            }

            // mvi
            b if (b & 0xC7) == 0x06 => { 
                let dest_code = (b >> 3) & 0b0000_0111;
                let imm_value = self.fetch_byte();
                let dest_val = self.get_register_ref_mut_by_code(dest_code);
                *dest_val = imm_value;

                self.update_metadata(7);
            }

            // LXI instructions
            b if (b & 0xCF) == 0x01 => {
                let rp_code = (b >> 4) & 0b0000_0011;
                let data = self.fetch_word();
                match rp_code {
                    0 => self.set_bc(data),
                    1 => self.set_de(data),
                    2 => self.set_hl(data),
                    3 => self.sp = data,
                    _ => panic!("Invalid register pair code: {}", rp_code),
                }

                self.update_metadata(10);
            }

            // INC / DEC instructions
            b if (b & 0xC7) == 0x04 => {
                let reg_code = (b >> 3) & 0b0000_0111;
                let reg_ref = self.get_register_ref_mut_by_code(reg_code);
                *reg_ref = reg_ref.wrapping_add(1);
                self.update_flags(*reg_ref);
                self.update_metadata(5);
            }

            b if (b & 0xC7) == 0x05 => {
                let reg_code = (b >> 3) & 0b0000_0111;
                let reg_ref = self.get_register_ref_mut_by_code(reg_code);
                *reg_ref = reg_ref.wrapping_sub(1);
                self.update_flags(*reg_ref);
                self.update_metadata(5);
            }

            // 

            0x76 => {
                // HLT
                self.halted = true;
                self.update_metadata(7);
            }
            
            _ => {
                panic!("Unimplemented opcode: {:02X}", opcode);
            }
        }
    } 

    fn update_flags(&mut self, result: u8) {
        // zero Flag
        self.flags.set_zero(result == 0);

        // sign Flag
        self.flags.set_sign((result & 0x80) != 0);

        // aux 

        // parity Flag
        let mut count = 0;
        for i in 0..8 {
            if (result >> i) & 1 == 1 {
                count += 1;
            }
        }
        self.flags.set_parity(count % 2 == 0);
    }

    // return an immutable reference to the register or memory specified by `code`.
    fn get_register_by_code(&self, code: u8) -> u8 {
        match code {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => self.read_m(),
            7 => self.a,
            _ => panic!("Invalid register code: {}", code),
        }
    }

    /// return a mutable reference to the register or memory specified by `code`. 
    fn get_register_ref_mut_by_code(&mut self, code: u8) -> &mut u8 {
        match code {
            0 => &mut self.b,
            1 => &mut self.c,
            2 => &mut self.d,
            3 => &mut self.e,
            4 => &mut self.h,
            5 => &mut self.l,
            6 => self.read_m_mut(),
            7 => &mut self.a,
            _ => panic!("Invalid register code: {}", code),
        }
    }
 
    fn fetch_word(&mut self) -> u16 {
        let low = self.fetch_byte() as u16;
        let high = self.fetch_byte() as u16;
        (high << 8) | low
    }

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.bus[self.pc as usize];
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }

    fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | (self.e as u16)
    }

    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }

    fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | (self.l as u16)
    }

    fn read_m(&self) -> u8 {
        let addr = self.get_hl();
        self.bus[addr as usize]
    }

    fn read_m_mut(&mut self) -> &mut u8 {
        let addr = self.get_hl() as usize;
        &mut self.bus[addr]
    }

    fn write_m(&mut self, value: u8) {
        let addr = self.get_hl();
        self.bus[addr as usize] = value;
    }
    
}