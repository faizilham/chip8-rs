/****
 * CPU Unit Test
 */

use wasm_bindgen_test::*;
use super::*;
use crate::iodevice::IOInterface;

// TEST UTILS

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

// TEST CPU PUBLIC METHOD

#[wasm_bindgen_test]
fn test_get_program_memory() {
    let mut cpu = CPU::new();

    let expected_value = 0x2A;

    cpu.memory[PROGRAM_START] = expected_value;

    let ptr = cpu.rom_ptr();
    let ptr_value = unsafe { *ptr };

    assert_eq!(expected_value, ptr_value);
}

// TEST CPU INSTRUCTIONS

struct MockDevice {}

impl IOInterface for MockDevice {
    fn clear_display(&mut self) {}

    fn draw_pixel(&mut self, _x: u8, _y: u8) -> u8 {
        0
    }

    fn key_pressed(&self, _key: u8) -> bool {
        false
    }

    fn read_any_key(&mut self) -> u8 {
        0
    }
}

struct CPUTester {
    cpu: CPU,
    device: MockDevice,
}

impl CPUTester {
    pub fn new() -> CPUTester {
        CPUTester { cpu: CPU::new(), device: MockDevice{} }
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
        self.cpu.tick(&mut self.device)
    }
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
fn test_op_8xy7_subn() {
    let mut tester = CPUTester::new();

    // non borrow case Vx >= Vy
    let x = 0x01;
    let y = 0xFF;
    let sum = 0xFE;
    let expect_vf = 1;

    tester.set_ops(0x80, 0x17);
    tester.cpu.register[0] = x;
    tester.cpu.register[1] = y;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.register[0], sum);
    assert_eq!(tester.cpu.register[0xF], expect_vf);

    // borrow case Vx < Vy
    let x = 0x04;
    let y = 0x01;
    let sum = 0xFD;
    let expect_vf = 0;

    tester.set_ops(0x80, 0x17);
    tester.cpu.register[0] = x;
    tester.cpu.register[1] = y;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.register[0], sum);
    assert_eq!(tester.cpu.register[0xF], expect_vf);

    // reverse case
    let x = 0x01;
    let y = 0xFF;
    let sum = 0xFE;
    let expect_vf = 1;

    tester.set_ops(0x81, 0x07);
    tester.cpu.register[0] = y;
    tester.cpu.register[1] = x;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.register[1], sum);
    assert_eq!(tester.cpu.register[0xF], expect_vf);

    // self case
    let sum = 0;
    let expect_vf = 1;

    tester.set_ops(0x80, 0x07);
    tester.cpu.register[0] = 0xFA;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.register[0], sum);
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

#[wasm_bindgen_test]
fn test_op_9xy0_skipnev() {
    let mut tester = CPUTester::new();

    let val = 0xAF;

    // skip case
    tester.set_ops(0x90, 0x10);
    tester.cpu.register[0] = val;
    tester.cpu.register[2] = val;

    let pc = tester.cpu.pc + 2;
    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.pc, pc + 2);

    // no skip case
    tester.set_ops(0x90, 0x20);
    tester.cpu.register[0] = val;
    tester.cpu.register[2] = val;

    let pc = tester.cpu.pc + 2;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.pc, pc);
}

#[wasm_bindgen_test]
fn test_op_annn_loadi() {
    let mut tester = CPUTester::new();

    let val = 0x356;

    tester.set_ops(0xa3, 0x56);

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.ir, val);
}

#[wasm_bindgen_test]
fn test_op_bnnn_jumpv() {
    let mut tester = CPUTester::new();

    let addr = 0x456;

    // success case
    tester.set_ops(0xb4, 0x56);

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.pc, addr);
}


#[wasm_bindgen_test]
fn test_op_cxkk_load() {
    let mock_random_value = utils::random();
    let mut tester = CPUTester::new();

    // reg 0 case
    let val = mock_random_value & 0x56;
    tester.set_ops(0xC0, 0x56);

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.register[0], val);

    // reg 3 case
    let val = mock_random_value & 0x4A;
    tester.set_ops(0xC3, 0x4A);

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.register[3], val);
}

#[wasm_bindgen_test]
fn test_op_fx07_readdt() {
    let mut tester = CPUTester::new();
    let val = 0x23;

    // reg 0 case
    tester.set_ops(0xF0, 0x07);
    tester.cpu.dt = val;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.register[0], val);

    // reg 3 case
    tester.set_ops(0xF3, 0x07);
    tester.cpu.dt = val;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.register[3], val);
}

#[wasm_bindgen_test]
fn test_op_fx15_loaddt() {
    let mut tester = CPUTester::new();
    let val = 0x23;

    // reg 0 case
    tester.set_ops(0xF0, 0x15);
    tester.cpu.register[0] = val;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.dt, val);

    // reg 3 case
    tester.set_ops(0xF3, 0x15);
    tester.cpu.register[3] = val;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.dt, val);
}

#[wasm_bindgen_test]
fn test_op_fx1e_addi() {
    let mut tester = CPUTester::new();

    let i_val: usize = 0x235;

    // reg 0 case
    let val: usize = 0x56;
    tester.set_ops(0xf0, 0x1e);
    tester.cpu.ir = i_val;
    tester.cpu.register[0] = val as u8;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.ir, i_val + val);

    // reg 3 case
    let val: usize = 0xFF;
    tester.set_ops(0xf3, 0x1e);
    tester.cpu.ir = i_val;
    tester.cpu.register[3] = val as u8;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.ir, i_val + val);
}

