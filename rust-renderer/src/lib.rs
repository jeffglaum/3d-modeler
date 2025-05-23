mod file;
mod global;
mod input;
mod matrix;
mod shader;
mod vao;
mod vbo;

use cgmath::{perspective, Deg, Matrix4, Point3, SquareMatrix, Transform, Vector3};
use global::DRAW_WIREFRAME;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::window;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as GL, WebGlProgram};

use crate::global::{Vertex, INDICES, VERTICES};
use crate::input::enable_mouse_controls;
use crate::matrix::matrix4_to_array;
use crate::shader::{compile_shader, link_program};
use crate::vao::VertexArray;
use crate::vbo::Buffer;

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

#[wasm_bindgen]
pub fn start_rendering(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let gl: GL = canvas.get_context("webgl2")?.unwrap().dyn_into::<GL>()?;

    // Enable mouse controls
    let zoom = Rc::new(RefCell::new(15.0));
    let rotation = Rc::new(RefCell::new((0.0, 0.0)));
    let center_pos = Rc::new(RefCell::new((0.0, 0.0, 0.0)));
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
            float ambientStrength = 0.2;
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
    let projection = perspective(
        Deg(45.0),
        canvas.width() as f32 / canvas.height() as f32,
        0.1,
        100.0,
    );

    let model_loc = gl
        .get_uniform_location(&program, "model")
        .ok_or("ERROR: could not get model uniform location")
        .unwrap();
    let proj_loc = gl
        .get_uniform_location(&program, "projection")
        .ok_or("ERROR: could not get projection uniform location")
        .unwrap();
    let light_pos_loc = gl
        .get_uniform_location(&program, "lightPos")
        .ok_or("ERROR: could not get lightPos uniform location")
        .unwrap();
    let view_pos_loc = gl
        .get_uniform_location(&program, "viewPos")
        .ok_or("ERROR: could not get viewPos uniform location")
        .unwrap();
    let light_color_loc = gl
        .get_uniform_location(&program, "lightColor")
        .ok_or("ERROR: could not get lightColor uniform location")
        .unwrap();
    let object_color_loc = gl
        .get_uniform_location(&program, "objectColor")
        .ok_or("ERROR: could not get objectColor uniform location")
        .unwrap();

    // Assign shader variable data
    gl.uniform_matrix4fv_with_f32_array(Some(&model_loc), false, &matrix4_to_array(&model));
    gl.uniform_matrix4fv_with_f32_array(Some(&proj_loc), false, &matrix4_to_array(&projection));
    gl.uniform3f(Some(&light_pos_loc), 0.0, 5.0, 5.0);
    gl.uniform3f(Some(&view_pos_loc), 0.0, 0.0, 2.0);
    gl.uniform3f(Some(&light_color_loc), 1.0, 1.0, 1.0);
    gl.uniform3f(Some(&object_color_loc), 1.0, 0.0, 0.5);

    gl.enable(GL::DEPTH_TEST);
    gl.depth_func(GL::LESS);

    // Clear the screen
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

    // Add mouse swipe listener
    let gl_clone = gl.clone();
    let program_clone = program.clone();
    let center_clone = center_pos.clone();
    let rotation_clone = rotation.clone();
    let zoom_clone = zoom.clone();

    let key_handler = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        if event.shift_key() && event.key() == "ArrowUp" {
            // Handle Shift + ArrowUp
            let mut zoom = zoom.borrow_mut();
            *zoom -= 0.5;
            *zoom = (*zoom as f32).max(1.0).min(50.0);
        } else if event.shift_key() && event.key() == "ArrowDown" {
            // Handle Shift + ArrowDown
            let mut zoom = zoom.borrow_mut();
            *zoom += 0.5;
            *zoom = (*zoom as f32).max(1.0).min(50.0);
        } else {
            match event.key().as_str() {
                "ArrowLeft" => {
                    let mut center_pos = center_pos.borrow_mut();
                    center_pos.0 -= 0.5;
                }
                "ArrowRight" => {
                    let mut center_pos = center_pos.borrow_mut();
                    center_pos.0 += 0.5;
                }
                "ArrowUp" => {
                    let mut center_pos = center_pos.borrow_mut();
                    center_pos.1 += 0.5;
                }
                "ArrowDown" => {
                    let mut center_pos = center_pos.borrow_mut();
                    center_pos.1 -= 0.5;
                }
                _ => {}
            }
        }

        draw_model(
            gl.clone(),
            program.clone(),
            center_pos.clone(),
            rotation.clone(),
            zoom.clone(),
        );
    }) as Box<dyn FnMut(_)>);

    let mouse_handler = Closure::wrap(Box::new(move || {
        draw_model(
            gl_clone.clone(),
            program_clone.clone(),
            center_clone.clone(),
            rotation_clone.clone(),
            zoom_clone.clone(),
        );
    }) as Box<dyn FnMut()>);

    canvas.add_event_listener_with_callback("mousemove", mouse_handler.as_ref().unchecked_ref())?;
    window()
        .unwrap()
        .add_event_listener_with_callback("keydown", key_handler.as_ref().unchecked_ref())?;
    mouse_handler.forget();
    key_handler.forget();

    Ok(())
}

