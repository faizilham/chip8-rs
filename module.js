// import wasm resources
// wasm-pkg will be resolved to builds in ./pkg by webpack
import { Machine, ExecutionStatus } from "./pkg";
import { memory } from "./pkg/chip8_rs_bg"
import Beeper from "./webplayer/beeper"

function loadROM(machine) {
  const rom = new Uint8Array(memory.buffer, machine.get_rom_ptr(), machine.max_rom_size());

  const program = [
    0x01, 0x2a,
    0x0f, 0x3c,
    0x12, 0x00,
  ];

  for (let i = 0; i < program.length; i++) {
    rom[i] = program[i];
  }

  console.log("ROM loaded");
}

function loop() {
  let result = machine.update_cpu();

  if (result == ExecutionStatus.OK) {
    requestAnimationFrame(loop)
  }
}

const startbtn = document.getElementById("startbtn");
startbtn.onclick = () => requestAnimationFrame(loop);

const machine = Machine.new();
loadROM(machine)
