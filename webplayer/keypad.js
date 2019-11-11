const defaultKeyMapping = ["X", "1", "2", "3", "Q", "W", "E", "A", "S", "D", "Z", "C", "4", "R", "F", "V"];
/*** Default Key Mapping:
 *
 *   chip8    ->  pc keyboard
 *   1 2 3 C      1 2 3 4
 *   4 5 6 D      Q W E R
 *   7 8 9 E      A S D F
 *   A 0 B F      Z X C V
 */

export class Keypad {
  constructor(keyMapping = defaultKeyMapping) {
    window.onkeydown = (e) => this.keydown(e);
    window.onkeyup = (e) => this.keyup(e);

    const mapFromPC = {};

    for (let i = 0; i < 16; i++) {
      const key = keyMapping[i];
      mapFromPC[key] = i;
    }

    this.mapping = mapFromPC;
    this.pressed = new Array(16).fill(false);
    this.released = new Array(16).fill(false);
  }

  read_keys() {
    // read pressed
    let pressed_update = 0;

    for (let key = 0; key < 16; key++) {
      if (this.pressed[key]) {
        pressed_update = write_keyupdate(pressed_update, key);
      }
    }

    // read released and clear
    let released_update = 0;

    for (let key = 0; key < 16; key++) {
      if (this.released[key]) {
        released_update = write_keyupdate(released_update, key);
        this.released[key] = false;
      }
    }

    return [pressed_update, released_update];
  }

  keydown(e) {
    let pcKey = e.key.toUpperCase();
    let key = this.mapping[pcKey];

    if (key == null) return;
    this.pressed[key] = true;
    this.released[key] = false;
  }

  keyup(e) {
    let pcKey = e.key.toUpperCase();
    let key = this.mapping[pcKey];

    if (key == null) return;
    this.pressed[key] = false;
    this.released[key] = true;
  }
}

function write_keyupdate(update, key) {
  return update | 1 << key;
}
