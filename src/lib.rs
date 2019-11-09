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
    memory: memory::Memory,
    cpu: cpu::CPU,

    // init_rom(size) -> *rom
    // update_cpu() -> exec_status
    // set_keys(list of keys)
}

#[wasm_bindgen]
impl Machine {
    pub fn new() -> Machine {
        utils::set_panic_hook();

        let memory = memory::allocate_memory();
        let cpu = cpu::CPU::new();

        Machine {
            memory,
            cpu,
        }
    }

    pub fn update_cpu(&mut self) -> ExecutionStatus {
        for _ in 0..CPU_TICK_PER_FRAME {
            let status = self.cpu.tick(&mut self.memory);

            if status != ExecutionStatus::OK {
                log!("machine halted");
                return status;
            }
        }

        ExecutionStatus::OK
    }

    pub fn rom_ptr(&mut self) -> *mut u8 {
        let ptr = unsafe {
            self.memory.as_mut_ptr().offset(memory::PROGRAM_START as isize)
        };

        ptr
    }

    pub fn max_rom_size(&self) -> usize {
        memory::MEM_SIZE - memory::PROGRAM_START + 1
    }
}


#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_get_program_memory() {
        let mut machine = Machine::new();

        let expected_value = 0x2A;

        machine.memory[memory::PROGRAM_START] = expected_value;

        let ptr = machine.rom_ptr();
        let ptr_value = unsafe { *ptr };

        assert_eq!(expected_value, ptr_value);
    }
}
