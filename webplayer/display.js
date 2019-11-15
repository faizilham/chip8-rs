import { memory } from "wasm-pkg/chip8_rs_bg"

const defaultColors = ["#000044", "#808088", "#FFFFCC"];
const defaultOnColor = defaultColors[defaultColors.length - 1];
const defaultOffColor = defaultColors[0];

export class Display {
  constructor(canvas, onColor = defaultOnColor, offColor = defaultOffColor) {
    this.setColor(onColor, offColor);

    this.context = canvas.getContext("2d", {alpha: false});

    this.cols = 64;
    this.rows = 32;
    this.pixelSize = 10;

    this.canvasWidth = this.cols*this.pixelSize;
    this.canvasHeight = this.rows*this.pixelSize;

    canvas.width = this.canvasWidth;
    canvas.height = this.canvasHeight;

    this.clearCanvas();

  }

  setColor(onColor, offColor) {
    this.onColor = onColor;
    this.offColor = offColor;
  }

  draw(pixelPtr, changedPtr, size) {
    const pixels = new Uint8Array(memory.buffer, pixelPtr, size);
    const changed = new Uint8Array(memory.buffer, changedPtr, size);

    let i;
    this.context.beginPath();

    // draw on
    this.context.fillStyle = this.onColor;

    i = 0;
    for (let row = 0; row < this.rows; row++) {
      for (let col = 0; col < this.cols; col++) {
        let idx = i;
        i++;

        if (!changed[idx] || pixels[idx] !== 1) {
          continue;
        }

        this.context.fillRect(
          col * this.pixelSize,
          row * this.pixelSize,
          this.pixelSize,
          this.pixelSize
        );
      }
    }

    // draw off
    this.context.fillStyle = this.offColor;
    i = 0;
    for (let row = 0; row < this.rows; row++) {
      for (let col = 0; col < this.cols; col++) {
        let idx = i;
        i++;

        if (!changed[idx] || pixels[idx] !== 0) {
          continue;
        }

        this.context.fillRect(
          col * this.pixelSize,
          row * this.pixelSize,
          this.pixelSize,
          this.pixelSize
        );
      }
    }

    this.context.closePath();
    this.context.stroke();
  }

  clearCanvas() {
    this.context.beginPath();

    this.context.fillStyle = this.offColor;
    this.context.fillRect(
      0,
      0,
      this.canvasWidth,
      this.canvasHeight
    );

    this.context.closePath();
    this.context.stroke();
  }

  resetCanvas() {
    this.clearCanvas();
  }

  finishDraw(pixelPtr, size) {
    // do nothing
  }
}

export class PhosphorDisplay {
  constructor(canvas, colors = defaultColors, framePersistence = 2) {
    this.context = canvas.getContext("2d", {alpha: false});

    this.cols = 64;
    this.rows = 32;
    this.pixelSize = 10;

    this.canvasWidth = this.cols*this.pixelSize;
    this.canvasHeight = this.rows*this.pixelSize;

    canvas.width = this.canvasWidth;
    canvas.height = this.canvasHeight;

    this.physicalDisplay = new Uint8Array(this.rows * this.cols);

    this.setColor(colors, framePersistence);
  }

  setColor(colors, framePersistence) {
    // framePersistence: how long a level of color persist when the pixel is off

    // levels of colors possible between off (first index) and on (last index)
    this.colors = colors;
    this.colorLevels = [];

    for (let i = 0; i < colors.length; i++) {
      for (let j = 0; j < framePersistence; j++) {
        this.colorLevels.push(i);
      }
    }

    this.maxColor = this.colors.length - 1;
    this.maxColorLevel = this.colorLevels.length - 1;
  }

  draw(pixelPtr, changedPtr, size) {
    let i;
    this.context.beginPath();

    const pixels = new Uint8Array(memory.buffer, pixelPtr, size);
    const changed = new Uint8Array(memory.buffer, changedPtr, size);

    // update values and draw on
    this.context.fillStyle = this.colors[this.maxColor];

    i = 0;
    for (let row = 0; row < this.rows; row++) {
      for (let col = 0; col < this.cols; col++) {
        let idx = i;
        i++;

        if (!changed[idx] || pixels[idx] !== 1) {
          continue;
        }

        this.physicalDisplay[idx] = this.maxColorLevel;
        this.drawPixel(col, row);
      }
    }

    // dim off
    const dimmedPixels = new Array(this.colors.length - 1).fill(null).map(() => []);

    i = 0;
    for (let row = 0; row < this.rows; row++) {
      for (let col = 0; col < this.cols; col++) {
        let idx = i;
        i++;

        if (pixels[idx] === 1) continue; // skip if on

        const prevLevel = this.physicalDisplay[idx];
        if (prevLevel === 0) continue; // skip if fully off

        const currentLevel = prevLevel - 1;
        this.physicalDisplay[idx] = currentLevel;

        if (this.colorLevels[currentLevel] !== this.colorLevels[prevLevel]) {
          let color = this.colorLevels[currentLevel];

          dimmedPixels[color].push(col);
          dimmedPixels[color].push(row);
        }
      }
    }

    // draw dimmed pixels
    for (let color = 0; color < dimmedPixels.length; color++) {
      let updates = dimmedPixels[color];
      this.context.fillStyle = this.colors[color];

      for (let j = 0; j < updates.length; j += 2) {
        const col = updates[j];
        const row = updates[j+1];

        this.drawPixel(col, row);
      }
    }

    this.context.closePath();
    this.context.stroke();
  }

  drawPixel(col, row) {
    this.context.fillRect(
      col * this.pixelSize,
      row * this.pixelSize,
      this.pixelSize,
      this.pixelSize
    );
  }

  clearCanvas() {
    // do nothing
  }

  resetCanvas() {
    for (let i = 0; i < this.physicalDisplay.length; i++) {
      this.physicalDisplay[i] = 0;
    }

    this.context.beginPath();

    this.context.fillStyle = this.colors[0];
    this.context.fillRect(
      0,
      0,
      this.canvasWidth,
      this.canvasHeight
    );

    this.context.closePath();
    this.context.stroke();
  }

  finishDraw() {
    this.context.beginPath();
    this.context.fillStyle = this.colors[0];

    let i = 0;
    for (let row = 0; row < this.rows; row++) {
      for (let col = 0; col < this.cols; col++) {
        let idx = i;
        i++;

        const level = this.physicalDisplay[idx];
        if ((level === 0) || (level === this.maxColorLevel)) continue; // skip if fully off or on

        this.physicalDisplay[idx] = 0;
        this.drawPixel(col, row);
      }
    }

    this.context.closePath();
    this.context.stroke();
  }
}
