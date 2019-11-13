// import wasm resources
import {Game, GameState} from "./webplayer/game";

const canvas = document.getElementById("display");
const startpause = document.getElementById("startpause");
const turnoff = document.getElementById("turnoff");
const openconfig = document.getElementById("configbtn");

const game = new Game(canvas);

startpause.setAttribute("disabled", "true");
turnoff.setAttribute("disabled", "true");

startpause.onclick = () => {
  if (!game.playing) {
    game.start();
  } else {
    game.pause();
  }
}

turnoff.onclick = () => {
  game.halt(true);
}

game.addListener((state) => {
  switch(state) {
    case GameState.PLAYING: {
      openconfig.setAttribute("disabled", "true");
      turnoff.removeAttribute("disabled");
      startpause.textContent = "Pause";
      break;
    }
    case GameState.HALTED: {
      openconfig.removeAttribute("disabled");
      startpause.textContent = "Start";
      turnoff.setAttribute("disabled", "true");
      break;
    }

    case GameState.PAUSED:
      startpause.textContent = "Start";
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

const configwindow = document.getElementById("configwindow");
const menu = document.getElementById("menu");

let configOpen = false;

toggleConfig(false);

function toggleConfig(show) {
  if (show) {
    configOpen = true;
    configwindow.className = "config";
    canvas.className = "display hidden";
    menu.className = "menu hidden";
  } else {
    configOpen = false;
    configwindow.className = "config hidden";
    canvas.className = "display";
    menu.className = "menu";
  }
}

openconfig.onclick = () => {
  toggleConfig(true);
}

/// config window

const colorSchemeSelect = document.getElementById("colorscheme");
colorSchemeSelect.value = "yellow-blue";

const colorSchemes = {
  "yellow-blue": ["#000044", "#808088", "#FFFFCC"],
  "green-black": ["#222222", "#77912B", "#CCFF33"],
  "white-black": ["#222222", "#919191", "#FFFFFF"],
};

colorSchemeSelect.onchange = (e) => {
  let scheme = e.target.value;
  let config = {};
  config.colorScheme = colorSchemes[scheme] || colorSchemes["yellow-blue"];
  game.setConfig(config);
}

const displayType = document.getElementById("displaytype");
displayType.value = "phosphor";

displayType.onchange = (e) => {
  let config = {
    displayType: e.target.value
  };

  game.setConfig(config);
}

const closeconfig = document.getElementById("closeconfig");
closeconfig.onclick = () => toggleConfig(false);
