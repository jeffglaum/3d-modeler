use std::sync::RwLock;
use web_sys::WebGl2RenderingContext as GL;

use super::set_attribute;
use crate::global::Vertex;
use crate::vao::VertexArray;
use crate::vbo::Buffer;

pub struct ModelObject {
    gl: GL,
    loaded: bool,
    vao: VertexArray,
    vbo: Buffer,
    ibo: Buffer,
    vertices: Option<RwLock<Vec<Vertex>>>,
    indices: Option<RwLock<Vec<u32>>>,
    draw_wireframe: bool,
    color: [f32; 4], // RGBA color
}

impl ModelObject {
    pub fn new(gl: GL) -> Self {
        let vao = unsafe { VertexArray::new(&gl) };
        let vbo = unsafe { Buffer::new(&gl, GL::ARRAY_BUFFER) };
        let ibo = unsafe { Buffer::new(&gl, GL::ELEMENT_ARRAY_BUFFER) };
        Self {
            gl,
            loaded: false,
            vao,
            vbo,
            ibo,
            vertices: None,
            indices: None,
            draw_wireframe: true,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn update_model(&mut self, vertices: Vec<Vertex>, indices: Vec<u32>) {
        // TODO - need to drop a previous allocation here?
        // TODO - is RwLock needed here?
        self.vertices = Some(RwLock::new(Vec::new()));
        self.indices = Some(RwLock::new(Vec::new()));
        self.vertices
            .as_ref()
            .unwrap()
            .write()
            .unwrap()
            .extend(vertices.clone());
        self.indices
            .as_ref()
            .unwrap()
            .write()
            .unwrap()
            .extend(indices.clone());
        unsafe {
            let vao = &self.vao;
            let gl = &self.gl;
            self.vbo.set_data(
                &self.gl,
                self.vertices.as_ref().unwrap().read().unwrap().to_vec(),
                GL::STATIC_DRAW,
            );
            set_attribute!(vao, gl, 0, Vertex::0);
            set_attribute!(vao, gl, 1, Vertex::1);
            self.ibo.set_data(
                &gl,
                self.indices.as_ref().unwrap().read().unwrap().to_vec(),
                GL::STATIC_DRAW,
            );
            //self.vao.bind(&gl)
        };
        self.loaded = true;
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn get_color(&self) -> [f32; 4] {
        self.color
    }

    pub fn set_draw_wireframe(&mut self, draw_wireframe: bool) {
        self.draw_wireframe = draw_wireframe;
    }

    pub fn get_draw_wireframe(&self) -> bool {
        self.draw_wireframe
    }

    pub fn get_indices_count(&self) -> usize {
        if let Some(ref indices) = self.indices {
            return indices.read().unwrap().len();
        }
        0
    }

    pub fn bind(&self) {
        unsafe { self.vbo.bind(&self.gl) };
        unsafe { self.vao.bind(&self.gl) };
    }
}