#[wasm_bindgen_test]
fn test_op_fx29_digit() {
    let mut tester = CPUTester::new();

    // reg 0 case
    let val: usize = 0x0A;
    tester.set_ops(0xf0, 0x29);
    tester.cpu.register[0] = val as u8;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.ir, 5 * val);

    // reg 3 case
    let val: usize = 0x09;
    tester.set_ops(0xf3, 0x29);
    tester.cpu.register[3] = val as u8;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.ir, 5 * val);
}


#[wasm_bindgen_test]
fn test_op_fx33_bcd() {
    let mut tester = CPUTester::new();

    let ir = 0x500;

    // case full
    let val = 123;
    let bcd = [1, 2, 3];

    tester.set_ops(0xf0, 0x33);
    tester.cpu.register[0] = val;
    tester.cpu.ir = ir;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    for i in 0..3 {
        assert_eq!(tester.cpu.memory[ir + i], bcd[i]);
    }

    // no hundreds
    let val = 23;
    let bcd = [0, 2, 3];

    tester.set_ops(0xf0, 0x33);
    tester.cpu.register[0] = val;
    tester.cpu.ir = ir;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    for i in 0..3 {
        assert_eq!(tester.cpu.memory[ir + i], bcd[i]);
    }

    // only last digit
    let val = 3;
    let bcd = [0, 0, 3];

    tester.set_ops(0xf0, 0x33);
    tester.cpu.register[0] = val;
    tester.cpu.ir = ir;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    for i in 0..3 {
        assert_eq!(tester.cpu.memory[ir + i], bcd[i]);
    }

    // middle empty
    let val = 103;
    let bcd = [1, 0, 3];

    tester.set_ops(0xf0, 0x33);
    tester.cpu.register[0] = val;
    tester.cpu.ir = ir;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    for i in 0..3 {
        assert_eq!(tester.cpu.memory[ir + i], bcd[i]);
    }

    // from reg 3
    let val = 120;
    let bcd = [1, 2, 0];

    tester.set_ops(0xf3, 0x33);
    tester.cpu.register[3] = val;
    tester.cpu.ir = ir;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    for i in 0..3 {
        assert_eq!(tester.cpu.memory[ir + i], bcd[i]);
    }

    // different ir
    let ir = 0x300;
    let val = 125;
    let bcd = [1, 2, 5];

    tester.set_ops(0xf0, 0x33);
    tester.cpu.register[0] = val;
    tester.cpu.ir = ir;

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    for i in 0..3 {
        assert_eq!(tester.cpu.memory[ir + i], bcd[i]);
    }
}

// fx55 storeg M[I..I+x] = [V0..Vx], I += x + 1

#[wasm_bindgen_test]
fn test_op_fx55_storeg() {
    let mut tester = CPUTester::new();
    let ir = 0x500;

    // case x > 0 (len > 1)
    let vals = [5, 2, 7, 4, 1];
    let nvals = vals.len();
    let x = (nvals - 1) as u8;

    tester.set_ops(0xf0 | x, 0x55);
    tester.cpu.ir = ir;

    for i in 0..nvals {
        tester.cpu.register[i] = vals[i];
    }

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    for i in 0..nvals {
        assert_eq!(tester.cpu.memory[ir + i], vals[i]);
    }

    assert_eq!(tester.cpu.ir, ir + x as usize + 1);

    // case x == 0 (len == 1)
    let vals = [7];
    let nvals = vals.len();
    let x = (nvals - 1) as u8;

    tester.set_ops(0xf0 | x, 0x55);
    tester.cpu.ir = ir;

    for i in 0..nvals {
        tester.cpu.register[i] = vals[i];
    }

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    for i in 0..nvals {
        assert_eq!(tester.cpu.memory[ir + i], vals[i]);
    }

    assert_eq!(tester.cpu.ir, ir + x as usize + 1);
}

// fx65 ldreg [V0..Vx] = M[I..I+x], I += x + 1
#[wasm_bindgen_test]
fn test_op_fx65_ldreg() {
    let mut tester = CPUTester::new();

    // case x > 0 (len > 1)
    let ir = 0x500;
    let vals = [5, 2, 7, 4, 1];
    let nvals = vals.len();
    let x = (nvals - 1) as u8;

    tester.set_ops(0xf0 | x, 0x65);
    tester.cpu.ir = ir;

    for i in 0..nvals {
        tester.cpu.memory[ir + i] = vals[i];
    }

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.ir, ir + x as usize + 1);

    for i in 0..nvals {
        assert_eq!(tester.cpu.register[i], vals[i]);
    }

    for i in nvals..16-nvals {
        assert_eq!(tester.cpu.register[i], 0);
    }

    // case x == 0 (len == 1)
    let ir = 0x520;
    let vals = [8];
    let nvals = vals.len();
    let x = (nvals - 1) as u8;

    tester.set_ops(0xf0 | x, 0x65);
    tester.cpu.ir = ir;

    for i in 0..nvals {
        tester.cpu.memory[ir + i] = vals[i];
    }

    let result = tester.tick_cpu();

    assert_eq!(result, ExecutionStatus::OK);
    assert_eq!(tester.cpu.ir, ir + x as usize + 1);

    for i in 0..nvals {
        assert_eq!(tester.cpu.register[i], vals[i]);
    }

    for i in nvals..16-nvals {
        assert_eq!(tester.cpu.register[i], 0);
    }

}
