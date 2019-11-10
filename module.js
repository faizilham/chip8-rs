// import wasm resources
// wasm-pkg will be resolved to builds in ./pkg by webpack
import { Machine, ExecutionStatus } from "./pkg";
import { loadROM } from "./webplayer/rom_loader";
import { Display } from "./webplayer/display";
import Beeper from "./webplayer/beeper";

const canvas = document.getElementById("display");


const machine = Machine.new();
const display = new Display(canvas);
let halted = true;
let playing = false;
let animationId = null;

function loop() {
  animationId = null;

  let result = machine.update();

  // update display
  let updates = machine.get_display_update();

  // console.log("cls:", updates.display_cleared, "updated:", updates.display_updated);

  if (updates.display_cleared) {
    display.clearCanvas();
  }

  display.draw(updates.display_ptr, updates.updated_ptr, updates.buffer_size);

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
