mod app;
mod browser;
mod ptr;

use std::{
    env::args,
    ffi::{c_char, c_int},
};

pub use app::{App, AppSettings};
pub use browser::{
    bridge::BridgeObserver,
    control::{
        ActionState, ImeAction, Modifiers, MouseAction, MouseButtons, TouchEventType,
        TouchPointerType,
    },
    Browser, BrowserSettings, BrowserState, Observer,
};
use ptr::AsCStr;

extern "C" {
    fn execute_sub_process(argc: c_int, argv: *const *const c_char);
}

pub fn execute_subprocess() {
    if tokio::runtime::Handle::try_current().is_ok() {
        panic!("cef sub process does not work in tokio runtime!");
    }

    let args = std::env::args()
        .map(|arg| arg.as_c_str())
        .collect::<Vec<_>>();
    let args = args
        .iter()
        .map(|arg| arg.ptr)
        .collect::<Vec<*const c_char>>();
    unsafe { execute_sub_process(args.len() as c_int, args.as_ptr()) }
}

pub fn is_sub_process() -> bool {
    args().find(|v| v.contains("--type")).is_some()
}
