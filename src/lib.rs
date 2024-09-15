use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Access the document and the canvas element
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    // Get the WebGL rendering context
    let gl: WebGlRenderingContext = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    // Set the background color to black and clear the color and depth buffers
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);

    // Vertex shader source code
    let vert_shader_source = r#"
        attribute vec4 a_position;
        uniform mat4 u_model_view_matrix;
        uniform mat4 u_projection_matrix;
        void main() {
            gl_Position = u_projection_matrix * u_model_view_matrix * a_position;
        }
    "#;

    // Fragment shader source code
    let frag_shader_source = r#"
        precision mediump float;
        void main() {
            gl_FragColor = vec4(0.0, 1.0, 0.0, 1.0); // Green color
        }
    "#;

    // Compile shaders
    let vert_shader = compile_shader(
        &gl,
        WebGlRenderingContext::VERTEX_SHADER,
        vert_shader_source,
    )?;
    let frag_shader = compile_shader(
        &gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        frag_shader_source,
    )?;

    // Link shaders into a program
    let program = link_program(&gl, &vert_shader, &frag_shader)?;
    gl.use_program(Some(&program));

    // Define the cube vertices
    let vertices: [f32; 72] = [
        -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, -1.0, 1.0, 1.0,
        -1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0,
        1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, -1.0,
        -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, -1.0, 1.0,
        1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0,
    ];

    // Create a buffer and bind it as the current array buffer
    let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

    // Transfer the vertex data to the buffer
    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }

    // Get the attribute location, enable it, and specify how to pull data out of the buffer
    let position_attrib_location = gl.get_attrib_location(&program, "a_position") as u32;
    gl.vertex_attrib_pointer_with_i32(
        position_attrib_location,
        3,
        WebGlRenderingContext::FLOAT,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(position_attrib_location);

    // Set up the model-view and projection matrices
    let model_view_matrix_location = gl
        .get_uniform_location(&program, "u_model_view_matrix")
        .unwrap();
    let projection_matrix_location = gl
        .get_uniform_location(&program, "u_projection_matrix")
        .unwrap();

    let mut angle_x: f32 = 0.0;
    let mut angle_y: f32 = 0.0;

    // Create a closure to update and draw the scene
    let f = Rc::new(RefCell::new(<Option<Closure<dyn FnMut()>>>::None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // Update rotation angles
        angle_x += 0.01;
        angle_y += 0.02;
        let sin_angle_x = angle_x.sin();
        let cos_angle_x = angle_x.cos();
        let sin_angle_y = angle_y.sin();
        let cos_angle_y = angle_y.cos();

        // Create the model-view matrix for rotating the cube
        let model_view_matrix: [f32; 16] = [
            cos_angle_y,
            0.0,
            sin_angle_y,
            0.0,
            sin_angle_x * sin_angle_y,
            cos_angle_x,
            -sin_angle_x * cos_angle_y,
            0.0,
            -cos_angle_x * sin_angle_y,
            sin_angle_x,
            cos_angle_x * cos_angle_y,
            0.0,
            0.0,
            0.0,
            -6.0,
            1.0,
        ];

        // Create a simple orthographic projection matrix
        let projection_matrix: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.0, -1.0, 0.0, 0.0, -0.02, 0.0,
        ];

        // Clear the color and depth buffers
        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);

        // Set the model-view and projection matrices in the shader
        gl.uniform_matrix4fv_with_f32_array(
            Some(&model_view_matrix_location),
            false,
            &model_view_matrix,
        );
        gl.uniform_matrix4fv_with_f32_array(
            Some(&projection_matrix_location),
            false,
            &projection_matrix,
        );

        // Draw the cube as lines
        gl.draw_arrays(WebGlRenderingContext::LINES, 0, 24);

        // Schedule the next frame
        web_sys::window()
            .unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));

    // Start the animation loop
    web_sys::window()
        .unwrap()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();

    Ok(())
}

// Function to compile a shader
fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

// Function to link shaders into a program
fn link_program(
    gl: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
