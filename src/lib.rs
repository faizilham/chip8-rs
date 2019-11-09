#[macro_use]
mod utils;
mod cpu;
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

    // init_rom(size) -> *rom
    // update_cpu() -> exec_status
    // set_keys(list of keys)
}

#[wasm_bindgen]
impl Machine {
    pub fn new() -> Machine {
        utils::set_panic_hook();

        let cpu = cpu::CPU::new();

        Machine {
            cpu,
        }
    }

    pub fn update_cpu(&mut self) -> ExecutionStatus {
        for _ in 0..CPU_TICK_PER_FRAME {
            let status = self.cpu.tick();

            if status != ExecutionStatus::OK {
                log!("machine halted");
                return status;
            }
        }

        ExecutionStatus::OK
    }

    pub fn get_rom_ptr(&mut self) -> *mut u8 {
        self.cpu.rom_ptr()
    }

    pub fn max_rom_size(&self) -> usize {
        memory::MEM_SIZE - memory::PROGRAM_START + 1
    }
}


#[cfg(test)]
mod test {

}
