mod vao;
mod vbo;

use crate::vao::VertexArray;
use crate::vbo::Buffer;

use cgmath::{perspective, Deg, InnerSpace, Matrix4, Point3, SquareMatrix, Vector3};
use rand::Rng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::BufReader;
use std::rc::Rc;
use std::sync::RwLock;
use wasm_bindgen::prelude::*;
use wavefront_rs::obj::{entity::*, parser::*};
use web_sys::{
    console, HtmlCanvasElement, WebGl2RenderingContext as GL, WebGlProgram, WebGlShader,
};

// vertex data type
type Pos = [f32; 3];
type Norm = [f32; 3];

#[repr(C, packed)]
#[derive(Debug, Clone)]
struct Vertex(Pos, Norm);

// Global storage for vertices and indices
thread_local! {
    static VERTICES: RwLock<Vec<Vertex>> = RwLock::new(Vec::new());
    static INDICES: RwLock<Vec<u32>> = RwLock::new(Vec::new());
}

#[macro_export]
macro_rules! set_attribute {
    ($vbo:ident, $gl:ident, $pos:tt, $t:ident :: $field:tt) => {{
        let dummy = core::mem::MaybeUninit::<$t>::uninit();
        let dummy_ptr = dummy.as_ptr();
        let member_ptr = core::ptr::addr_of!((*dummy_ptr).$field);
        const fn size_of_raw<T>(_: *const T) -> usize {
            core::mem::size_of::<T>()
        }
        let member_offset = member_ptr as i32 - dummy_ptr as i32;
        $vbo.set_attribute::<$t>(
            &$gl,
            $pos,
            (size_of_raw(member_ptr) / core::mem::size_of::<f32>()) as i32,
            member_offset,
        )
    }};
}

fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or("Unable to create shader")?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl.get_shader_info_log(&shader).unwrap_or_default())
    }
}

fn link_program(
    gl: &GL,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl.create_program().ok_or("Unable to create program")?;
    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl.get_program_info_log(&program).unwrap_or_default())
    }
}

/// Converts a cgmath::Matrix4<f32> into a column-major [f32; 16] array for OpenGL.
pub fn matrix4_to_array(matrix: &Matrix4<f32>) -> [f32; 16] {
    [
        matrix.x.x, matrix.x.y, matrix.x.z, matrix.x.w, matrix.y.x, matrix.y.y, matrix.y.z,
        matrix.y.w, matrix.z.x, matrix.z.y, matrix.z.z, matrix.z.w, matrix.w.x, matrix.w.y,
        matrix.w.z, matrix.w.w,
    ]
}

#[wasm_bindgen]
pub fn handle_mouse_click(x: f64, y: f64) {
    console::log_1(&format!("INFO: mouse click at: {}, {}", x, y).into());
}

#[wasm_bindgen]
pub fn process_file_content(content: &str) {
    // Log the file content to the browser console (for debugging)
    //web_sys::console::log_1(&format!("Received file content: {}", content).into());

    // Prepare to parse the OBJ content
    let mut positions: HashMap<usize, Pos> = HashMap::new();
    let mut normals: HashMap<usize, Norm> = HashMap::new();
    let mut vertices: Vec<Vertex> = vec![];
    let mut indices: Vec<u32> = vec![];

    // Parse the OBJ content from the provided string
    Parser::read_to_end(&mut BufReader::new(content.as_bytes()), |x| match x {
        Entity::Vertex { x, y, z, w: _ } => {
            let index = positions.len() + 1; // OBJ indices are 1-based
            positions.insert(index, [x as f32, y as f32, z as f32]);
            //web_sys::console::log_1(&format!("Vertex: {},{},{}", x, y, z).into());
        }
        Entity::VertexNormal { x, y, z } => {
            let index = normals.len() + 1; // OBJ indices are 1-based
            normals.insert(index, [x as f32, y as f32, z as f32]);
            //web_sys::console::log_1(&format!("Vertex Normal: {},{},{}", x, y, z).into());
        }
        Entity::Face {
            vertices: face_vertices,
        } => {
            for v in face_vertices {
                let pos_index = v.vertex as usize;
                let norm_index = v.normal.unwrap_or(0) as usize;

                // Retrieve position and normal
                if let Some(pos) = positions.get(&pos_index) {
                    let norm = normals.get(&norm_index).unwrap_or(&[0.0, 0.0, 0.0]);
                    vertices.push(Vertex(*pos, *norm));
                }

                // Push the index (subtract 1 because OBJ indices are 1-based)
                indices.push((v.vertex - 1) as u32);
            }
        }
        _ => {}
    })
    .unwrap();

    // Store the vertices and indices in the global storage
    VERTICES.with(|v| {
        let mut global_vertices = v.write().unwrap();
        *global_vertices = vertices;
    });

    INDICES.with(|i| {
        let mut global_indices = i.write().unwrap();
        *global_indices = indices;
    });

    // Log the number of indices for debugging
    //let indices_length = INDICES.with(|i| i.read().unwrap().len());
    //web_sys::console::log_1(&format!("Number of indices: {}", indices_length).into());
}

