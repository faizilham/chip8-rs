// import wasm resources
import {Game, GameState} from "./webplayer/game";

const canvas = document.getElementById("display");
const startpause = document.getElementById("startpause");

const game = new Game(canvas);

startpause.onclick = () => {
  if (!game.playing) {
    game.start();
  } else {
    game.pause();
  }
}

game.addListener((state) => {
  switch(state) {
    case GameState.PLAYING: {
      startpause.textContent = "Pause";
      break;
    }
    default: {
      startpause.textContent = "Start";
    }
  }
});

const loadbtn = document.getElementById("loadbtn");
const fileinput = document.getElementById("fileinput");

loadbtn.onclick = () => {
  const file = fileinput.files[0];

  if (!file) return;

  game.loadFile(file).then(() => {
    startpause.removeAttribute("disabled");
  });
}
