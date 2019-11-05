const audioContext = new (window.AudioContext || window.webkitAudioContext)();

let beepFrequency = 440;
let beepWave = "triangle";
let beepVolume = 0.3;

let playing = false;
let oscillator = null;
let gainNode = null;

export function InitBeep(frequency = 440, wave = "triangle", volume = 0.3) {
  beepFrequency = frequency;
  beepWave = wave;
  beepVolume = volume;
}

export function StartBeep() {
  if (playing) return;
  playing = true;
  createBeeper();

  oscillator.start();
}

export function StopBeep() {
  if (!playing) return;
  playing = false;

  const now = audioContext.currentTime;

  // workaround for firefox audio click bug
  gainNode.gain.setValueAtTime(beepVolume, now + 0.012);
  gainNode.gain.linearRampToValueAtTime(0.0, now + 0.0149985);

  oscillator.stop(now + 0.015);
}

function createBeeper() {
  oscillator = createOscillator();
  gainNode = createGainNode();

  oscillator.connect(gainNode);
  gainNode.connect(audioContext.destination);
}

function createOscillator() {
  let oscillator = audioContext.createOscillator();
  oscillator.type = beepWave;
  oscillator.frequency.value = beepFrequency;

  return oscillator;
}

function createGainNode() {
  let gainNode = audioContext.createGain();
  gainNode.gain.value = beepVolume;

  return gainNode;
}
