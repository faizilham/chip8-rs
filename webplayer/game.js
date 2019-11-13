import { Machine, ExecutionStatus } from "../pkg";
import { ROMLoader } from "./rom_loader";
import { Display, PhosphorDisplay } from "./display";
import { Keypad } from "./keypad";
import { Beeper } from "./beeper";

export const GameState = Object.freeze({
  HALTED: 0,
  PLAYING: 1,
  PAUSED: 2
});

export class Game {
  constructor(canvas) {
    this.machine = Machine.new();
    this.loader = new ROMLoader(this.machine);
    // this.display = new Display(canvas);
    this.display = new PhosphorDisplay(canvas);
    this.beeper = new Beeper();
    this.keypad = new Keypad();

    this.halted = true;
    this.playing = false;
    this.animationId = null;

    this.stateListeners = [];
  }

  loop() {
    this.animationId = null;

    // update keys
    let [pressed, released] = this.keypad.read_keys();
    this.machine.set_keys(pressed, released);

    // run machine
    const executionResult = this.machine.update();

    // update sound
    this.beeper.setPlaying(this.machine.is_beeping());

    // update display
    const updates = this.machine.get_display_update();

    if (updates.display_cleared) {
      this.display.clearCanvas();
    }

    this.display.draw(updates.display_ptr, updates.updated_ptr, updates.buffer_size);

    // request next frame
    if (executionResult == ExecutionStatus.OK) {
      this.animationId = requestAnimationFrame(() => this.loop());
    } else {
      this.halt();
    }
  }

  loadFile(file) {
    if (!file) {
      console.error("File is empty");
      return;
    }

    return this.loader.loadFile(file)
      .then(() => {
        this.halt();
        this.display.resetCanvas();
      })
      .catch((err) =>{
        console.error(err);
      });
  }

  start() {
    console.log("start", this.playing, this.halted);
    if (this.playing) return;

    if (this.halted) {
      if (!this.loader.reloadROM()) {
        return;
      }

      this.display.resetCanvas();
      this.machine.reset();
    }

    this.playing = true;
    this.halted = false;

    this.animationId = requestAnimationFrame(() => this.loop());

    this.updateListeners();
  }

  pause() {
    if (!this.playing) return;
    this.playing = false;
    this.beeper.stop();
    this.display.finishDraw();

    if (this.animationId) {
      cancelAnimationFrame(this.animationId);
      this.animationId = null;
    }

    this.updateListeners();
  }

  halt() {
    this.halted = true;
    this.playing = false;
    this.beeper.stop();
    this.display.finishDraw();

    if (this.animationId) {
      cancelAnimationFrame(this.animationId);
      this.animationId = null;
    }

    this.updateListeners();
  }

  addListener(updateFunc) {
    this.stateListeners.push(updateFunc);
  }

  updateListeners() {
    let state;

    if (this.halted) {
      state = GameState.HALTED;
    } else if (!this.playing) {
      state = GameState.PAUSED;
    } else {
      state = GameState.PLAYING;
    }

    for (let i = 0; i < this.stateListeners.length; i++) {
      const updateFunc = this.stateListeners[i];
      updateFunc(state);
    }
  }
}