#[wasm_bindgen]
pub fn toggle_wireframe() {
    DRAW_WIREFRAME.with(|v| {
        let mut draw_wireframe = v.write().unwrap();
        *draw_wireframe = !*draw_wireframe;
        *draw_wireframe
    });
}

fn draw_model(
    gl: GL,
    program: WebGlProgram,
    center: Rc<RefCell<(f32, f32, f32)>>,
    rotation: Rc<RefCell<(f64, f64)>>,
    zoom: Rc<RefCell<f32>>,
) {
    // Clear the screen
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

    // Retrieve the current rotation angles
    let (x_rotation, y_rotation) = *rotation.borrow();

    // Update the view matrix based on rotation
    let view = Matrix4::look_at_rh(
        Point3::new(0.0, 0.0, *zoom.borrow()),
        Point3::new(center.borrow().0, center.borrow().1, center.borrow().2),
        Vector3::unit_y(),
    ) * Matrix4::from_angle_x(Deg(x_rotation as f32))
        * Matrix4::from_angle_y(Deg(y_rotation as f32));

    // Update the shader program with the view matrix
    gl.use_program(Some(&program));
    let view_loc = gl
        .get_uniform_location(&program, "view")
        .ok_or("ERROR: could not get view uniform location")
        .unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&view_loc), false, &matrix4_to_array(&view));

    // Calculate rotated lightPos and viewPos
    let light_pos = Vector3::new(0.0, 5.0, 5.0);
    let view_pos = Vector3::new(0.0, 0.0, 2.0);

    let rotation_matrix = Matrix4::from_angle_x(Deg(x_rotation as f32))
        * Matrix4::from_angle_y(Deg(y_rotation as f32))
            .invert()
            .unwrap();

    let rotated_light_pos = rotation_matrix.transform_vector(light_pos);
    let rotated_view_pos = rotation_matrix.transform_vector(view_pos);

    // Update lightPos and viewPos in the shader
    let light_pos_loc = gl
        .get_uniform_location(&program, "lightPos")
        .ok_or("ERROR: could not get lightPos uniform location")
        .unwrap();
    let view_pos_loc = gl
        .get_uniform_location(&program, "viewPos")
        .ok_or("ERROR: could not get viewPos uniform location")
        .unwrap();

    gl.uniform3f(
        Some(&light_pos_loc),
        rotated_light_pos.x,
        rotated_light_pos.y,
        rotated_light_pos.z,
    );
    gl.uniform3f(
        Some(&view_pos_loc),
        rotated_view_pos.x,
        rotated_view_pos.y,
        rotated_view_pos.z,
    );

    // Draw
    let indices = INDICES.with(|i| i.read().unwrap().clone());
    let indices_length = indices.len() as i32;
    let draw_wireframe = DRAW_WIREFRAME.with(|v| v.read().unwrap().clone());
    if draw_wireframe {
        gl.draw_arrays(GL::LINES, 0, indices_length);
    } else {
        gl.draw_arrays(GL::TRIANGLES, 0, indices_length);
    }
}

pub fn update_model(gl: GL) {
    // Retrieve the vertices and indices from the global storage
    let vertices = VERTICES.with(|v| v.read().unwrap().clone());
    let indices = INDICES.with(|i| i.read().unwrap().clone());

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
