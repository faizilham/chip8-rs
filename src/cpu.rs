use wasm_bindgen::prelude::*;
use crate::memory::{PROGRAM_START, Memory, allocate_memory};
use crate::utils;

const STACK_SIZE : usize = 64;
const REGISTER_SIZE : usize = 16;

pub struct CPU {
    memory: Memory,

    register: [u8; REGISTER_SIZE],
    ir: usize,                  // index register
    pc: usize,                  // program counter

    stack: [usize; STACK_SIZE],
    sp: usize,                  // stack pointer. 0 means empty,

    dt: u8,                     // delay timer
}

impl CPU {
    pub fn new() -> CPU {
        let register = [0; REGISTER_SIZE];
        let stack = [0; STACK_SIZE];
        let memory = allocate_memory();

        CPU {
            memory,
            register,
            ir: 0,
            pc: PROGRAM_START,
            stack,
            sp: 0,
            dt: 0
        }
    }

    pub fn rom_ptr(&mut self) -> *mut u8 {
        let ptr = unsafe {
            self.memory.as_mut_ptr().offset(PROGRAM_START as isize)
        };

        ptr
    }

    pub fn reset(&mut self) {
        self.ir = 0;
        self.pc = PROGRAM_START;
        self.sp = 0;
        self.dt = 0;

        for i in 0..REGISTER_SIZE {
            self.register[i] = 0;
        }
    }

    pub fn tick(&mut self) -> ExecutionStatus {
        // fetch
        let high = self.memory[self.pc];
        let low = self.memory[self.pc + 1];
        self.pc += 2;

        // parse
        let result = match get1(high, low) {
            0x0 => match low {
                // TODO: 00e0 clear display
                0xE0 => unimplemented("clear display"),

                // 00ee return
                0xEE => self.op_00ee_ret(),

                // -- 0nnn syscall, ignored
                _ => ExecutionStatus::OK
            },

            // 1nnn jump
            0x1 => self.op_1nnn_jump(high, low),

            // 2nnn call
            0x2 => self.op_2nnn_call(high, low),

            // 3xkk skip eq Vx, byte
            0x3 => self.op_3xkk_skipeq(high, low),

            // 4xkk skip neq Vx, byte
            0x4 => self.op_4xkk_skipneq(high, low),

            // 5xy0 skip eq Vx, Vy
            0x5 => self.op_5xy0_skipeqv(high, low),

            // 6xkk load Vx = byte
            0x6 => self.op_6xkk_load(high, low),

            // 7xkk incr Vx += byte
            0x7 => self.op_7xkk_incr(high, low),

            // binary operator instructions
            0x8 => {
                let x = get2(high, low) as usize;
                let y = get3(high, low) as usize;

                match get4(high, low) {
                    // 8xy0 set Vx = Vy
                    0x0 => self.op_8xy0_set(x, y),

                    // 8xy1 or Vx |=Vy
                    0x1 => self.op_8xy1_or(x, y),

                    // 8xy2 and Vx &= Vy
                    0x2 => self.op_8xy2_and(x, y),

                    // 8xy3 xor Vx ^= Vy
                    0x3 => self.op_8xy3_xor(x, y),

	                // 8xy4 add Vx = Vx + Vy, VF = carry
                    0x4 => self.op_8xy4_add(x, y),

                    // 8xy5 sub Vx = Vx - Vy, VF = not borrow (Vx >= Vy)
                    0x5 => self.op_8xy5_sub(x, y),

	                // 8xy6 shr Vx = Vx >> 1. VF = last bit
                    0x6 => self.op_8xy6_shr(x, y),

                    // 8xy7 subn Vx = Vy - Vx, VF = not borrow (Vy >= Vx)
                    0x7 => self.op_8xy7_subn(x, y),

                    // 8xyE shl Vx = Vx << 1. VF = first bit
                    0xE => self.op_8xye_shl(x, y),


                    _ => runtime_error("Unknown opcode")
                }
            }

            // 9xy0 skip ne v Vx != Vy
            0x9 => self.op_9xy0_skipnev(high, low),

            // annn loadi I = annn
            0xA => self.op_annn_loadi(high, low),

            // bnnn jumpv v0 + nnn
            0xB => self.op_bnnn_jumpv(high, low),

            // cxkk rand Vx = rand() & byte
            0xC => self.op_cxkk_rand(high, low),

            // TODO: dxyn - draw Vx, Vy, nibble
            0xD => unimplemented("dxyn draw"),

            _ => {
                runtime_error("Unknown opcode")
            }
        };

        result
    }

