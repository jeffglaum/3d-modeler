use web_sys::{WebGl2RenderingContext as GL, WebGlVertexArrayObject};
pub struct VertexArray {
    pub obj: Option<WebGlVertexArrayObject>,
}

impl VertexArray {
    pub fn _get_obj(&self) -> &Option<WebGlVertexArrayObject> {
        &self.obj
    }
}

impl VertexArray {
    pub unsafe fn new(gl: &GL) -> Self {
        let vao = gl
            .create_vertex_array()
            .ok_or("Could not create VAO")
            .unwrap();
        Self { obj: Some(vao) }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        // TODO
    }
}

impl VertexArray {
    pub unsafe fn bind(&self, gl: &GL) {
        gl.bind_vertex_array(Some(self.obj.as_ref().unwrap().as_ref()));
    }
}

impl VertexArray {
    pub unsafe fn set_attribute<V: Sized>(
        &self,
        gl: &GL,
        attrib_pos: u32,
        components: i32,
        offset: i32,
    ) {
        self.bind(gl);

        gl.vertex_attrib_pointer_with_i32(
            attrib_pos,
            components,
            GL::FLOAT,
            false,
            std::mem::size_of::<V>() as i32,
            offset,
        );
        gl.enable_vertex_attrib_array(attrib_pos);
    }
}
