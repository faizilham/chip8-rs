// import wasm resources
// wasm-pkg will be resolved to builds in ./pkg by webpack
import * as wasm from "./pkg";
import Beeper from "./webplayer/beeper"

wasm.greet();



const startbtn = document.getElementById("startbtn");
const stopbtn = document.getElementById("stopbtn");

const beeper = new Beeper();

startbtn.onclick = () => beeper.start();
stopbtn.onclick = () => beeper.stop();
