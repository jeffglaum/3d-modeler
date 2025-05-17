mod vao;
mod vbo;

use crate::vao::VertexArray;
use crate::vbo::Buffer;

use cgmath::{perspective, Deg, Matrix4, Point3, SquareMatrix, Vector3};
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
    static MODEL_LOADED: RwLock<bool> = RwLock::new(false);
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
            // Fan triangulation: create triangles from v0, vi, vi+1
            if face_vertices.len() >= 3 {
                let v0 = &face_vertices[0];
                for i in 1..face_vertices.len() - 1 {
                    let v1 = &face_vertices[i];
                    let v2 = &face_vertices[i + 1];

                    let indices_set = [v0, v1, v2];

                    for v in &indices_set {
                        let pos_index = v.vertex as usize;
                        let norm_index = v.normal.unwrap_or(0) as usize;

                        if let Some(pos) = positions.get(&pos_index) {
                            let norm = normals.get(&norm_index).unwrap_or(&[0.0, 0.0, 0.0]);
                            vertices.push(Vertex(*pos, *norm));
                        }

                        // OBJ indices are 1-based, subtract 1
                        indices.push((v.vertex - 1) as u32);
                    }
                }
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

    MODEL_LOADED.with(|i| {
        let mut loaded = i.write().unwrap();
        *loaded = false;
    });

    // Log the number of indices for debugging
    let indices_length = INDICES.with(|i| i.read().unwrap().len());
    web_sys::console::log_1(&format!("Number of indices: {}", indices_length).into());
}

pub fn enable_mouse_controls(
    canvas: HtmlCanvasElement,
    rotation: Rc<RefCell<(f64, f64)>>,
) -> Result<(), JsValue> {
    let canvas = Rc::new(canvas);
    let is_dragging = Rc::new(RefCell::new(false));
    let last_mouse_pos = Rc::new(RefCell::new((0.0, 0.0)));

    // Clone references for the `mousedown` event
    let canvas_clone = canvas.clone();
    let is_dragging_clone = is_dragging.clone();
    let last_mouse_pos_clone = last_mouse_pos.clone();

    // Mouse down event
    let on_mouse_down = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        *is_dragging_clone.borrow_mut() = true;
        *last_mouse_pos_clone.borrow_mut() = (event.client_x() as f64, event.client_y() as f64);
    }) as Box<dyn FnMut(_)>);
    canvas_clone
        .add_event_listener_with_callback("mousedown", on_mouse_down.as_ref().unchecked_ref())?;
    on_mouse_down.forget();

    // Clone references for the `mousemove` event
    let canvas_clone = canvas.clone();
    let is_dragging_clone = is_dragging.clone();
    let last_mouse_pos_clone = last_mouse_pos.clone();
    let rotation_clone = rotation.clone();

    // Mouse move event
    let on_mouse_move = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        if *is_dragging_clone.borrow() {
            let (last_x, last_y) = *last_mouse_pos_clone.borrow();
            let (current_x, current_y) = (event.client_x() as f64, event.client_y() as f64);

            // Calculate the change in mouse position
            let delta_x = current_x - last_x;
            let delta_y = current_y - last_y;

            // Update rotation angles (scale the deltas for smoother rotation)
            let mut rotation = rotation_clone.borrow_mut();
            rotation.0 += delta_y * 0.05; // Rotate around X-axis
            rotation.1 += delta_x * 0.05; // Rotate around Y-axis

            // Update the last mouse position
            *last_mouse_pos_clone.borrow_mut() = (current_x, current_y);

            //web_sys::console::log_1(&format!("Rotation {},{}", rotation.0, rotation.1).into());
        }
    }) as Box<dyn FnMut(_)>);
    canvas_clone
        .add_event_listener_with_callback("mousemove", on_mouse_move.as_ref().unchecked_ref())?;
    on_mouse_move.forget();

    // Clone references for the `mouseup` event
    let is_dragging_clone = is_dragging.clone();

    // Mouse up event
    let on_mouse_up = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
        *is_dragging_clone.borrow_mut() = false;
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mouseup", on_mouse_up.as_ref().unchecked_ref())?;
    on_mouse_up.forget();

    Ok(())
}

fn update_model(gl: GL) {
    // Retrieve the vertices and indices from the global storage
    let vertices = VERTICES.with(|v| v.read().unwrap().clone());
    let indices = INDICES.with(|i| i.read().unwrap().clone());
    let indices_length = indices.len() as i32;

    web_sys::console::log_1(
        &format!(
            "INFO: update_model called, indices_length={}",
            indices_length
        )
        .into(),
    );

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
}

#[wasm_bindgen]
pub fn start_rendering(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let gl: GL = canvas.get_context("webgl2")?.unwrap().dyn_into::<GL>()?;

    // Enable mouse controls
    let rotation = Rc::new(RefCell::new((0.0, 0.0)));
    enable_mouse_controls(canvas.clone(), rotation.clone())?;

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
            float ambientStrength = 0.4;
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

    // Model matrix
    let model = Matrix4::identity();

    // Projection matrix
    let projection = perspective(Deg(45.0), 1920.0 / 1080.0, 0.1, 100.0);

    let model_loc = gl
        .get_uniform_location(&program, "model")
        .ok_or("Could not get model uniform location")
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
    gl.uniform_matrix4fv_with_f32_array(Some(&proj_loc), false, &matrix4_to_array(&projection));
    gl.uniform3f(Some(&light_pos_loc), 1.2, 1.0, 2.0);
    gl.uniform3f(Some(&view_pos_loc), 0.0, 0.0, 2.0);
    gl.uniform3f(Some(&light_color_loc), 1.0, 1.0, 1.0);
    gl.uniform3f(Some(&object_color_loc), 1.0, 0.0, 0.5);

    gl.enable(GL::DEPTH_TEST);
    gl.depth_func(GL::LESS);

    // Animate the rotation
    animate_with_rotation(gl, program, rotation.clone());

    Ok(())
}

fn window() -> web_sys::Window {
    web_sys::window().expect("ERROR: no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("ERROR: should register `requestAnimationFrame` OK");
}

fn animate_with_rotation(gl: GL, program: WebGlProgram, rotation: Rc<RefCell<(f64, f64)>>) {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        // Clear the screen
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        // Check if the model is loaded
        let mut loaded = MODEL_LOADED.with(|i| i.read().unwrap().clone());
        let indices = INDICES.with(|i| i.read().unwrap().clone());
        let indices_length = indices.len() as i32;

        // If the model is not loaded, load it
        if !loaded {
            if indices_length != 0 {
                web_sys::console::log_1(&"INFO: Loading model...".into());
                update_model(gl.clone());
                MODEL_LOADED.with(|i| {
                    let mut l = i.write().unwrap();
                    *l = true;
                });
                loaded = true;
            }
        }

        if loaded {
            // Retrieve the current rotation angles
            let (x_rotation, y_rotation) = *rotation.borrow();

            // Update the view matrix based on rotation
            let view = Matrix4::look_at_rh(
                Point3::new(0.0, 0.0, 15.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::unit_y(),
            ) * Matrix4::from_angle_x(Deg(x_rotation as f32))
                * Matrix4::from_angle_y(Deg(y_rotation as f32));

            // Update the shader program with the view matrix
            gl.use_program(Some(&program));
            let view_loc = gl
                .get_uniform_location(&program, "view")
                .ok_or("Could not get view uniform location")
                .unwrap();
            gl.uniform_matrix4fv_with_f32_array(Some(&view_loc), false, &matrix4_to_array(&view));

            // Draw
            gl.draw_arrays(GL::TRIANGLES, 0, indices_length);
        }
        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
}
