import { memory } from "wasm-pkg/chip8_rs_bg"

export class Display {
  constructor(canvas, onColor = "#ffffff", offColor = "#000000") {
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
    // this.context.beginPath();

    // draw on
    this.context.fillStyle = self.onColor;

    i = 0;
    for (let row = 0; row < this.rows; row++) {
      for (let col = 0; col < this.cols; col++) {
        let idx = i;
        i++;

        if (!changed[idx] || pixels[idx] !== 1) {
          continue;
        }

        console.log("on", col, row, pixels[idx], changed[idx]);


        this.drawPixel(col, row);
      }
    }

    // draw off
    this.context.fillStyle = self.offColor;
    i = 0;
    for (let row = 0; row < this.rows; row++) {
      for (let col = 0; col < this.cols; col++) {
        let idx = i;
        i++;

        if (!changed[idx] || pixels[idx] !== 0) {
          continue;
        }

        console.log("off", col, row, pixels[idx], changed[idx]);
        // console.log("off", col, row);

        this.drawPixel(col, row);

        // this.context.fillRect(
        //   col * this.pixelSize,
        //   row * this.pixelSize,
        //   this.pixelSize,
        //   this.pixelSize
        // );
      }
    }

    // this.context.closePath();
    // this.context.stroke();
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
    console.log("cls");
    // this.context.beginPath();


    // this.drawPixel(1, 1);
    // this.drawPixel(2, 2);
    // this.drawPixel(46, 23);

    // this.context.fillStyle = this.onColor;
    // this.drawPixel(-10, 0);

    this.context.fillStyle = this.offColor;
    this.context.fillRect(
      0,
      0,
      this.canvasWidth,
      this.canvasHeight
    );

    this.context.fillStyle = this.onColor;
    this.drawPixel(-1, -1);

    // this.context.closePath();
    // this.context.stroke();
  }
}
