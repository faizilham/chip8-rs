extern crate web_sys;
extern crate js_sys;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme

    // set panic hook only on development & test
    #[cfg(all(feature = "console_error_panic_hook", debug_assertions))]
    console_error_panic_hook::set_once();
}

#[cfg(not(test))]
macro_rules! log {
    ($( $t:tt )*) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    };
}

// mock log for test
#[cfg(test)]
macro_rules! log {
    ($( $t:tt )*) => {};
}

#[allow(unused_macros)]
#[cfg(debug_assertions)]
macro_rules! debug {
    ($( $t:tt )*) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    };
}

// disable debug for release
#[cfg(test)]
#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($( $t:tt )*) => {};
}

#[cfg(not(test))]
pub fn random() -> u8 {
    (js_sys::Math::random() * 255.0).floor() as u8
}

// mock random for test
#[cfg(test)]
pub fn random() -> u8 {
    0xBD
}
