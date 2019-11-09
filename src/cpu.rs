use wasm_bindgen::prelude::*;
use crate::memory::{PROGRAM_START, Memory, allocate_memory};

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
                // 00e0 clear display
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

                    0x7 => unimplemented("subn"),

                    // 8xyE shl Vx = Vx << 1. VF = first bit
                    0xE => self.op_8xye_shl(x, y),


                    _ => runtime_error("Unknown opcode")
                }
            }

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

    // 8xyE shl Vx = Vx << 1. VF = first bit
    fn op_8xye_shl(&mut self, x: usize, _y: usize) -> ExecutionStatus {
        self.register[0xF] = (self.register[x] & 0x80) >> 7;
        self.register[x] <<= 1;

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

fn runtime_error(s : &str) -> ExecutionStatus {
    log!("Runtime Error: {}", s);
    ExecutionStatus::RuntimeError
}

fn unimplemented(s: &str) -> ExecutionStatus {
    log!("Unimplemented Error: {}", s);
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

// UNIT TESTS

#[cfg(test)]
mod test {
    use wasm_bindgen_test::*;
    use crate::cpu::*;

    struct CPUTester {
        cpu: CPU
    }

    impl CPUTester {
        pub fn new() -> CPUTester {
            CPUTester { cpu: CPU::new() }
        }

        fn reset(&mut self) {
            self.cpu.reset();
        }

        fn set_ops(&mut self, high: u8, low: u8) {
            self.cpu.reset();
            self.cpu.memory[PROGRAM_START] = high;
            self.cpu.memory[PROGRAM_START + 1] = low;
        }

        fn tick_cpu(&mut self) -> ExecutionStatus {
            self.cpu.tick()
        }
    }

    // test cpu method

    #[wasm_bindgen_test]
    fn test_get_program_memory() {
        let mut cpu = CPU::new();

        let expected_value = 0x2A;

        cpu.memory[PROGRAM_START] = expected_value;

        let ptr = cpu.rom_ptr();
        let ptr_value = unsafe { *ptr };

        assert_eq!(expected_value, ptr_value);
    }

    #[wasm_bindgen_test]
    fn test_op_00ee_ret() {
        let mut tester = CPUTester::new();

        let addr = 0x456;

        // success case
        tester.set_ops(0x00, 0xEE);
        tester.cpu.stack[0] = addr;
        tester.cpu.sp = 1;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, addr as usize);
        assert_eq!(tester.cpu.sp, 0);

        // stack underflow case
        tester.reset();

        let result = tester.tick_cpu();
        assert_eq!(result, ExecutionStatus::RuntimeError);
    }

    #[wasm_bindgen_test]
    fn test_op_1nnn_jump() {
        let mut tester = CPUTester::new();

        let addr = 0x456;

        // success case
        tester.set_ops(0x14, 0x56);

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, addr);

        // trap jump case
        tester.set_ops(0x12, 0x00);

        let result = tester.tick_cpu();
        assert_eq!(result, ExecutionStatus::Halt);
    }

    #[wasm_bindgen_test]
    fn test_op_2nnn_call() {
        let mut tester = CPUTester::new();

        let addr = 0x456;

        // success case
        tester.set_ops(0x24, 0x56);

        let last_pc = tester.cpu.pc + 2;
        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, addr as usize);
        assert_eq!(tester.cpu.sp, 1);
        assert_eq!(tester.cpu.stack[0], last_pc);

        // stack overflow case
        tester.reset();

        tester.cpu.sp = STACK_SIZE;

        let result = tester.tick_cpu();
        assert_eq!(result, ExecutionStatus::RuntimeError);
    }

    #[wasm_bindgen_test]
    fn test_op_3xkk_skipeq() {
        let mut tester = CPUTester::new();

        let val = 0xAF;

        // skip case
        tester.set_ops(0x30, 0xAF);
        tester.cpu.register[0] = val;

        let pc = tester.cpu.pc + 2;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, pc + 2);

        // no skip case
        tester.set_ops(0x31, 0xAF);
        tester.cpu.register[0] = val;

        let pc = tester.cpu.pc + 2;
        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, pc);
    }

    #[wasm_bindgen_test]
    fn test_op_4xkk_skipneq() {
        let mut tester = CPUTester::new();

        let val = 0xAF;

        // no skip case
        tester.set_ops(0x40, 0xAF);
        tester.cpu.register[0] = val;

        let pc = tester.cpu.pc + 2;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, pc);

        // skip case
        tester.set_ops(0x41, 0xAF);
        tester.cpu.register[0] = val;

        let pc = tester.cpu.pc + 2;
        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, pc + 2);
    }

    #[wasm_bindgen_test]
    fn test_op_5xy0_skipeqv() {
        let mut tester = CPUTester::new();

        let val = 0xAF;

        // skip case
        tester.set_ops(0x50, 0x20);
        tester.cpu.register[0] = val;
        tester.cpu.register[2] = val;

        let pc = tester.cpu.pc + 2;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, pc + 2);

        // no skip case
        tester.set_ops(0x50, 0x10);
        tester.cpu.register[0] = val;
        tester.cpu.register[2] = val;

        let pc = tester.cpu.pc + 2;
        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, pc);
    }

    #[wasm_bindgen_test]
    fn test_op_6xkk_load() {
        let mut tester = CPUTester::new();

        let val = 0x56;

        // reg 0 case
        tester.set_ops(0x60, 0x56);

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], val);

        // reg 3 case
        let val = 0x52;

        tester.set_ops(0x63, 0x52);

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[3], val);
    }

    #[wasm_bindgen_test]
    fn test_op_7xkk_load() {
        let mut tester = CPUTester::new();

        let val = 0xCA + 0x02;

        // reg 0 case
        tester.set_ops(0x70, 0x02);
        tester.cpu.register[0] = 0xCA;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], val);

        // reg 3 case
        let val = 0x47 + 0x03;

        tester.set_ops(0x73, 0x03);
        tester.cpu.register[3] = 0x47;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[3], val);
    }

    #[wasm_bindgen_test]
    fn test_op_8xy0_set() {
        let mut tester = CPUTester::new();

        let val = 0x28;

        // (0, 1) case
        tester.set_ops(0x80, 0x10);
        tester.cpu.register[0] = 0;
        tester.cpu.register[1] = val;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], val);

        // (1, 0) case
        tester.set_ops(0x81, 0x00);
        tester.cpu.register[0] = val;
        tester.cpu.register[1] = 0;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[1], val);
    }

    #[wasm_bindgen_test]
    fn test_op_8xy1_or() {
        let mut tester = CPUTester::new();

        let left = 0x28;
        let right = 0x5E;
        let expected = left | right;

        // (0, 1) case
        tester.set_ops(0x80, 0x11);
        tester.cpu.register[0] = left;
        tester.cpu.register[1] = right;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], expected);

        // (1, 0) case
        tester.set_ops(0x81, 0x01);
        tester.cpu.register[0] = right;
        tester.cpu.register[1] = left;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[1], expected);

        // self case
        let val = 0xFA;
        tester.set_ops(0x80, 0x01);
        tester.cpu.register[0] = val;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], val);
    }

    #[wasm_bindgen_test]
    fn test_op_8xy2_and() {
        let mut tester = CPUTester::new();

        let left = 0x28;
        let right = 0x5E;
        let expected = left & right;

        // (0, 1) case
        tester.set_ops(0x80, 0x12);
        tester.cpu.register[0] = left;
        tester.cpu.register[1] = right;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], expected);

        // (1, 0) case
        tester.set_ops(0x81, 0x02);
        tester.cpu.register[0] = right;
        tester.cpu.register[1] = left;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[1], expected);

        // self case
        let val = 0xFA;
        tester.set_ops(0x80, 0x02);
        tester.cpu.register[0] = val;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], val);
    }

    #[wasm_bindgen_test]
    fn test_op_8xy3_xor() {
        let mut tester = CPUTester::new();

        let left = 0x28;
        let right = 0x5E;
        let expected = left ^ right;

        // (0, 1) case
        tester.set_ops(0x80, 0x13);
        tester.cpu.register[0] = left;
        tester.cpu.register[1] = right;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], expected);

        // (1, 0) case
        tester.set_ops(0x81, 0x03);
        tester.cpu.register[0] = right;
        tester.cpu.register[1] = left;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[1], expected);

        // self case
        tester.set_ops(0x80, 0x03);
        tester.cpu.register[0] = 0xFA;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], 0);
    }

    #[wasm_bindgen_test]
    fn test_op_8xy4_add() {
        let mut tester = CPUTester::new();

        // overflow case
        let x = 0xFF;
        let y = 0xFE;
        let sum = 0xFD;
        let expect_vf = 1;

        tester.set_ops(0x80, 0x14);
        tester.cpu.register[0] = x;
        tester.cpu.register[1] = y;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], sum);
        assert_eq!(tester.cpu.register[0xF], expect_vf);

        // non-overflow case
        let x = 0xFA;
        let y = 0x02;
        let sum = 0xFC;
        let expect_vf = 0;

        tester.set_ops(0x80, 0x14);
        tester.cpu.register[0] = x;
        tester.cpu.register[1] = y;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], sum);
        assert_eq!(tester.cpu.register[0xF], expect_vf);

        // reverse case
        let x = 0xFF;
        let y = 0xFE;
        let sum = 0xFD;
        let expect_vf = 1;

        tester.set_ops(0x81, 0x04);
        tester.cpu.register[0] = x;
        tester.cpu.register[1] = y;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[1], sum);
        assert_eq!(tester.cpu.register[0xF], expect_vf);

        // self case
        let x = 0xFF;
        let sum = 0xFE;
        let expect_vf = 1;

        tester.set_ops(0x80, 0x04);
        tester.cpu.register[0] = x;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], sum);
        assert_eq!(tester.cpu.register[0xF], expect_vf);
    }

    #[wasm_bindgen_test]
    fn test_op_8xy5_sub() {
        let mut tester = CPUTester::new();

        // non borrow case Vx >= Vy
        let x = 0xFF;
        let y = 0x01;
        let sum = 0xFE;
        let expect_vf = 1;

        tester.set_ops(0x80, 0x15);
        tester.cpu.register[0] = x;
        tester.cpu.register[1] = y;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], sum);
        assert_eq!(tester.cpu.register[0xF], expect_vf);

        // borrow case Vx < Vy
        let x = 0x01;
        let y = 0x04;
        let sum = 0xFD;
        let expect_vf = 0;

        tester.set_ops(0x80, 0x15);
        tester.cpu.register[0] = x;
        tester.cpu.register[1] = y;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], sum);
        assert_eq!(tester.cpu.register[0xF], expect_vf);

        // reverse case
        let x = 0xFF;
        let y = 0x01;
        let sum = 0xFE;
        let expect_vf = 1;

        tester.set_ops(0x81, 0x05);
        tester.cpu.register[0] = y;
        tester.cpu.register[1] = x;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[1], sum);
        assert_eq!(tester.cpu.register[0xF], expect_vf);

        // self case
        let sum = 0;
        let expect_vf = 1;

        tester.set_ops(0x80, 0x05);
        tester.cpu.register[0] = 0xFA;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], sum);
        assert_eq!(tester.cpu.register[0xF], expect_vf);
    }

    #[wasm_bindgen_test]
    fn test_op_8xy6_shr() {
        let mut tester = CPUTester::new();

        // underflow case
        let val = 0b0000_0111;
        let fin_val = val >> 1;
        let expect_vf = 1;

        tester.set_ops(0x80, 0x16);
        tester.cpu.register[0] = val;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], fin_val);
        assert_eq!(tester.cpu.register[0xF], expect_vf);

        // non underflow case
        let val = 0b0000_0110;
        let fin_val = val >> 1;
        let expect_vf = 0;

        tester.set_ops(0x80, 0x16);
        tester.cpu.register[0] = val;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], fin_val);
        assert_eq!(tester.cpu.register[0xF], expect_vf);
    }

    #[wasm_bindgen_test]
    fn test_op_8xye_shl() {
        let mut tester = CPUTester::new();

        // overflow case
        let val = 0b1101_0000;
        let fin_val = val << 1;
        let expect_vf = 1;

        tester.set_ops(0x80, 0x1E);
        tester.cpu.register[0] = val;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], fin_val);
        assert_eq!(tester.cpu.register[0xF], expect_vf);

        // non overflow case
        let val = 0b0101_0000;
        let fin_val = val << 1;
        let expect_vf = 0;

        tester.set_ops(0x80, 0x1E);
        tester.cpu.register[0] = val;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.register[0], fin_val);
        assert_eq!(tester.cpu.register[0xF], expect_vf);
    }

    // test utils

    #[wasm_bindgen_test]
    fn test_get_utils() {
        let high = 0x1F;
        let low = 0x2A;

        assert_eq!(0x1, get1(high, low));
        assert_eq!(0xF, get2(high, low));
        assert_eq!(0x2, get3(high, low));
        assert_eq!(0xA, get4(high, low));
        assert_eq!(0x2A, get_kk(high, low));
        assert_eq!(0xF2A, get_nnn(high, low));
    }
}
