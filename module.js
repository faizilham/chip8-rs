// import wasm resources
// wasm-pkg will be resolved to builds in ./pkg by webpack
import { Machine, ExecutionStatus } from "./pkg";
import { loadROM } from "./webplayer/rom_loader";
import { Display } from "./webplayer/display";
import { Keypad } from "./webplayer/keypad";
import Beeper from "./webplayer/beeper";

const machine = Machine.new();
const canvas = document.getElementById("display");
const display = new Display(canvas);
const keypad = new Keypad();

let halted = true;
let playing = false;
let animationId = null;

function loop() {
  animationId = null;

  let result = machine.update();

  // update display
  let updates = machine.get_display_update();

  if (updates.display_cleared) {
    display.clearCanvas();
  }

  display.draw(updates.display_ptr, updates.updated_ptr, updates.buffer_size);

  // update keys
  let [pressed, released] = keypad.read_keys();
  machine.set_keys(pressed, released);

  if (result == ExecutionStatus.OK) {
    animationId = requestAnimationFrame(loop);
  } else {
    halt();
  }
}

const startpause = document.getElementById("startpause");

startpause.onclick = () => {
  if (!playing) {
    start();
  } else {
    pause();
  }
}

function start() {
  if (playing) return;

  if (halted) {
    display.clearCanvas();
    machine.reset();
  }

  playing = true;
  halted = false;

  animationId = requestAnimationFrame(loop);
  startpause.textContent = "Pause";
}

function pause() {
  if (!playing) return;
  playing = false;

  if (animationId) {
    cancelAnimationFrame(animationId);
    animationId = null;
  }

  startpause.textContent = "Start";
}

function halt() {
  halted = true;
  playing = false;

  if (animationId) {
    cancelAnimationFrame(animationId);
    animationId = null;
  }

  startpause.textContent = "Start";
}


const loadbtn = document.getElementById("loadbtn");
const fileinput = document.getElementById("fileinput");

loadbtn.onclick = () => {
  const file = fileinput.files[0];

  if (!file) return;

  loadROM(machine, file)
    .then(() => {
      halt();
      display.clearCanvas();
      console.log("ROM Loaded");
    })
    .catch((err) =>{
      console.error(err);
    })
}

// loadROM(machine)
