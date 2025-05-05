use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlShader};

#[wasm_bindgen]
pub fn start_rendering(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let gl: WebGl2RenderingContext = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;

    // Shaders
    let vert_shader = compile_shader(
        &gl,
        WebGl2RenderingContext::VERTEX_SHADER,
        r#"#version 300 es
        layout(location = 0) in vec2 a_position;
        void main() {
            gl_Position = vec4(a_position, 0.0, 1.0);
        }"#,
    )?;

    let frag_shader = compile_shader(
        &gl,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r#"#version 300 es
        precision mediump float;
        out vec4 color;
        void main() {
            color = vec4(0.0, 1.0, 0.0, 1.0); // Green
        }"#,
    )?;

    let program = link_program(&gl, &vert_shader, &frag_shader)?;
    gl.use_program(Some(&program));

    // Line vertices
    let vertices: [f32; 4] = [-0.5, -0.5, 0.5, 0.5];

    let buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

    // Pass data to buffer
    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(0);

    // Draw
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    gl.draw_arrays(WebGl2RenderingContext::LINES, 0, 2);

    Ok(())
}

fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl.create_shader(shader_type).ok_or("Unable to create shader")?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl.get_shader_info_log(&shader).unwrap_or_default())
    }
}

fn link_program(
    gl: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl.create_program().ok_or("Unable to create program")?;
    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl.get_program_info_log(&program).unwrap_or_default())
    }
}
