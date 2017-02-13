/* Temporarily disable dead_code warning */
#![allow(dead_code)]

use std::os::raw::{c_uint, c_int, c_void, c_short, c_ushort};
use std::ptr;
use std::ffi::CString;

use ffi;
use ffi_x11;

pub type VANativeDisplay = ffi::VANativeDisplay;
pub type VANativeDrawable = ffi_x11::Drawable;
pub type VAConfigAttribType = ffi::VAConfigAttribType;
pub type VARectangle = ffi::VARectangle;
pub type VAProfile = ffi::VAProfile;
pub type VAEntrypoint = ffi::VAEntrypoint;
pub type VABufferType = ffi::VABufferType;

pub const VA_STATUS_SUCCESS: i32 = ffi::VA_STATUS_SUCCESS as i32;

pub struct VADisplay {
    disp: ffi::VADisplay,
    min: c_int,
    maj: c_int,
    max_profiles: c_int,
    vendor_string: String,
    profile_list: Vec<VAProfile>,
}

/* TODO: replace print message with debug message or ruturning new value */
impl VADisplay {
    pub fn initialize(native_disp: *mut VANativeDisplay) -> Result<VADisplay, ()> {
        let mut min = 0;
        let mut maj = 0;
        let disp = va_get_display(native_disp);

        match va_init(disp, &mut maj, &mut min) {
            VA_STATUS_SUCCESS => {
                println!("Initialization sucess - Version: {}.{}", maj, min);
            }
            _ => return Err(()),
        }

        let max_profiles = va_max_num_profiles(disp);
        let vendor_string = va_query_vendor_string(disp);
        let profile_list = va_query_config_profiles(disp, max_profiles);

        Ok(VADisplay {
            disp: disp,
            min: min,
            maj: maj,
            max_profiles: max_profiles,
            vendor_string: vendor_string,
            profile_list: profile_list,
        })
    }

    pub fn get_profiles(&self) -> &Vec<VAProfile> {
        &self.profile_list
    }

    pub fn get_entrypoints(&self, profile: VAProfile) -> Vec<VAEntrypoint> {
        let max_entrypoints = va_max_num_entrypoints(self.disp);
        va_query_config_entrypoints(self.disp, profile, max_entrypoints)
    }

    pub fn get_va_version(&self) -> (i32, i32) {
        (self.maj, self.min)
    }

    pub fn get_max_profiles(&self) -> i32 {
        self.max_profiles
    }

    pub fn get_display(&self) -> ffi::VADisplay {
        self.disp
    }

    pub fn get_vendor_string(&self) -> &String {
        &self.vendor_string
    }

    pub fn destroy(&self) {
        match va_terminate(self.disp) {
            VA_STATUS_SUCCESS => {
                println!("destroy success!");
            }
            _ => {
                println!("destroy fail!");
            }
        }
    }
}

pub struct VAImage {
    image: ffi::VAImage,
}

pub struct VAImageFormat {
    format: ffi::VAImageFormat,
}

impl VAImageFormat {
    pub fn new(format: ffi::VAImageFormat) -> VAImageFormat {
        VAImageFormat { format: format }
    }
}

impl VAImage {
    pub fn new(va_disp: &VADisplay,
               format: &VAImageFormat,
               width: c_int,
               height: c_int)
               -> Result<VAImage, ()> {
        let mut image: ffi::VAImage = Default::default();

        match va_create_image(va_disp.disp, &format.format, width, height, &mut image) {
            VA_STATUS_SUCCESS => Ok(VAImage { image: image }),
            _ => return Err(()),
        }
    }

    pub fn destroy(&self, va_disp: &VADisplay) {
        match va_destroy_image(va_disp.disp, self.image.image_id) {
            VA_STATUS_SUCCESS => {
                println!("image destroy success!");
            }
            _ => {
                println!("image destroy fail!");
            }
        }
    }

    pub fn put_image(&self,
                     va_disp: &VADisplay,
                     va_surface: &VASurface,
                     src_x: c_int,
                     src_y: c_int,
                     src_w: c_uint,
                     src_h: c_uint,
                     dst_x: c_int,
                     dst_y: c_int,
                     dst_w: c_uint,
                     dst_h: c_uint) {
        match va_put_image(va_disp.disp,
                           va_surface.id,
                           self.image.image_id,
                           src_x,
                           src_y,
                           src_w,
                           src_h,
                           dst_x,
                           dst_y,
                           dst_w,
                           dst_h) {
            VA_STATUS_SUCCESS => {
                println!("put image success!");
            }
            _ => {
                println!("put image fail!");
            }
        }
    }

