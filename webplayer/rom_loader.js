import { memory } from "wasm-pkg/chip8_rs_bg"


export function loadROM(machine, file) {
  const max_rom_size = machine.max_rom_size();

  if (file.size > max_rom_size) {
    return Promise.reject("File too big");
  }

  const reader = new FileReader();

  const promise = new Promise((resolve, reject) => {
    reader.onload = (e) => {
      const buffer = new Uint8Array(e.target.result);
      const rom = new Uint8Array(memory.buffer, machine.get_rom_ptr(), max_rom_size);

      for (let i = 0; i < buffer.length; i++) {
        rom[i] = buffer[i];
      }

      resolve();
    };

    reader.onerror = (e) => {
      reject(e);
    };
  });

  reader.readAsArrayBuffer(file);

  return promise;
}
