use std::{cell::RefCell, rc::Rc};

use crate::memory::Memory;

pub type SharedMemory = Rc<RefCell<Memory>>;

pub struct Controller {
    pub register: u64,
    /// Instruction pointer
    pub ip: usize,
    /// Instructions as set of bytes
    pub program: Vec<u8>,
    /// Instruction's byte position
    pub instruction_pos: Vec<usize>,
    /// Weather the controller is stopped
    pub halted: bool,
    /// Set of memory addresses that can be accessed
    pub memories: Vec<SharedMemory>,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            register: 0,
            ip: 0,
            program: Vec::new(),
            instruction_pos: Vec::new(),
            halted: false,
            memories: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.register = 0;
        self.ip = 0;
        self.halted = false;
    }

    pub fn set_program(&mut self, program: Vec<u8>) {
        self.program = program;
        self.instruction_pos = vec![];
        let mut i = 0;
        while i < self.program.len() {
            self.instruction_pos.push(i);
            let param_count = (self.program[i] & 0b111) as usize;
            i += 1 + param_count;
        }
    }

    pub fn add_memory(&mut self, memory: SharedMemory) {
        self.memories.push(memory);
    }

    pub fn write_to(&self, address: &[u8], value: u64) {
        if address.is_empty() {
            return;
        }

        let mem_id = address[0] as usize;
        if mem_id >= self.memories.len() {
            return;
        }

        let addr = address[1..]
            .iter()
            .fold(0, |acc, &b| (acc << 8) | b as usize);
        let mut memory = self.memories[mem_id].borrow_mut();
        if addr >= memory.len() {
            return;
        }

        memory.write(addr as u64, value);
    }

    pub fn read_from(&self, address: &[u8]) -> u64 {
        if address.is_empty() {
            return 0;
        }

        let mem_id = address[0] as usize;
        if mem_id >= self.memories.len() {
            return 0;
        }

        let addr = address[1..]
            .iter()
            .fold(0, |acc, &b| (acc << 8) | b as usize);
        let memory = self.memories[mem_id].borrow();
        if addr >= memory.len() {
            return 0;
        }

        memory.read(addr as u64)
    }

    pub fn execute(&mut self, instruction: Vec<u8>) {
        let opcode = instruction[0] >> 3;
        match opcode {
            // Jump
            1 => {
                self.ip = instruction[1..]
                    .iter()
                    .fold(0, |acc, &b| (acc << 8) | b as usize)
            }
            // Jumpz
            2 => {
                if self.register == 0 {
                    self.ip = instruction[1..]
                        .iter()
                        .fold(0, |acc, &b| (acc << 8) | b as usize)
                }
            }
            // Jumpnz
            3 => {
                if self.register != 0 {
                    self.ip = instruction[1..]
                        .iter()
                        .fold(0, |acc, &b| (acc << 8) | b as usize)
                }
            }
            // Jumpreg
            4 => self.ip = self.register as usize,
            // Jumpind
            // TODO
            5 => self.ip = self.read_from(&instruction[1..]) as usize,
            // Line
            6 => self.register = self.ip as u64,
            // Load
            7 => {
                self.register = instruction[1..]
                    .iter()
                    .fold(0, |acc, &b| (acc << 8) | b as u64)
            }
            // Read
            8 => self.register = self.read_from(&instruction[1..]),
            // Write
            9 => self.write_to(&instruction[1..], self.register),
            // Readind
            10 => self.register = self.read_from(&self.read_from(&instruction[1..]).to_be_bytes()),
            // Writeind
            11 => self.write_to(
                &self.read_from(&instruction[1..]).to_be_bytes(),
                self.register,
            ),
            // Incr
            12 => self.register += 1,
            // Decr
            13 => self.register -= 1,
            // Neg
            14 => self.register = !self.register,
            // Not
            15 => {
                if self.register == 0 {
                    self.register = 1;
                } else {
                    self.register = 0;
                }
            }
            // Add
            16 => self.register += self.read_from(&instruction[1..]),
            // Sub
            17 => self.register -= self.read_from(&instruction[1..]),
            // Mul
            18 => self.register *= self.read_from(&instruction[1..]),
            // Div
            19 => self.register /= self.read_from(&instruction[1..]),
            // And
            20 => self.register &= self.read_from(&instruction[1..]),
            // Or
            21 => self.register |= self.read_from(&instruction[1..]),
            // Xor
            22 => self.register ^= self.read_from(&instruction[1..]),
            // Lshift
            23 => self.register <<= 1,
            // Rshift
            24 => self.register >>= 1,
            // Andi
            25 => {
                self.register &= instruction[1..]
                    .iter()
                    .fold(0, |acc, &b| (acc << 8) | b as u64)
            }
            // Ori
            26 => {
                self.register |= instruction[1..]
                    .iter()
                    .fold(0, |acc, &b| (acc << 8) | b as u64)
            }
            // Xori
            27 => {
                self.register ^= instruction[1..]
                    .iter()
                    .fold(0, |acc, &b| (acc << 8) | b as u64)
            }
            // Comp
            28 => {
                let value = self.read_from(&instruction[1..]);
                self.register = if self.register < value {
                    0xFFFFFFFFFFFFFFFF
                } else if self.register > value {
                    1
                } else {
                    0
                };
            }
            _ => self.halted = true,
        };
    }

    pub fn step(&mut self) {
        if self.halted || self.instruction_pos.is_empty() {
            return;
        }
        if self.ip >= self.instruction_pos.len() {
            self.halted = true;
            return;
        }

        let start = self.instruction_pos[self.ip];
        let end = if self.ip + 1 < self.instruction_pos.len() {
            self.instruction_pos[self.ip + 1]
        } else {
            self.program.len()
        };
        let instruction = self.program[start..end].to_vec();

        let prev_ip = self.ip;
        self.execute(instruction);

        // If execute didn't change ip (non-jump), advance to the next instruction.
        if self.ip == prev_ip {
            self.ip += 1;
        }

        if self.ip >= self.instruction_pos.len() {
            self.halted = true;
        }
    }
}
