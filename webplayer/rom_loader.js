import { memory } from "wasm-pkg/chip8_rs_bg"

export class ROMLoader {
  constructor(machine) {
    this.buffer = null;
    this.max_rom_size = machine.max_rom_size();
    this.machine = machine;
  }

  loadFile(file) {
    if (file.size > this.max_rom_size) {
      return Promise.reject("File too big");
    }

    const reader = new FileReader();

    const promise = new Promise((resolve, reject) => {
      reader.onload = (e) => {
        this.buffer = new Uint8Array(e.target.result);

        this.reloadROM();

        resolve();
      };

      reader.onerror = (e) => {
        reject(e);
      };
    });

    reader.readAsArrayBuffer(file);

    return promise;
  }

  loadBuffer(buffer) {
    this.buffer = buffer;
    this.reloadROM();
  }

  reloadROM() {
    if (!this.buffer) {
      console.log("ROM not yet loaded");
      return false;
    }

    const rom = new Uint8Array(memory.buffer, this.machine.get_rom_ptr(), this.max_rom_size);

    for (let i = 0; i < this.buffer.length; i++) {
      rom[i] = this.buffer[i];
    }

    console.log("ROM Loaded");

    return true;
  }
}
