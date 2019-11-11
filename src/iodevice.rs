use wasm_bindgen::prelude::*;

pub const DISPLAY_WIDTH : usize = 64;
pub const DISPLAY_HEIGHT : usize = 32;
pub const DISPLAY_SIZE : usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

pub const NO_KEY : u8 = 0xFF;

pub trait IOInterface {
    fn clear_display(&mut self);
    fn draw_pixel(&mut self, x: u8, y: u8) -> u8;

    // key_pressed returns true if `key` is in pressed state
    fn key_pressed(&self, key: u8) -> bool;

    // read_any_key returns key code when a keypad is released. otherwise, it returns NO_KEY
    fn read_any_key(&mut self) -> u8;
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct DisplayUpdate {
    pub display_ptr: *const u8,
    pub updated_ptr: *const bool,
    pub buffer_size: usize,
    pub display_updated: bool,
    pub display_cleared: bool,
}

pub struct IODevice {
    display_buffer: [u8; DISPLAY_SIZE],
    updated: [bool; DISPLAY_SIZE],

    pressed_keys: u16,      // mapping u16 of which key is currently pressed
    released_keys: u16,     // mapping u16 of which key has just been released

    display_cleared: bool,
    display_updated: bool,
}

impl IODevice {
    pub fn new() -> IODevice {
        let display_buffer = [0; DISPLAY_SIZE];
        let updated = [false; DISPLAY_SIZE];

        IODevice {
            display_buffer,
            updated,
            pressed_keys: 0,
            released_keys: 0,
            display_cleared: false,
            display_updated: false,
        }
    }

    pub fn reset(&mut self) {
        self.clear_display();
        self.pressed_keys = 0;
        self.released_keys = 0;
        self.display_cleared = false;
        self.display_updated = false;
    }

    pub fn reset_display_flags(&mut self) {
        if !self.display_updated {
            return;
        }

        for i in 0..DISPLAY_SIZE {
            self.updated[i] = false;
        }

        self.display_cleared = false;
        self.display_updated = false;
    }

    pub fn get_display_update(&self) -> DisplayUpdate {
        DisplayUpdate {
            display_ptr: self.display_buffer.as_ptr(),
            updated_ptr: self.updated.as_ptr(),
            buffer_size: DISPLAY_SIZE,
            display_cleared: self.display_cleared,
            display_updated: self.display_updated,
        }
    }

    pub fn set_keys(&mut self, pressed_keys: u16, released_keys: u16) {
        self.pressed_keys = pressed_keys;
        self.released_keys = released_keys;
    }
}

impl IOInterface for IODevice {
    fn clear_display(&mut self) {
        for i in 0..DISPLAY_SIZE {
            self.display_buffer[i] = 0;
            self.updated[i] = false;
        }

        self.display_cleared = true;
        self.display_updated = true;
    }

    fn draw_pixel(&mut self, x: u8, y: u8) -> u8 {
        let i = to_index(x as usize, y as usize);

        let color = self.display_buffer[i] ^ 1;
        self.display_buffer[i] = color;
        self.updated[i] = true;
        self.display_updated = true;

        !color & 1
    }

    fn key_pressed(&self, key: u8) -> bool {
        check_key(self.pressed_keys, key)
    }

    fn read_any_key(&mut self) -> u8 {

        // check if any key is released
        if self.released_keys == 0 {
            return NO_KEY;
        }

        for key in 0..16 {
            if check_key(self.released_keys, key) {
                // clear released after read
                self.released_keys = 0;
                return key;
            }
        }

        NO_KEY
    }
}

#[inline]
fn to_index(x: usize, y: usize) -> usize {
    (y % DISPLAY_HEIGHT) * DISPLAY_WIDTH + (x % DISPLAY_WIDTH)
}

#[inline]
fn check_key(keys: u16, key: u8) -> bool {
    keys & (1 << key) > 0
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::*;
    use super::*;

    #[wasm_bindgen_test]
    fn test_to_index() {
        let minus_one = std::usize::MAX;
        let max_x = DISPLAY_WIDTH - 1;
        let max_y = DISPLAY_HEIGHT - 1;

        // normal case
        let i = 5 * DISPLAY_WIDTH + 3;
        assert_eq!(i, to_index(3, 5));

        // wrap-around cases
        assert_eq!(to_index(0, 5), to_index(max_x + 1, 5)); // right
        assert_eq!(to_index(max_x, 5), to_index(minus_one, 5)); // left
        assert_eq!(to_index(5, max_y), to_index(5, minus_one)); // top
        assert_eq!(to_index(5, 0), to_index(5, max_y + 1)); // top
    }

    #[wasm_bindgen_test]
    fn test_draw_pixel() {
        let mut device = IODevice::new();

        let x = 17; let y = 26;
        let i = to_index(x as usize, y as usize);

        // normal draw case
        let result = device.draw_pixel(x, y);

        assert_eq!(device.display_buffer[i], 1);
        assert_eq!(result, 0);

        // erasure case
        let result = device.draw_pixel(x, y);

        assert_eq!(device.display_buffer[i], 0);
        assert_eq!(result, 1);
    }
}
