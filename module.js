// import wasm resources
// wasm-pkg will be resolved to builds in ./pkg by webpack
import { Machine, ExecutionStatus } from "./pkg";
import { memory } from "./pkg/chip8_rs_bg"
import Beeper from "./webplayer/beeper"

function loadROM(machine) {
  const rom = new Uint8Array(memory.buffer, machine.get_rom_ptr(), machine.max_rom_size());

  const program = [
    0x60, 0x05,
    0x00, 0xe0,
    0x70, 0xff,
    0x30, 0x00,
    0x12, 0x02,
    0xd0, 0x00
  ];

  /// add trap jump halt at the end
  let last = program.length;
  program.push(0x12);
  program.push(last);


  for (let i = 0; i < program.length; i++) {
    rom[i] = program[i];
  }

  console.log("ROM loaded");
}

function loop() {
  let result = machine.update_cpu();

  if (result == ExecutionStatus.OK) {
    requestAnimationFrame(loop);
  }
}

function start() {
  machine.reset();
  requestAnimationFrame(loop);
}

const startbtn = document.getElementById("startbtn");
startbtn.onclick = start;

const machine = Machine.new();
loadROM(machine)
