import Beeper from "./webplayer/beeper"


(function() {
  const startbtn = document.getElementById("startbtn");
  const stopbtn = document.getElementById("stopbtn");

  const beeper = new Beeper();

  startbtn.onclick = () => beeper.start();
  stopbtn.onclick = () => beeper.stop();
})()
