// import wasm resources
import {Game, GameState} from "./webplayer/game";
import roms from "./roms/roms.json";

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

/// init rom list
const fileinput = document.getElementById("fileinput");
let openFileOption;

const romselect = document.getElementById("romselect");
(function (){

  function addOption(value, text, id) {
    let option = document.createElement("option");
    option.value = value;
    option.textContent = text;

    romselect.appendChild(option);
    return option;
  }

  addOption("none", "Select ROM...");
  openFileOption = addOption("file", "Open File...");

  openFileOption.onclick = () => fileinput.click();

  for (let i = 0; i < roms.length; i++) {
    addOption(i, roms[i].title);
  }

  romselect.value = "none";
})();

romselect.onchange = (e) => {
  let idx = e.target.value;

  openFileOption.textContent = "Open File...";

  if (idx === "none") return;
  if (idx === "file") {
    fileinput.click();
    return;
  }

  const rom = roms[idx];

  const req = new Request("roms/" + rom.file);

  fetch(req).then((resp) => resp.arrayBuffer()).then((arraybuffer) => {
    const buffer = new Uint8Array(arraybuffer);
    game.loadBuffer(buffer);

    const quirks = rom.quirks || {};
    game.setConfig({ quirks });

    startpause.removeAttribute("disabled");
  });
};

fileinput.onchange = (e) => {
  const file = fileinput.files[0];
  if (!file) return;

  let filename = file.name;

  if (filename.length > 18) {
    filename = filename.substring(0, 15) + "...";
  }

  openFileOption.textContent = "File: " + filename;

  game.loadFile(file).then(() => {
    startpause.removeAttribute("disabled");
  });
}



/// config window

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


const colorSchemeSelect = document.getElementById("colorscheme");
colorSchemeSelect.value = "yellow-blue";

const colorSchemes = {
  "yellow-blue": ["#000044", "#808088", "#FFFFCC"],
  "green-black": ["#222222", "#77912B", "#CCFF33"],
  "white-black": ["#222222", "#919191", "#FFFFFF"],
};

const displayType = document.getElementById("displaytype");
displayType.value = "phosphor";

function updateDisplayConfig() {
  let config = {};

  let scheme = colorSchemeSelect.value;
  config.colorScheme = colorSchemes[scheme] || colorSchemes["yellow-blue"];

  config.displayType = displayType.value;

  game.setConfig(config);
}

colorSchemeSelect.onchange = updateDisplayConfig;
displayType.onchange = updateDisplayConfig;

const closeconfig = document.getElementById("closeconfig");
closeconfig.onclick = () => toggleConfig(false);
