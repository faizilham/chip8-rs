#[macro_use]
mod utils;
mod cpu;
mod iodevice;
mod memory;

use wasm_bindgen::prelude::*;
use cpu::ExecutionStatus;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const CPU_TICK_PER_FRAME : u8 = 9;

#[wasm_bindgen]
pub struct Machine {
    cpu: cpu::CPU,
    device: iodevice::IODevice,
}

#[wasm_bindgen]
impl Machine {
    pub fn new() -> Machine {
        utils::set_panic_hook();

        let cpu = cpu::CPU::new();
        let device = iodevice::IODevice::new();

        Machine {
            cpu,
            device,
        }
    }

    /*** Runtime Related ***/

    pub fn set_quirks(&mut self, shift: bool, loadstore: bool) {
        self.cpu.set_quirks(shift, loadstore);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.device.reset();
    }

    pub fn update(&mut self) -> ExecutionStatus {
        self.device.reset_display_flags();

        let status = self.update_cpu();
        self.update_device();

        status
    }

    /*** Memory & Device Related ***/

    pub fn get_rom_ptr(&mut self) -> *mut u8 {
        self.cpu.rom_ptr()
    }

    pub fn max_rom_size(&self) -> usize {
        memory::MEM_SIZE - memory::PROGRAM_START + 1
    }

    pub fn get_display_update(&self) -> iodevice::DisplayUpdate {
        self.device.get_display_update()
    }

    pub fn set_keys(&mut self, pressed_keys: u16, released_keys: u16) {
        self.device.set_keys(pressed_keys, released_keys);
    }

    pub fn is_beeping(&self) -> bool {
        self.cpu.beeping()
    }
}

impl Machine {
    fn update_cpu(&mut self) -> ExecutionStatus {
        let mut status = ExecutionStatus::OK;

        for _ in 0..CPU_TICK_PER_FRAME {
            status = self.cpu.tick(&mut self.device);

            match status {
                ExecutionStatus::OK => (), // continue,
                ExecutionStatus::Halt => {
                    log!("machine halted");
                    break;
                },
                ExecutionStatus::RuntimeError => {
                    log!("machine halted because of runtime error");
                    break;
                }
                ExecutionStatus::WaitForKey => {
                    status = ExecutionStatus::OK;
                    break;
                }
            }
        }

        self.cpu.update_timer();

        status
    }

    fn update_device(&mut self) {

    }
}
