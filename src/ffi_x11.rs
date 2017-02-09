use ffi;

pub type XID = ::std::os::raw::c_ulong;
pub type Drawable = XID;

#[link(name = "va-x11")]
extern "C" {
    pub fn vaGetDisplay(dpy: *mut ffi::VANativeDisplay) -> ffi::VADisplay;
}
extern "C" {
    pub fn vaPutSurface(dpy: ffi::VADisplay, surface: ffi::VASurfaceID, draw: Drawable,
                        srcx: ::std::os::raw::c_short,
                        srcy: ::std::os::raw::c_short,
                        srcw: ::std::os::raw::c_ushort,
                        srch: ::std::os::raw::c_ushort,
                        destx: ::std::os::raw::c_short,
                        desty: ::std::os::raw::c_short,
                        destw: ::std::os::raw::c_ushort,
                        desth: ::std::os::raw::c_ushort,
                        cliprects: *mut ffi::VARectangle,
                        number_cliprects: ::std::os::raw::c_uint,
                        flags: ::std::os::raw::c_uint) -> ffi::VAStatus;
}

