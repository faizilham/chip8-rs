chip8-rs
========
CHIP-8 implementation in Rust, compiled to wasm. [Live Demo](https://faizilham.github.io/lab/chip8/)

Features
--------
- All CHIP-8 standard features
- Includes 90 ROMs from CHIP-8 pack
- Emulates afterglow in old phosphor screen to reduce flickering (can be turned off)
- Handles 3 implementation quirks that can be enabled:

    1. Shift quirk: opcodes 8xy6 & 8xyE shift Vy instead of Vx
    2. Load/Store register quirk: opcodes Fx55 & Fx65 won't increase I register by x + 1
    3. Sprite clipping quirk: when drawn outside of display, clip sprite instead of wrapping around

- Lightweight page (transfer size < 30kb, total size < 70kb)

How to Build
------------
Install all the tools needed:
- [Rust toolchain](https://rustup.rs) and [wasm-pack](https://github.com/rustwasm/wasm-pack)
- [NodeJS](https://nodejs.org/en/download/) and [yarn](https://yarnpkg.com/lang/en/)
- (Optional) wasm-opt from [binaryen toolchain](https://github.com/WebAssembly/binaryen)


Clone this repo and install Node packages
```
git clone https://github.com/faizilham/chip8-rs.git
cd chip8-rs
yarn install
```

Build project
```
yarn build
```

Build result will be available in `dist/` directory

Keymapping
----------
```
CHIP-8       PC Keyboard
1 2 3 C      1 2 3 4
4 5 6 D      Q W E R
7 8 9 E      A S D F
A 0 B F      Z X C V
```
Included ROMs
------------
All included [ROMs](roms/) have descriptions and default quirk configuration data, taken from
[mir3z's chip8 project](https://github.com/mir3z/chip8-emu) with some modification