#[wasm_bindgen]
pub fn start_rendering(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let gl: GL = canvas.get_context("webgl2")?.unwrap().dyn_into::<GL>()?;

    // Shaders
    let vert_shader = compile_shader(
        &gl,
        GL::VERTEX_SHADER,
        r#"#version 300 es
        layout(location = 0) in vec3 aPos;
        layout(location = 1) in vec3 aNormal;

        out vec3 FragPos;
        out vec3 Normal;

        uniform mat4 model;
        uniform mat4 view;
        uniform mat4 projection;

        void main() {
            FragPos = vec3(model * vec4(aPos, 1.0));
            Normal = mat3(transpose(inverse(model))) * aNormal;
            gl_Position = projection * view * vec4(FragPos, 1.0);
        }"#,
    )
    .unwrap();

    let frag_shader = compile_shader(
        &gl,
        GL::FRAGMENT_SHADER,
        r#"#version 300 es
        precision mediump float;
        in vec3 FragPos;
        in vec3 Normal;

        out vec4 FragColor;

        uniform vec3 lightPos;
        uniform vec3 viewPos;
        uniform vec3 lightColor;
        uniform vec3 objectColor;

        void main() {
            float ambientStrength = 0.1;
            vec3 ambient = ambientStrength * lightColor;

            vec3 norm = normalize(Normal);
            vec3 lightDir = normalize(lightPos - FragPos);
            float diff = max(dot(norm, lightDir), 0.0);
            vec3 diffuse = diff * lightColor;

            float specularStrength = 0.5;
            vec3 viewDir = normalize(viewPos - FragPos);
            vec3 reflectDir = reflect(-lightDir, norm);
            float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32.0);
            vec3 specular = specularStrength * spec * lightColor;

            vec3 result = (ambient + diffuse + specular) * objectColor;
            FragColor = vec4(result, 1.0);
        }"#,
    )
    .unwrap();

    let program = link_program(&gl, &vert_shader, &frag_shader).unwrap();
    gl.use_program(Some(&program));

    gl.clear_color(0.0, 0.0, 0.0, 1.0);

    // Retrieve the vertices and indices from the global storage
    let vertices = VERTICES.with(|v| v.read().unwrap().clone());
    let indices = INDICES.with(|i| i.read().unwrap().clone());
    let indices_length = indices.len() as i32;

    // Log the number of vertices and indices for debugging
    //web_sys::console::log_1(&format!("Rendering with {} vertices", vertices.len()).into());
    //web_sys::console::log_1(&format!("Rendering with {} indices", indices.len()).into());

    // Create VBO
    let vbo = unsafe { Buffer::new(&gl, GL::ARRAY_BUFFER) };
    unsafe { vbo.set_data(&gl, vertices, GL::STATIC_DRAW) };

    // Create VAO
    let vao = unsafe { VertexArray::new(&gl) };
    unsafe { set_attribute!(vao, gl, 0, Vertex::0) };
    unsafe { set_attribute!(vao, gl, 1, Vertex::1) };

    // Create index (element) buffer
    let index_buffer = unsafe { Buffer::new(&gl, GL::ELEMENT_ARRAY_BUFFER) };
    unsafe { index_buffer.set_data(&gl, indices, GL::STATIC_DRAW) };

    // Bind the VBO to the VAO
    unsafe { vao.bind(&gl) };

    // View matrix
    let view = Matrix4::look_at_rh(
        Point3::new(0.0, 0.0, 4.0),
        Point3::new(0.0, 0.0, 0.0),
        Vector3::unit_y(),
    );

    // Projection matrix
    let projection = perspective(Deg(45.0), 1920.0 / 1080.0, 0.1, 100.0);

    // Choose a random axis of rotation
    let mut rng = rand::rng();
    let rotation_axis = Vector3::new(
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
    )
    .normalize();

    // Adjust the model matrix by rotating it around the randomly-chosen vector
    let model = Matrix4::identity();

    let model_loc = gl
        .get_uniform_location(&program, "model")
        .ok_or("Could not get model uniform location")
        .unwrap();
    let view_loc = gl
        .get_uniform_location(&program, "view")
        .ok_or("Could not get view uniform location")
        .unwrap();
    let proj_loc = gl
        .get_uniform_location(&program, "projection")
        .ok_or("Could not get projection uniform location")
        .unwrap();
    let light_pos_loc = gl
        .get_uniform_location(&program, "lightPos")
        .ok_or("Could not get lightPos uniform location")
        .unwrap();
    let view_pos_loc = gl
        .get_uniform_location(&program, "viewPos")
        .ok_or("Could not get viewPos uniform location")
        .unwrap();
    let light_color_loc = gl
        .get_uniform_location(&program, "lightColor")
        .ok_or("Could not get lightColor uniform location")
        .unwrap();
    let object_color_loc = gl
        .get_uniform_location(&program, "objectColor")
        .ok_or("Could not get objectColor uniform location")
        .unwrap();

    // Assign shader variable data
    gl.uniform_matrix4fv_with_f32_array(Some(&model_loc), false, &matrix4_to_array(&model));
    gl.uniform_matrix4fv_with_f32_array(Some(&view_loc), false, &matrix4_to_array(&view));
    gl.uniform_matrix4fv_with_f32_array(Some(&proj_loc), false, &matrix4_to_array(&projection));
    gl.uniform3f(Some(&light_pos_loc), 1.2, 1.0, 2.0);
    gl.uniform3f(Some(&view_pos_loc), 0.0, 0.0, 2.0);
    gl.uniform3f(Some(&light_color_loc), 1.0, 1.0, 1.0);
    gl.uniform3f(Some(&object_color_loc), 0.3, 0.5, 1.0);

    gl.enable(GL::DEPTH_TEST);
    gl.depth_func(GL::LESS);

    // Animate the rotation
    animate(0.0, gl, program, indices_length, rotation_axis);
    Ok(())
}

fn animate(
    start_time: f64,
    gl: GL,
    program: WebGlProgram,
    indices_length: i32,
    rotation_axis: Vector3<f32>,
) {
    let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let closure = Closure::wrap(Box::new(move |time: f64| {
        // Clear the screen
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        // Update model matrix based on rotation around an axis
        let angle = Deg(((time - start_time) / 1000.0) as f32 * 45.0); // 45 degrees per second
        let model = Matrix4::from_axis_angle(rotation_axis, angle);

        // Update the shader program with the model
        gl.use_program(Some(&program));
        let model_loc = gl
            .get_uniform_location(&program, "model")
            .ok_or("Could not get model uniform location")
            .unwrap();
        gl.uniform_matrix4fv_with_f32_array(Some(&model_loc), false, &matrix4_to_array(&model));

        // Draw
        gl.draw_arrays(GL::TRIANGLES, 0, indices_length);

        // Schedule next frame
        web_sys::window()
            .unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut(f64)>);

    *g.borrow_mut() = Some(closure);

    web_sys::window()
        .unwrap()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}
