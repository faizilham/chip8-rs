import {InitBeep, StartBeep, StopBeep} from "./webplayer/beeper"


(function() {
  const startbtn = document.getElementById("startbtn");
  const stopbtn = document.getElementById("stopbtn");

  InitBeep();

  startbtn.onclick = StartBeep;
  stopbtn.onclick = StopBeep;
})()
