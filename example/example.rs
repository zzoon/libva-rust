// Copyright (c) 2017 zzoon <zzoon@igalia.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate libva_rust;
extern crate x11;

use std::ptr;
use std::thread::sleep;
use std::time::Duration;
use x11::xlib::{Display, XInitThreads, XOpenDisplay, XCloseDisplay, XCreateSimpleWindow,
                XRootWindow, XBlackPixel, XSync, XMapWindow};

use libva_rust::*;
use libva_rust::va::*;

pub mod test_draw;
pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 240;

fn main() {
    let native_display;
    let win;

    unsafe {
        assert!(XInitThreads() != 0);
        native_display = XOpenDisplay(ptr::null());

        win = XCreateSimpleWindow(native_display,
                                  XRootWindow(native_display, 0),
                                  0,
                                  0,
                                  WIDTH,
                                  HEIGHT,
                                  0,
                                  0,
                                  XBlackPixel(native_display, 0));

        XMapWindow(native_display, win);
        XSync(native_display, 0);
    }

    let va_disp = VADisplay::initialize(native_display as *mut VANativeDisplay).unwrap();
    let va_surface = VASurface::new(&va_disp, WIDTH, HEIGHT, ffi::VA_RT_FORMAT_YUV420, 1).unwrap();
    let va_config = VAConfig::new(&va_disp, ffi::VAProfileMPEG2Main, ffi::VAEntrypointVLD, 1).unwrap();
    let va_context = VAContext::new(&va_disp,
                                    &va_config,
                                    &va_surface,
                                    WIDTH as i32,
                                    HEIGHT as i32,
                                    0).unwrap();

    let va_ffi_format = ffi::VAImageFormat {
        fourcc: ffi::VA_FOURCC_NV12,
        byte_order: ffi::VA_LSB_FIRST,
        bits_per_pixel: 12,
        ..Default::default()
    };

    let va_format = VAImageFormat::new(va_ffi_format);

    let va_image = VAImage::new(&va_disp, &va_format, WIDTH as i32, HEIGHT as i32).unwrap();
    let va_image_buf = va_image.get_buffer();

    test_draw::image_generate(&va_disp, &va_image, &va_image_buf);
    va_image.put_image(&va_disp,
                       &va_surface,
                       0,
                       0,
                       WIDTH,
                       HEIGHT,
                       0,
                       0,
                       WIDTH,
                       HEIGHT);
    va_surface.sync(&va_disp);
    va_image.destroy(&va_disp);

    let wait_dur = Duration::new(1, 0);

    /* FIXME: why doesn't it show up without sleep */
    sleep(wait_dur);
    va_surface.put_surface(&va_disp, win, 0, 0, WIDTH, HEIGHT, 0, 0, WIDTH, HEIGHT);
    va_surface.sync(&va_disp);
    sleep(wait_dur);

    va_context.destroy(&va_disp);
    va_config.destroy(&va_disp);
    va_surface.destroy_surfaces(&va_disp);

    va_disp.destroy();

    unsafe {
        XCloseDisplay(native_display as *mut Display);
    }
}
