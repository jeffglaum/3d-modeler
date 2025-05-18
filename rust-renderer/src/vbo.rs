use js_sys::Uint8Array;
use web_sys::{WebGl2RenderingContext as GL, WebGlBuffer};
pub struct Buffer {
    pub obj: WebGlBuffer,
    target: u32,
}

impl Buffer {
    pub unsafe fn new(gl: &GL, target: u32) -> Self {
        let vbo = gl
            .create_buffer()
            .ok_or("ERROR: could not create VBO")
            .unwrap();
        Self { obj: vbo, target }
    }

    pub unsafe fn bind(&self, gl: &GL) {
        gl.bind_buffer(self.target, Some(&self.obj));
    }

    pub unsafe fn set_data<D>(&self, gl: &GL, data: Vec<D>, usage: u32) {
        self.bind(gl);
        let (_, data_bytes, _) = data.align_to::<u8>();
        let js_array = Uint8Array::view(data_bytes);
        let js_object: js_sys::Object = js_array.into();
        gl.buffer_data_with_array_buffer_view(self.target, &js_object, usage);
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        // TODO
    }
}