    pub fn get_id(&self) -> u32 {
        self.image.image_id
    }

    pub fn get_num_planes(&self) -> u32 {
        self.image.num_planes
    }

    pub fn get_offset(&self, idx: usize) -> u32 {
        self.image.offsets[idx]
    }

    pub fn get_stride(&self, idx: usize) -> u32 {
        self.image.pitches[idx]
    }

    pub fn get_buffer(&self) -> VABuffer {
        VABuffer {
            id: self.image.buf,
            size: self.image.data_size,
            num_elem: 1,
        }
    }
}

pub struct VASurface {
    id: ffi::VASurfaceID,
    format: c_uint,
    width: c_uint,
    height: c_uint,
    num_surfaces: c_uint,
}

impl VASurface {
    pub fn new(va_disp: &VADisplay,
               width: c_uint,
               height: c_uint,
               format: c_uint,
               num_surfaces: c_uint)
               -> Result<VASurface, ()> {
        let surface_id = 0;
        let surface_p = &surface_id as *const u32;
        match va_create_surfaces(va_disp.disp,
                                 width,
                                 height,
                                 format,
                                 num_surfaces,
                                 surface_p as *mut u32) {
            VA_STATUS_SUCCESS => {
                Ok(VASurface {
                    id: surface_id,
                    format: format,
                    width: width,
                    height: height,
                    num_surfaces: num_surfaces,
                })
            }
            _ => return Err(()),
        }
    }

    pub fn destroy_surfaces(&self, va_disp: &VADisplay) {
        let surface_p = &self.id as *const u32;
        match va_destroy_surfaces(va_disp.disp,
                                  self.num_surfaces as c_int,
                                  surface_p as *mut u32) {
            VA_STATUS_SUCCESS => {
                println!("surface destroy success!");
            }
            _ => {
                println!("surface destroy fail!");
            }
        }
    }

    pub fn put_surface(&self, va_disp: &VADisplay, win: VANativeDrawable,
                        srcx: c_short,
                        srcy: c_short,
                        srcw: c_uint,
                        srch: c_uint,
                        dstx: c_short,
                        dsty: c_short,
                        dstw: c_uint,
                        dsth: c_uint,
                        ) {
        match va_put_surface(va_disp.disp, win, self.id,
                             srcx, srcy, srcw as c_ushort, srch as c_ushort,
                             dstx, dsty, dstw as c_ushort, dsth as c_ushort) {
            VA_STATUS_SUCCESS => {
                println!("put surface success!");
            }
            _ => {
                println!("put surface fail!");
            }
        }
    }

    pub fn derive_image(&self, va_disp: &VADisplay) -> Result<VAImage, ()> {
        let mut image: ffi::VAImage = Default::default();

        match va_derive_image(va_disp.disp, self.id, &mut image) {
            VA_STATUS_SUCCESS => {
                Ok(VAImage { image: image })
            }
            _ => return Err(()),
        }
    }

    pub fn sync(&self, va_disp: &VADisplay) {
        match va_sync_surface(va_disp.disp, self.id) {
            VA_STATUS_SUCCESS => {
                println!("sync surface success!");
            }
            _ => {
                println!("sync surface fail!");
            }
        }
    }

    pub fn get_id(&self) -> ffi::VASurfaceID {
        self.id
    }

    pub fn get_format(&self) -> u32 {
        self.format
    }
}

pub struct VAConfig {
    id: ffi::VAConfigID,
    attr_list: *const ffi::VAConfigAttrib,
    attr_num: c_int,
    profile: VAProfile,
    entrypoint: VAEntrypoint,
}

impl VAConfig {
    pub fn new(va_disp: &VADisplay,
               profile: VAProfile,
               entrypoint: VAEntrypoint,
               attr_num: c_int)
               -> Result<VAConfig, ()> {
        let mut attr_list = ffi::VAConfigAttrib {
            type_: ffi::VAConfigAttribRTFormat,
            value: 0,
        };
        match va_get_config_attributes(va_disp.disp,
                                       profile,
                                       entrypoint,
                                       &mut attr_list,
                                       attr_num) {
            VA_STATUS_SUCCESS => {
                println!("va_get_config_attributes success!");
            }
            _ => return Err(()),
        }

        let mut id = 0;
        match va_create_config(va_disp.disp,
                               profile,
                               entrypoint,
                               &mut attr_list,
                               attr_num,
                               &mut id) {
            VA_STATUS_SUCCESS => {
                Ok(VAConfig {
                    id: id,
                    attr_list: &attr_list,
                    attr_num: attr_num,
                    profile: profile,
                    entrypoint: entrypoint,
                })
            }
            _ => return Err(()),
        }
    }

