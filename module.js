// import wasm resources
import {Game, GameState} from "./webplayer/game";
import roms from "./roms/roms.json";

const canvas = document.getElementById("display");
const startpause = document.getElementById("startpause");
const turnoff = document.getElementById("turnoff");
const openconfig = document.getElementById("configbtn");
const openhelp = document.getElementById("helpbtn");

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
  game.halt();
}

game.addListener((state) => {
  switch(state) {
    case GameState.PLAYING: {
      openconfig.setAttribute("disabled", "true");
      openconfig.setAttribute("title", "Turn off the machine before setting the configuration");
      turnoff.removeAttribute("disabled");
      startpause.textContent = "Pause";
      break;
    }
    case GameState.HALTED: {
      openconfig.removeAttribute("disabled");
      openconfig.setAttribute("title", "Set Configuration");
      startpause.textContent = "Play";
      turnoff.setAttribute("disabled", "true");
      break;
    }

    case GameState.PAUSED:
      startpause.textContent = "Play";
  }
});

/// quirk config
let quirkShiftChk = document.getElementById("quirkShiftChk");
let quirkLoadRegChk = document.getElementById("quirkLoadRegChk");
let quirkClipSpriteChk = document.getElementById("quirkClipSpriteChk");

function setQuirkConfig() {
  const quirks = {
    shift: quirkShiftChk.checked,
    loadStore: quirkLoadRegChk.checked,
    clipSprite: quirkClipSpriteChk.checked,
  };

  game.setConfig({ quirks });
}

quirkShiftChk.checked = false;
quirkLoadRegChk.checked = false;
quirkClipSpriteChk.checked = false;

quirkShiftChk.onchange = setQuirkConfig;
quirkLoadRegChk.onchange = setQuirkConfig;
quirkClipSpriteChk.onchange = setQuirkConfig;

/// rom list
const romdescription = document.getElementById("romdescription");

const fileinput = document.getElementById("fileinput");
let openFileOption;

// init rom lists
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

  romdescription.innerHTML = "";
  openFileOption.textContent = "Open File...";
  startpause.setAttribute("disabled", "true");

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

    quirkShiftChk.checked = !!quirks.shift;
    quirkLoadRegChk.checked = !!quirks.loadStore;
    quirkClipSpriteChk.checked = !!quirks.clipSprite;

    romdescription.innerHTML = `<h4>${rom.title} Description</h4>${rom.description}`

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

/// display configs
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

  return config;
}

colorSchemeSelect.onchange = () => {
  const colorScheme = updateDisplayConfig().colorScheme;

  const root = document.documentElement;

  root.style.setProperty("--back-color", colorScheme[0]);
  root.style.setProperty("--middle-color", colorScheme[1]);
  root.style.setProperty("--front-color", colorScheme[2]);
};

displayType.onchange = updateDisplayConfig;

//// windows
const menu = document.getElementById("menu");
const configwindow = document.getElementById("configwindow");
const helpwindow = document.getElementById("helpwindow");

function toggleWindow(window, show) {
  if (show) {
    window.className = "window";
    canvas.className = "display hidden";
    menu.className = "menu hidden";
  } else {
    window.className = "window hidden";
    canvas.className = "display";
    menu.className = "menu";
  }
}

function toggleConfig(show) {
  toggleWindow(configwindow, show);
}

function toggleHelp(show) {
  toggleWindow(helpwindow, show);
}

toggleConfig(false);
toggleHelp(false);

/// config window buttons

openconfig.onclick = () => toggleConfig(true);

const closeconfig = document.getElementById("closeconfig");
closeconfig.onclick = () => toggleConfig(false);

/// help window buttons

openhelp.onclick = () => {
  if (game.playing) {
    game.pause();
  }

  toggleHelp(true);
}

const closehelp = document.getElementById("closehelp");
closehelp.onclick = () => {
  if (!game.playing && !game.halted) {
    game.start();
  }

  toggleHelp(false);
};
