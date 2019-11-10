// import wasm resources
// wasm-pkg will be resolved to builds in ./pkg by webpack
import { Machine, ExecutionStatus } from "./pkg";
import { loadROM } from "./webplayer/rom_loader";
import Beeper from "./webplayer/beeper"

const machine = Machine.new();

function loop() {
  let result = machine.update();

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