    pub fn destroy(&self, va_disp: &VADisplay) {
        match va_destroy_config(va_disp.disp, self.id) {
            VA_STATUS_SUCCESS => {
                println!("config destroy success!");
            }
            _ => {
                println!("config destroy fail!");
            }
        }
    }
}

pub struct VAContext {
    id: ffi::VAContextID,
    width: c_int,
    height: c_int,
    flag: c_int,
    //render_targets: *const VASurfaceID,
    //num_render_targets: c_int,
}

impl VAContext {
    pub fn new(va_disp: &VADisplay,
               va_config: &VAConfig,
               va_surface: &VASurface,
               width: c_int,
               height: c_int,
               flag: c_int)
               -> Result<VAContext, ()> {

        let mut id = 0;
        let surface_p = &va_surface.id as *const u32;
        match va_create_context(va_disp.disp,
                                va_config.id,
                                width,
                                height,
                                flag,
                                surface_p as *mut u32,
                                va_surface.num_surfaces as c_int,
                                &mut id) {
            VA_STATUS_SUCCESS => {
                Ok(VAContext {
                    id: id,
                    width: width,
                    height: height,
                    flag: flag,
                })
            }
            _ => return Err(()),
        }
    }

    pub fn destroy(&self, va_disp: &VADisplay) {
        match va_destroy_context(va_disp.disp, self.id) {
            VA_STATUS_SUCCESS => {
                println!("context destroy success!");
            }
            _ => {
                println!("context destroy fail!");
            }
        }
    }

    pub fn get_id(&self) -> ffi::VAContextID {
        self.id
    }
}

pub struct VABuffer {
    id: ffi::VABufferID,
    size: c_uint,
    num_elem: c_uint,
}

impl VABuffer {
    pub fn new(va_disp: &VADisplay,
               va_context: &VAContext,
               buffer_type: VABufferType,
               size: c_uint,
               num_elem: c_uint,
               data: *mut c_void)
               -> Result<VABuffer, ()> {
        let mut id = 0;
        match va_create_buffer(va_disp.disp,
                               va_context.id,
                               buffer_type,
                               size,
                               num_elem,
                               data,
                               &mut id) {
            VA_STATUS_SUCCESS => {
                Ok(VABuffer {
                    id: id,
                    size: size,
                    num_elem: num_elem,
                })
            }
            _ => return Err(()),
        }
    }

    pub fn destroy(&self, va_disp: &VADisplay) {
        match va_destroy_buffer(va_disp.disp, self.id) {
            VA_STATUS_SUCCESS => {
                println!("buffer destroy success!");
            }
            _ => {
                println!("buffer destroy fail!");
            }
        }
    }

    pub fn map(&self, va_disp: &VADisplay) -> *mut u8 {
        let mut p_buf = ptr::null_mut();
        match va_map_buffer(va_disp.disp, self.id, &mut p_buf) {
            VA_STATUS_SUCCESS => {
                p_buf as *mut u8
            }
            _ => {
                ptr::null_mut()
            }
        }
    }

