extern crate web_sys;

#[cfg(not(test))]
macro_rules! log {
    ($( $t:tt )*) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    };
}

#[cfg(test)]
macro_rules! log {
    ($( $t:tt )*) => {};
}

#[allow(unused_macros)]
#[cfg(any(test, debug))]
macro_rules! debug {
    ($( $t:tt )*) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    };
}

#[allow(unused_macros)]
#[cfg(not(any(test, debug)))]
macro_rules! debug {
    ($( $t:tt )*) => {};
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
