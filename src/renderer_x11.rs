use std::ptr;
use std::mem;

use va::*;
use renderer::*;
use x11::xlib::*;
use ffi;

#[derive(Debug)]
pub struct VARendererX11 {
    disp: VADisplay,
    win: Window,
    width: u32,
    height: u32,
}

unsafe impl Send for VARendererX11 {}

impl VARendererX11 {
    pub fn new(disp: VADisplay, width: u32, height: u32) -> Result<Box<VARenderer>, ()> {
        Ok(Box::new(VARendererX11 {
            disp: disp,
            win: 0,
            width: width,
            height: height,
        }))
    }
}

fn mem_write(dst: *mut u8, offset: u32, value: u8) {
    unsafe {
        let root: *mut u8 = mem::transmute(
            mem::transmute::<*mut u8, u64>(dst as *mut u8) +
                (mem::size_of::<u8>() * offset as usize) as u64,
        );
        ptr::write(root, value);
    }
}

fn mem_ptr(src: *mut u8, offset: u32) -> *mut u8 {
    unsafe {
        mem::transmute(
            mem::transmute::<*mut u8, u64>(src) + (mem::size_of::<u8>() * offset as usize) as u64,
        )
    }
}


impl VARenderer for VARendererX11 {
    fn open(&mut self) -> Option<u8> {
        let native_display = self.disp.get_native_display();
        unsafe {
            self.win = XCreateSimpleWindow(
                native_display as *mut Display,
                XRootWindow(native_display as *mut Display, 0),
                0,
                0,
                self.width,
                self.height,
                0,
                0,
                XBlackPixel(native_display as *mut Display, 0),
            );
            XMapWindow(native_display as *mut Display, self.win);
            XSync(native_display as *mut Display, 0);
            XInitThreads();
        }


        None
    }

    fn close(&mut self) -> Option<u8> {
        None
    }

    fn render(&self, data: &[u8], len: usize) -> Option<u8> {
        let va_ffi_format = ffi::VAImageFormat {
            fourcc: ffi::VA_FOURCC_NV12,
            byte_order: ffi::VA_LSB_FIRST,
            bits_per_pixel: 12,
            ..Default::default()
        };

        let va_surface = VASurface::new(
            &self.disp,
            self.width,
            self.height,
            ffi::VA_RT_FORMAT_YUV420,
            1,
        ).unwrap();

        let va_format = VAImageFormat::new(va_ffi_format);

        let va_image = VAImage::new(
            &self.disp,
            &va_format,
            self.width as i32,
            self.height as i32,
        ).unwrap();
        let va_image_buf = va_image.get_buffer();

        let mut image_buf_ptr = va_image_buf.map(&self.disp);

        let num_planes = va_image.get_num_planes();
        let mut stride = [0, 0, 0];
        let mut pixel: [*mut u8; 3] = [ptr::null_mut(), ptr::null_mut(), ptr::null_mut()];

        for x in 0..num_planes as usize {
            stride[x] = va_image.get_stride(x);
            pixel[x] = mem_ptr(image_buf_ptr, va_image.get_offset(x));
        }

        let mut idx: usize = 0;
        let mut h = self.height;

        #[allow(unused_variables)]
        for k in 0..num_planes {
            image_buf_ptr = pixel[k as usize];
            for i in 0..h {
                for j in 0..self.width {
                    if idx >= len {
                        break;
                    }
                    mem_write(image_buf_ptr, j as u32, data[idx]);
                    idx += 1;
                }
                image_buf_ptr = mem_ptr(image_buf_ptr, stride[k as usize]);
            }
            h /= 2;
        }

        va_image_buf.unmap(&self.disp);

        va_image.put_image(
            &self.disp,
            &va_surface,
            0,
            0,
            self.width,
            self.height,
            0,
            0,
            self.width,
            self.height,
        );
        va_image.destroy(&self.disp);

        va_surface.put_surface(
            &self.disp,
            self.win,
            0,
            0,
            self.width,
            self.height,
            0,
            0,
            self.width,
            self.height,
        );
        va_surface.destroy_surfaces(&self.disp);

        None
    }
}
