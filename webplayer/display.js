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

// TODO: optimize?
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

    this.physicalDisplay = new Array(this.rows * this.cols).fill(0);

    this.setColor(colors, framePersistence);
  }

  setColor(colors, framePersistence) {
    // how long a level of color persist when the pixel is off
    this.framePersistence = framePersistence;

    // levels of colors possible between off (first index) and on (last index)
    this.colors = colors;

    this.maxColor = this.colors.length - 1;
    this.maxPhysicalColor = this.maxColor * this.framePersistence;
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

        this.physicalDisplay[idx] = this.maxPhysicalColor;
        this.drawPixel(col, row);
      }
    }

    // dim off
    let dimFinished = true;

    const dimmedPixels = new Array(this.colors.length - 1).fill(null).map(() => []);

    i = 0;
    for (let row = 0; row < this.rows; row++) {
      for (let col = 0; col < this.cols; col++) {
        let idx = i;
        i++;

        if (pixels[idx] === 1) continue; // skip if on
        if (this.physicalDisplay[idx] === 0) continue; // skip if fully off

        let currentLevel = this.physicalDisplay[idx] - 1;
        this.physicalDisplay[idx] = currentLevel;

        if (currentLevel % this.framePersistence === 0) {
          let color = currentLevel / this.framePersistence;
          dimmedPixels[color].push([col, row]);
        }

        dimFinished = false;
      }
    }

    // draw dimmed pixels
    for (let color = 0; color < dimmedPixels.length; color++) {
      let updates = dimmedPixels[color];
      this.context.fillStyle = this.colors[color];

      for (let j = 0; j < updates.length; j++) {
        let [col, row] = updates[j];
        this.drawPixel(col, row);
      }
    }

    this.context.closePath();
    this.context.stroke();

    return dimFinished;
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

        if (this.physicalDisplay[idx] === this.maxPhysicalColor) continue; // skip if fully on
        if (this.physicalDisplay[idx] === 0) continue; // skip if fully off

        this.physicalDisplay[idx] = 0;
        this.drawPixel(col, row);
      }
    }

    this.context.closePath();
    this.context.stroke();
  }
}