    // OPCODES

    // 00EE return
    fn op_00ee_ret(&mut self) -> ExecutionStatus {
        if self.sp == 0{
            return runtime_error("Stack underflow");
        }

        self.sp -= 1;

        self.pc = self.stack[self.sp];

        ExecutionStatus::OK
    }

    // 1nnn jump
    fn op_1nnn_jump(&mut self, high: u8, low: u8) -> ExecutionStatus {
        let addr = get_nnn(high, low) as usize;

        // handle trap jump: prev_pc (current_pc - 2) = addr
        if self.pc == addr + 2 {
            return ExecutionStatus::Halt;
        }

        self.pc = addr;

        ExecutionStatus::OK
    }

    // 2nnn call
    fn op_2nnn_call(&mut self, high: u8, low: u8) -> ExecutionStatus {
        if self.sp == STACK_SIZE {
            return runtime_error("Stack overflow");
        }

        self.stack[self.sp] = self.pc;
        self.sp += 1;

        let addr = get_nnn(high, low) as usize;
        self.pc = addr;

        ExecutionStatus::OK
    }

    // 3xkk skip eq Vx, byte
    fn op_3xkk_skipeq(&mut self, high: u8, low: u8) -> ExecutionStatus {
        let x = get2(high, low) as usize;
        let kk = get_kk(high, low);

        if self.register[x] == kk {
            self.pc += 2;
        }

        ExecutionStatus::OK
    }

    // 4xkk skip neq Vx, byte
    fn op_4xkk_skipneq(&mut self, high: u8, low: u8) -> ExecutionStatus {
        let x = get2(high, low) as usize;
        let kk = get_kk(high, low);

        if self.register[x] != kk {
            self.pc += 2;
        }

        ExecutionStatus::OK
    }

    // 5xy0 skip eq Vx, Vy
    fn op_5xy0_skipeqv(&mut self, high: u8, low: u8) -> ExecutionStatus {
        let x = get2(high, low) as usize;
        let y = get3(high, low) as usize;

        if self.register[x] == self.register[y] {
            self.pc += 2;
        }

        ExecutionStatus::OK
    }

    // 6xkk load Vx = byte
    fn op_6xkk_load(&mut self, high: u8, low: u8) -> ExecutionStatus {
        let x = get2(high, low) as usize;
        let kk = get_kk(high, low);

        self.register[x] = kk;

        ExecutionStatus::OK
    }

    // 7xkk incr Vx += byte
    fn op_7xkk_incr(&mut self, high: u8, low: u8) -> ExecutionStatus {
        let x = get2(high, low) as usize;
        let kk = get_kk(high, low);

        self.register[x] += kk;

        ExecutionStatus::OK
    }

    // 8xy0 set Vx = Vy
    fn op_8xy0_set(&mut self, x: usize, y: usize) -> ExecutionStatus {
        self.register[x] = self.register[y];
        ExecutionStatus::OK
    }

    // 8xy1 or Vx |=Vy
    fn op_8xy1_or(&mut self, x: usize, y: usize) -> ExecutionStatus {
        self.register[x] |= self.register[y];
        ExecutionStatus::OK
    }

    // 8xy2 and Vx &= Vy
    fn op_8xy2_and(&mut self, x: usize, y: usize) -> ExecutionStatus {
        self.register[x] &= self.register[y];
        ExecutionStatus::OK
    }

    // 8xy3 xor Vx ^= Vy
    fn op_8xy3_xor(&mut self, x: usize, y: usize) -> ExecutionStatus {
        self.register[x] ^= self.register[y];
        ExecutionStatus::OK
    }

