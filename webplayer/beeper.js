export default class Beeper {
  constructor(frequency = 440, wave = "triangle", volume = 0.3) {
    this.frequency = frequency;
    this.wave = wave;
    this.volume = volume;

    this.playing = false;
    this.oscillator = null;
    this.gainNode = null;

    this.audioContext = new (window.AudioContext || window.webkitAudioContext)();
  }

  start() {
    if (this.playing) return;
    this.playing = true;

    this.createOscillator();
    this.createGainNode();

    this.oscillator.connect(this.gainNode);
    this.gainNode.connect(this.audioContext.destination);

    this.oscillator.start();
  }

  stop() {
    if (!this.playing) return;
    this.playing = false;

    const now = this.audioContext.currentTime;

    // workaround for firefox audio click bug
    this.gainNode.gain.setValueAtTime(this.volume, now + 0.012);
    this.gainNode.gain.linearRampToValueAtTime(0.0, now + 0.0149985);

    this.oscillator.stop(now + 0.015);
  }

  createOscillator() {
    this.oscillator = this.audioContext.createOscillator();
    this.oscillator.type = this.wave;
    this.oscillator.frequency.value = this.frequency;
  }

  createGainNode() {
    this.gainNode = this.audioContext.createGain();
    this.gainNode.gain.value = this.volume;
  }
}