    pub fn unmap(&self, va_disp: &VADisplay) {
        match va_unmap_buffer(va_disp.disp, self.id) {
            VA_STATUS_SUCCESS => {
                println!("buffer unmap success!");
            }
            _ => {
                println!("buffer unmap fail!");
            }
        }
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

    pub fn get_id(&self) -> ffi::VABufferID {
        self.id
    }
}



pub fn va_init(disp: ffi::VADisplay, maj: *mut c_int, min: *mut c_int) -> ffi::VAStatus {
    unsafe { ffi::vaInitialize(disp, maj, min) }
}

pub fn va_get_display(native_disp: *mut VANativeDisplay) -> ffi::VADisplay {
    unsafe { ffi_x11::vaGetDisplay(native_disp) }
}

pub fn va_terminate(disp: ffi::VADisplay) -> ffi::VAStatus {
    unsafe { ffi::vaTerminate(disp) }
}

pub fn va_max_num_profiles(disp: ffi::VADisplay) -> c_int {
    unsafe { ffi::vaMaxNumProfiles(disp) }
}

pub fn va_max_num_entrypoints(disp: ffi::VADisplay) -> c_int {
    unsafe { ffi::vaMaxNumEntrypoints(disp) }
}

pub fn va_create_surfaces(disp: ffi::VADisplay,
                          width: c_uint,
                          height: c_uint,
                          format: c_uint,
                          num_surfaces: c_uint,
                          surfaces: *mut ffi::VASurfaceID)
                          -> ffi::VAStatus {
    unsafe {
        let mut attr = ffi::VASurfaceAttrib {
            type_: ffi::VASurfaceAttribPixelFormat,
            flags: ffi::VA_SURFACE_ATTRIB_SETTABLE,
            value: ffi::VAGenericValue {
                type_: ffi::VAGenericValueTypeInteger,
                value: ffi::_VAGenericValue__bindgen_ty_1 {
                    i: Default::default(),
                    f: Default::default(),
                    p: Default::default(),
                    fn_: Default::default(),
                    bindgen_union_field: ffi::VA_FOURCC_NV12 as u64,
                },
            },
        };

        ffi::vaCreateSurfaces(disp,
                              format,
                              width,
                              height,
                              surfaces,
                              num_surfaces,
                              &mut attr,
                              1)
    }
}

pub fn va_destroy_surfaces(disp: ffi::VADisplay,
                           num_surfaces: c_int,
                           surfaces: *mut ffi::VASurfaceID)
                           -> ffi::VAStatus {
    unsafe { ffi::vaDestroySurfaces(disp, surfaces, num_surfaces) }
}

pub fn va_sync_surface(disp: ffi::VADisplay, target: ffi::VASurfaceID) -> ffi::VAStatus {
    unsafe { ffi::vaSyncSurface(disp, target) }
}

pub fn va_get_config_attributes(disp: ffi::VADisplay,
                                profile: ffi::VAProfile,
                                entrypoint: ffi::VAEntrypoint,
                                attrib_list: *mut ffi::VAConfigAttrib,
                                attr_num: c_int)
                                -> ffi::VAStatus {
    unsafe { ffi::vaGetConfigAttributes(disp, profile, entrypoint, attrib_list, attr_num) }
}

pub fn va_query_vendor_string(disp: ffi::VADisplay) -> String {
    unsafe {
        let str = ffi::vaQueryVendorString(disp) as *mut i8;
        CString::into_string(CString::from_raw(str)).unwrap()
    }
}

pub fn va_query_config_profiles(disp: ffi::VADisplay, max_len: c_int)
                                -> Vec<VAProfile> {
    let mut profile_num = 0;
    let mut profiles: Vec<VAProfile> = Vec::with_capacity(max_len as usize);
    let v_ptr = profiles.as_mut_ptr();

    unsafe {
        ffi::vaQueryConfigProfiles(disp, v_ptr, &mut profile_num);
        let rebuilt = Vec::from_raw_parts(v_ptr, profile_num as usize, max_len as usize);
        rebuilt
    }

}

pub fn va_query_config_entrypoints(disp: ffi::VADisplay, profile: ffi::VAProfile, max_len: c_int)
                                -> Vec<VAEntrypoint> {
    let mut entry_num = 0;
    let mut entries: Vec<VAEntrypoint> = Vec::with_capacity(max_len as usize);
    let e_ptr = entries.as_mut_ptr();

    unsafe {
        ffi::vaQueryConfigEntrypoints(disp, profile, e_ptr, &mut entry_num);
        let rebuilt = Vec::from_raw_parts(e_ptr, entry_num as usize, max_len as usize);
        rebuilt
    }
}

pub fn va_create_config(disp: ffi::VADisplay,
                        profile: ffi::VAProfile,
                        entrypoint: ffi::VAEntrypoint,
                        attrib_list: *mut ffi::VAConfigAttrib,
                        attr_num: c_int,
                        id: *mut ffi::VAConfigID)
                        -> ffi::VAStatus {
    unsafe { ffi::vaCreateConfig(disp, profile, entrypoint, attrib_list, attr_num, id) }
}

pub fn va_create_context(disp: ffi::VADisplay,
                         config_id: ffi::VAConfigID,
                         width: c_int,
                         height: c_int,
                         flag: c_int,
                         render_target: *mut ffi::VASurfaceID,
                         num_render_targets: c_int,
                         context: *mut ffi::VAContextID)
                         -> ffi::VAStatus {
    unsafe {
        ffi::vaCreateContext(disp,
                             config_id,
                             width,
                             height,
                             flag,
                             render_target,
                             num_render_targets,
                             context)
    }
}

pub fn va_destroy_config(disp: ffi::VADisplay, id: ffi::VAConfigID) -> ffi::VAStatus {
    unsafe { ffi::vaDestroyConfig(disp, id) }
}

pub fn va_destroy_context(disp: ffi::VADisplay, id: ffi::VAContextID) -> ffi::VAStatus {
    unsafe { ffi::vaDestroyContext(disp, id) }
}

pub fn va_put_surface(disp: ffi::VADisplay,
                      win: VANativeDrawable,
                      surface_id: ffi::VASurfaceID,
                      srcx: c_short,
                      srcy: c_short,
                      srcw: c_ushort,
                      srch: c_ushort,
                      dstx: c_short,
                      dsty: c_short,
                      dstw: c_ushort,
                      dsth: c_ushort
                      )
                      -> ffi::VAStatus {
    unsafe {
        ffi_x11::vaPutSurface(disp,
                          surface_id,
                          win,
                          srcx,
                          srcy,
                          srcw,
                          srch,
                          dstx,
                          dsty,
                          dstw,
                          dsth,
                          ptr::null_mut(),
                          0,
                          0)
    }
}

pub fn va_create_buffer(disp: ffi::VADisplay,
                        context_id: ffi::VAContextID,
                        buffer_type: ffi::VABufferType,
                        size: c_uint,
                        num_elem: c_uint,
                        data: *mut c_void,
                        buffer_id: *mut ffi::VABufferID)
                        -> ffi::VAStatus {
    unsafe {
        ffi::vaCreateBuffer(disp,
                            context_id,
                            buffer_type,
                            size,
                            num_elem,
                            data,
                            buffer_id)
    }
}

pub fn va_destroy_buffer(disp: ffi::VADisplay, id: ffi::VABufferID) -> ffi::VAStatus {
    unsafe { ffi::vaDestroyBuffer(disp, id) }
}

pub fn va_map_buffer(disp: ffi::VADisplay,
                     id: ffi::VABufferID,
                     pbuf: *mut *mut c_void)
                     -> ffi::VAStatus {
    unsafe { ffi::vaMapBuffer(disp, id, pbuf) }
}

pub fn va_unmap_buffer(disp: ffi::VADisplay, id: ffi::VABufferID) -> ffi::VAStatus {
    unsafe { ffi::vaUnmapBuffer(disp, id) }
}

pub fn va_create_image(disp: ffi::VADisplay,
                       format: *const ffi::VAImageFormat,
                       width: c_int,
                       height: c_int,
                       image: *mut ffi::VAImage)
                       -> ffi::VAStatus {
    unsafe {
        ffi::vaCreateImage(disp,
                           format as *mut ffi::VAImageFormat,
                           width,
                           height,
                           image)
    }
}

pub fn va_destroy_image(disp: ffi::VADisplay, id: ffi::VAImageID) -> ffi::VAStatus {
    unsafe { ffi::vaDestroyImage(disp, id) }
}

pub fn va_derive_image(disp: ffi::VADisplay,
                       surface_id: ffi::VASurfaceID,
                       image: *mut ffi::VAImage)
                       -> ffi::VAStatus {
    unsafe { ffi::vaDeriveImage(disp, surface_id, image) }
}

pub fn va_begin_picture(disp: ffi::VADisplay,
                        context_id: ffi::VAContextID,
                        surface_id: ffi::VASurfaceID)
                        -> ffi::VAStatus {
    unsafe { ffi::vaBeginPicture(disp, context_id, surface_id) }
}

pub fn va_render_picture(disp: ffi::VADisplay,
                         context_id: ffi::VAContextID,
                         buffers: *mut ffi::VABufferID,
                         num_buffers: c_int)
                         -> ffi::VAStatus {
    unsafe { ffi::vaRenderPicture(disp, context_id, buffers, num_buffers) }
}

pub fn va_end_picture(disp: ffi::VADisplay, context_id: ffi::VAContextID) -> ffi::VAStatus {
    unsafe { ffi::vaEndPicture(disp, context_id) }
}

pub fn va_put_image(disp: ffi::VADisplay,
                    surface_id: ffi::VASurfaceID,
                    image_id: ffi::VAImageID,
                    src_x: c_int,
                    src_y: c_int,
                    src_w: c_uint,
                    src_h: c_uint,
                    dst_x: c_int,
                    dst_y: c_int,
                    dst_w: c_uint,
                    dst_h: c_uint)
                    -> ffi::VAStatus {
    unsafe {
        ffi::vaPutImage(disp,
                        surface_id,
                        image_id,
                        src_x,
                        src_y,
                        src_w,
                        src_h,
                        dst_x,
                        dst_y,
                        dst_w,
                        dst_h)
    }
}