    // 8xy4 add Vx = Vx + Vy, VF = carry
    fn op_8xy4_add(&mut self, x: usize, y: usize) -> ExecutionStatus {
        let temp = self.register[x] as u16 + self.register[y] as u16;

        self.register[x] = (temp & 0x00FF) as u8;
        self.register[0xF] = (temp >> 8) as u8;

        ExecutionStatus::OK
    }

    // 8xy5 sub Vx = Vx - Vy, VF = not borrow (Vx >= Vy)
    fn op_8xy5_sub(&mut self, x: usize, y: usize) -> ExecutionStatus {
        let temp = self.register[x] as i16 - self.register[y] as i16;

        self.register[x] = (temp & 0x00FF) as u8;
        self.register[0xF] = ((temp >> 8) + 1) as u8;

        ExecutionStatus::OK
    }

    // 8xy6 shr Vx = Vx >> 1. VF = last bit
    fn op_8xy6_shr(&mut self, x: usize, _y: usize) -> ExecutionStatus {
        self.register[0xF] = self.register[x] & 0x01;
        self.register[x] >>= 1;
        ExecutionStatus::OK
    }

    // 8xy7 subn Vx = Vy - Vx, VF = not borrow (Vy >= Vx)
    fn op_8xy7_subn(&mut self, x: usize, y: usize) -> ExecutionStatus {
        let temp = self.register[y] as i16 - self.register[x] as i16;

        self.register[x] = (temp & 0x00FF) as u8;
        self.register[0xF] = ((temp >> 8) + 1) as u8;

        ExecutionStatus::OK
    }

    // 8xyE shl Vx = Vx << 1. VF = first bit
    fn op_8xye_shl(&mut self, x: usize, _y: usize) -> ExecutionStatus {
        self.register[0xF] = (self.register[x] & 0x80) >> 7;
        self.register[x] <<= 1;

        ExecutionStatus::OK
    }

    // 9xy0 skip ne v Vx != Vy
    fn op_9xy0_skipnev(&mut self, high: u8, low: u8) -> ExecutionStatus {
        let x = get2(high, low) as usize;
        let y = get3(high, low) as usize;

        if self.register[x] != self.register[y] {
            self.pc += 2;
        }

        ExecutionStatus::OK
    }

    // annn loadi I = annn
    fn op_annn_loadi(&mut self, high: u8, low: u8) -> ExecutionStatus {
        self.ir = get_nnn(high, low) as usize;

        ExecutionStatus::OK
    }

    // bnnn jumpv v0 + nnn
    fn op_bnnn_jumpv(&mut self, high: u8, low: u8) -> ExecutionStatus {
        let addr = self.register[0] as usize + get_nnn(high, low) as usize;

        self.pc = addr;

        ExecutionStatus::OK
    }

    // cxkk rand Vx = rand() & byte
    fn op_cxkk_rand(&mut self, high: u8, low: u8) -> ExecutionStatus {
        let x = get2(high, low) as usize;
        let kk = get_kk(high, low);

        self.register[x] = utils::random() & kk;

        ExecutionStatus::OK
    }
}

// UTILITIES
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionStatus {
    OK,
    Halt,
    RuntimeError,
}

fn runtime_error(_s : &str) -> ExecutionStatus {
    log!("Runtime Error: {}", _s);
    ExecutionStatus::RuntimeError
}

fn unimplemented(_s: &str) -> ExecutionStatus {
    log!("Unimplemented Error: {}", _s);
    ExecutionStatus::RuntimeError
}

#[inline]
fn get1(high: u8, _low: u8) -> u8 {
    (high >> 4) & 0x0F
}

#[inline]
fn get2(high: u8, _low: u8) -> u8 {
    high & 0x0F
}

#[inline]
fn get3(_high: u8, low: u8) -> u8 {
	(low >> 4) & 0x0F
}

#[inline]
fn get4(_high: u8, low: u8) -> u8 {
    low & 0x0F
}

#[inline]
fn get_kk(_high: u8, low: u8) -> u8 {
    low
}

#[inline]
fn get_nnn(high: u8, low: u8) -> u16 {
    let front = get2(high, low) as u16;
    let back = get_kk(high, low) as u16;

    (front << 8) | back
}

// UNIT TEST MODULE
#[cfg(test)]
mod test;
