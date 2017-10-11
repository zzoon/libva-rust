use std::fmt;

pub enum RendererBackend {
    X11,
    WAYLAND,
    DRM,
}

pub trait VARenderer: Send + 'static {
    fn open(&mut self) -> Option<u8>;
    fn close(&mut self) -> Option<u8>;
    fn set_resolution(&mut self, width: u32, height: u32) -> Option<u8>;
    fn render(&self, data: &[u8], len: usize) -> Option<u8>;
}

impl fmt::Debug for VARenderer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Renderer")
    }
}
