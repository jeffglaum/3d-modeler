use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as GL, WebGlProgram, WebGlShader, WebGlUniformLocation, console};
use std::mem;
use cgmath::{perspective, Deg, InnerSpace, Matrix4, Point3, SquareMatrix, Vector3};
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;


fn compile_shader(
    gl: &GL,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl.create_shader(shader_type).ok_or("Unable to create shader")?;
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
        matrix.x.x, matrix.x.y, matrix.x.z, matrix.x.w,
        matrix.y.x, matrix.y.y, matrix.y.z, matrix.y.w,
        matrix.z.x, matrix.z.y, matrix.z.z, matrix.z.w,
        matrix.w.x, matrix.w.y, matrix.w.z, matrix.w.w,
    ]
}

#[wasm_bindgen]
pub fn handle_mouse_click(x: f64, y: f64) {
    web_sys::console::log_1(&format!("INFO: mouse click at: {}, {}", x, y).into());
}

#[wasm_bindgen]
pub fn start_rendering(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let gl: GL = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<GL>()?;

    gl.clear_color(0.0, 0.0, 0.0, 1.0);

    // Triangle vertex data
    let vertices: [f32; 18] = [
        // positions         // normals
        -0.5, -0.5, 0.0,     0.0, 0.0, 1.0,
         0.5, -0.5, 0.0,     0.0, 0.0, 1.0,
         0.0,  0.5, 0.0,     0.0, 0.0, 1.0,
    ];

    // Create VAO
    let vao = gl.create_vertex_array().ok_or("Could not create VAO")?;
    gl.bind_vertex_array(Some(&vao));

    // Create VBO
    let vbo = gl.create_buffer().ok_or("Could not create VBO")?;
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));

    // Transfer vertex data
    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vert_array, GL::STATIC_DRAW);
    }

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
    )?;

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
    )?;

    let program = link_program(&gl, &vert_shader, &frag_shader)?;
    gl.use_program(Some(&program));

    // Attribute location 0: position
    gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 6 * mem::size_of::<f32>() as i32, 0);
    gl.enable_vertex_attrib_array(0);

    // Attribute location 1: normals
    gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, 6 * mem::size_of::<f32>() as i32, (3 * mem::size_of::<f32>()) as i32);
    gl.enable_vertex_attrib_array(1);

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
    ).normalize();

    // Adjust the model matrix by rotating it around the randomly-chosen vector
    let model = Matrix4::identity();

    let model_loc = gl.get_uniform_location(&program, "model").ok_or("Could not get model uniform location")?;
    let view_loc = gl.get_uniform_location(&program, "view").ok_or("Could not get view uniform location")?;
    let proj_loc = gl.get_uniform_location(&program, "projection").ok_or("Could not get projection uniform location")?;
    let light_pos_loc = gl.get_uniform_location(&program, "lightPos").ok_or("Could not get lightPos uniform location")?;
    let view_pos_loc = gl.get_uniform_location(&program, "viewPos").ok_or("Could not get viewPos uniform location")?;
    let light_color_loc = gl.get_uniform_location(&program, "lightColor").ok_or("Could not get lightColor uniform location")?;
    let object_color_loc = gl.get_uniform_location(&program, "objectColor").ok_or("Could not get objectColor uniform location")?;

    // Assign shader variable data
    gl.uniform_matrix4fv_with_f32_array(Some(&model_loc), false, &matrix4_to_array(&model));
    gl.uniform_matrix4fv_with_f32_array(Some(&view_loc), false, &matrix4_to_array(&view));
    gl.uniform_matrix4fv_with_f32_array(Some(&proj_loc), false, &matrix4_to_array(&projection));
    gl.uniform3f(Some(&light_pos_loc), 1.2, 1.0, 2.0);
    gl.uniform3f(Some(&view_pos_loc), 0.0, 0.0, 2.0);
    gl.uniform3f(Some(&light_color_loc), 1.0, 1.0, 1.0);
    gl.uniform3f(Some(&object_color_loc), 0.3, 0.5, 1.0);

    // Select the triangle vao into context
    gl.bind_vertex_array(Some(&vao));

    // Animate the rotation
    animate(0.0, gl, model_loc, rotation_axis);

    Ok(())
}

fn animate(start_time: f64, gl: GL, model_loc: WebGlUniformLocation, rotation_axis: Vector3<f32>) {
    let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let closure = Closure::wrap(Box::new(move |time: f64| {

        gl.clear_color(0.0, 0.0, 0.0, 1.0);

        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        let angle = Deg(((time - start_time) / 1000.0) as f32 * 45.0); // 45 degrees per second
        let model = Matrix4::from_axis_angle(rotation_axis, angle);
        gl.uniform_matrix4fv_with_f32_array(Some(&model_loc), false, &matrix4_to_array(&model));

        // Clear and draw
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT);
        gl.draw_arrays(GL::TRIANGLES, 0, 3);

        // Schedule next frame
        web_sys::window()
            .unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut(f64)>);

    *g.borrow_mut() = Some(closure);
    web_sys::window().unwrap()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}
