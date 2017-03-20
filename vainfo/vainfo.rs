// Copyright (c) 2017 zzoon <zzoon@igalia.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate libva_rust;
extern crate x11;

use std::env;
use std::ptr;
use x11::xlib::{Display, XInitThreads, XOpenDisplay, XCloseDisplay, XSync};

use libva_rust::va::*;

fn main() {
    let native_display;

    unsafe {
        assert!(XInitThreads() != 0);
        native_display = XOpenDisplay(ptr::null());

        XSync(native_display, 0);
    }

    let va_disp = VADisplay::initialize(native_display as *mut VANativeDisplay).unwrap();
    let (maj, min) = va_disp.get_va_version();
    let name = env::args().nth(0).unwrap();

    println!("{}: VA-API version: {}.{}", name, maj, min);
    println!("{}: Driver version: {}", name, va_disp.get_vendor_string());
    println!("{}: Supported profile and entrypoints", name);

    let profiles = va_disp.get_profiles();

    for profile in profiles.iter().cloned() {
        let entries = va_disp.get_entrypoints(profile);
        for entry in entries.iter() {
            println!("      {:?}:\t\t{:?}", profile, entry);
        }
    }

    va_disp.destroy();
    unsafe {
        XCloseDisplay(native_display as *mut Display);
    }
}
