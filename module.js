// import wasm resources
// wasm-pkg will be resolved to builds in ./pkg by webpack
import { Machine, ExecutionStatus } from "./pkg";
import { loadROM } from "./webplayer/rom_loader";
import { Display } from "./webplayer/display";
import Beeper from "./webplayer/beeper";

const canvas = document.getElementById("display");


const machine = Machine.new();
const display = new Display(canvas);


function loop() {
  let result = machine.update();

  // update display
  let updates = machine.get_display_update();

  console.log("cls:", updates.display_cleared, "updated:", updates.display_updated);

  if (updates.display_cleared) {
    display.clearCanvas();
  }

  display.draw(updates.display_ptr, updates.updated_ptr, updates.buffer_size);

  if (result == ExecutionStatus.OK) {
    requestAnimationFrame(loop);
  }
}

function start() {
  display.clearCanvas();
  machine.reset();
  requestAnimationFrame(loop);
}


const startbtn = document.getElementById("startbtn");
startbtn.onclick = start;

const loadbtn = document.getElementById("loadbtn");
const fileinput = document.getElementById("fileinput");

loadbtn.onclick = () => {
  const file = fileinput.files[0];

  if (!file) return;

  loadROM(machine, file)
    .then(() => {
      console.log("ROM Loaded");
    })
    .catch((err) =>{
      console.error(err);
    })
}

// loadROM(machine)
