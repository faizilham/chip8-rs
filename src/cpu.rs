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
            0x0 => {
                // 0x00E0 clear display
                // 0x00EE return
                // 0x0nnn -- syscall, not implemented
                log!("bytes {:x} {:x}", high, low);
                Ok(())
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

            _ => {
                runtime_error("unknown opcode")
            }
        };

        if let Err(status) = result {
            return status;
        }

        ExecutionStatus::OK
    }

    // OPCODES

    // 1nnn jump
    fn op_1nnn_jump(&mut self, high: u8, low: u8) -> ExecutionResult {
        let addr = get_nnn(high, low) as usize;

        // handle trap jump: prev_pc (current_pc - 2) = addr
        if self.pc == addr + 2 {
            return halt();
        }

        self.pc = addr;

        Ok(())
    }

    // 2nnn call
    fn op_2nnn_call(&mut self, high: u8, low: u8) -> ExecutionResult {
        if self.sp == STACK_SIZE {
            return runtime_error("stack overflow");
        }

        self.stack[self.sp] = self.pc;
        self.sp += 1;

        let addr = get_nnn(high, low) as usize;
        self.pc = addr;

        Ok(())
    }

    // 3xkk skip eq Vx, byte
    fn op_3xkk_skipeq(&mut self, high: u8, low: u8) -> ExecutionResult {
        let x = get2(high, low) as usize;
        let kk = get_kk(high, low);

        if self.register[x] == kk {
            self.pc += 2;
        }

        Ok(())
    }

    // 4xkk skip neq Vx, byte
    fn op_4xkk_skipneq(&mut self, high: u8, low: u8) -> ExecutionResult {
        let x = get2(high, low) as usize;
        let kk = get_kk(high, low);

        if self.register[x] != kk {
            self.pc += 2;
        }

        Ok(())
    }

    // 5xy0 skip eq Vx, Vy
    fn op_5xy0_skipeqv(&mut self, high: u8, low: u8) -> ExecutionResult {
        let x = get2(high, low) as usize;
        let y = get3(high, low) as usize;

        if self.register[x] == self.register[y] {
            self.pc += 2;
        }

        Ok(())
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

pub type ExecutionResult = Result<(), ExecutionStatus>;

pub fn runtime_error(s : &str) -> ExecutionResult {
    log!("Runtime Error: {}", s);
    Err(ExecutionStatus::RuntimeError)
}

pub fn halt() -> ExecutionResult {
    Err(ExecutionStatus::Halt)
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
    fn test_op_1nnn_jump() {
        let mut tester = CPUTester::new();

        let addr = 0x456;

        tester.set_ops(0x14, 0x56);

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, addr);

        tester.set_ops(0x12, 0x00);

        let result = tester.tick_cpu();
        assert_eq!(result, ExecutionStatus::Halt);
    }

    #[wasm_bindgen_test]
    fn test_op_2nnn_call() {
        let mut tester = CPUTester::new();

        tester.set_ops(0x24, 0x56);

        let addr = 0x456;

        let last_pc = tester.cpu.pc + 2;
        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, addr as usize);
        assert_eq!(tester.cpu.sp, 1);
        assert_eq!(tester.cpu.stack[0], last_pc);

        tester.reset();

        tester.cpu.sp = STACK_SIZE;

        let result = tester.tick_cpu();
        assert_eq!(result, ExecutionStatus::RuntimeError);
    }

    #[wasm_bindgen_test]
    fn test_op_3xkk_skipeq() {
        let mut tester = CPUTester::new();

        let val = 0xAF;

        tester.set_ops(0x30, 0xAF);
        tester.cpu.register[0] = val;

        let pc = tester.cpu.pc + 2;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, pc + 2);

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

        tester.set_ops(0x40, 0xAF);
        tester.cpu.register[0] = val;

        let pc = tester.cpu.pc + 2;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, pc);

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

        tester.set_ops(0x50, 0x20);
        tester.cpu.register[0] = val;
        tester.cpu.register[2] = val;

        let pc = tester.cpu.pc + 2;

        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, pc + 2);

        tester.set_ops(0x50, 0x10);
        tester.cpu.register[0] = val;
        tester.cpu.register[2] = val;

        let pc = tester.cpu.pc + 2;
        let result = tester.tick_cpu();

        assert_eq!(result, ExecutionStatus::OK);
        assert_eq!(tester.cpu.pc, pc);
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
