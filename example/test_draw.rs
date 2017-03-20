// Copyright (c) 2017 zzoon <zzoon@igalia.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(non_snake_case)]
extern crate libva_rust;

use std::mem;
use std::ptr;
use libva_rust::va::*;

pub const RGB_COLOR: [u32; 4] = [0xffff0000, 0xff00ff00, 0xff0000ff, 0xff000000];
pub const YUV_COLOR: [u32; 4] = [0x4c55ff, 0x952b15, 0x1cff6b, 0x8080];

pub fn argb_to_yuv(value: u32) -> u32 {
    let r = ((value >> 16) & 0xff) as i32;
    let g = ((value >> 8) & 0xff) as i32;
    let b = ((value) & 0xff) as i32;

    let y = (306 * r + 601 * g + 116 * b) >> 10;
    let u = ((-172 * r - 339 * g + 512 * b) >> 10) + 128;
    let v = ((512 * r - 428 * g - 83 * b) >> 10) + 128;

    ((y << 16) | (u << 8) | v) as u32
}

pub fn mem_write(dst: *mut u8, offset: u32, value: u8) {
    unsafe {
        let root: *mut u8 = mem::transmute(mem::transmute::<*mut u8, u64>(dst as *mut u8) +
                                           (mem::size_of::<u8>() * offset as usize) as u64);
        ptr::write(root, value);
    }
}

pub fn mem_ptr(src: *mut u8, offset: u32) -> *mut u8 {
    unsafe {
        mem::transmute(mem::transmute::<*mut u8, u64>(src) +
                      (mem::size_of::<u8>() * offset as usize) as u64)
    }
}

pub fn draw_nv12(pixel1: *mut u64,
                 pixel2: *mut u64,
                 stride1: u32,
                 stride2: u32,
                 mut x: u32,
                 mut y: u32,
                 mut w: u32,
                 mut h: u32,
                 color: u32) {
    let Y = (color >> 16) as u8;
    let Cb = (color >> 8) as u8;
    let Cr = color as u8;

    let mut offset = (y * stride1) + x;
    let mut dst: *mut u8 = mem_ptr(pixel1 as *mut u8, offset);

    for i in 0..h {
        for j in 0..w {
            mem_write(dst, j + i * stride1, Y);
        }
    }

    x /= 2;
    y /= 2;
    w /= 2;
    h /= 2;

    offset = y * stride2 + x * 2;
    dst = mem_ptr(pixel2 as *mut u8, offset);

    for i in 0..h {
        for j in 0..w {
            mem_write(dst, 2 * j + i * stride2, Cb);
            mem_write(dst, 2 * j + i * stride2 + 1, Cr);
        }
    }
}

pub fn image_generate(disp: &VADisplay, image: &VAImage, buffer: &VABuffer) {
    let num_planes = image.get_num_planes();
    let mut stride = [0, 0, 0];
    let mut pixel: [*mut u8; 3] = [ptr::null_mut(), ptr::null_mut(), ptr::null_mut()];

    let my_memptr = buffer.map(&disp);

    for x in 0..num_planes as usize {
        stride[x] = image.get_stride(x);
        pixel[x] = mem_ptr(my_memptr, image.get_offset(x));
    }

    let mut idx = 0;
    for i in 0..2 as u32 {
        for j in 0.. 2 as u32 {
            let yuv = argb_to_yuv(RGB_COLOR[idx]);
            let w:u32 = 160;
            let h:u32 = 120;
            let x:u32 = w * i;
            let y:u32 = h * j;

            draw_nv12(pixel[0] as *mut u64, pixel[1] as *mut u64,
                      stride[0], stride[1], x, y, w, h, yuv);
            idx += 1;
        }
    }

    buffer.unmap(&disp);
}
