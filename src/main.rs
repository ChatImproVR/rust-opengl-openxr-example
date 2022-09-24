extern crate openxr as xr;
use anyhow::{Ok, Result};
use glow::HasContext;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let use_vr = args.next().is_some();

    unsafe {
        if use_vr {
            vr_main()?;
        } else {
            desktop_main()?;
        }
    }

    Ok(())
}

unsafe fn desktop_main() -> Result<()> {
    let event_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title("Hello triangle!")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));

    let window = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window_builder, &event_loop)?
        .make_current()
        .unwrap();

    let gl = glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);

    let shader_version = "#version 410";

    let vertex_array = gl
        .create_vertex_array()
        .expect("Cannot create vertex array");
    gl.bind_vertex_array(Some(vertex_array));

    let program = gl.create_program().expect("Cannot create program");

    let (vertex_shader_source, fragment_shader_source) = (
        r#"const vec2 verts[3] = vec2[3](
                vec2(0.5f, 1.0f),
                vec2(0.0f, 0.0f),
                vec2(1.0f, 0.0f)
            );
            out vec2 vert;
            void main() {
                vert = verts[gl_VertexID];
                gl_Position = vec4(vert - 0.5, 0.0, 1.0);
            }"#,
        r#"precision mediump float;
            in vec2 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 0.5, 1.0);
            }"#,
    );

    let shader_sources = [
        (glow::VERTEX_SHADER, vertex_shader_source),
        (glow::FRAGMENT_SHADER, fragment_shader_source),
    ];

    let mut shaders = Vec::with_capacity(shader_sources.len());

    for (shader_type, shader_source) in shader_sources.iter() {
        let shader = gl
            .create_shader(*shader_type)
            .expect("Cannot create shader");
        gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            panic!("{}", gl.get_shader_info_log(shader));
        }
        gl.attach_shader(program, shader);
        shaders.push(shader);
    }

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!("{}", gl.get_program_info_log(program));
    }

    for shader in shaders {
        gl.detach_shader(program, shader);
        gl.delete_shader(shader);
    }

    gl.use_program(Some(program));
    gl.clear_color(0.1, 0.2, 0.3, 1.0);

    // We handle events differently between targets

    use glutin::event::{Event, WindowEvent};
    use glutin::event_loop::ControlFlow;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::LoopDestroyed => {
                return;
            }
            Event::MainEventsCleared => {
                window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                gl.clear(glow::COLOR_BUFFER_BIT);
                gl.draw_arrays(glow::TRIANGLES, 0, 3);
                window.swap_buffers().unwrap();
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    window.resize(*physical_size);
                }
                WindowEvent::CloseRequested => {
                    gl.delete_program(program);
                    gl.delete_vertex_array(vertex_array);
                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            },
            _ => (),
        }
    });
}

unsafe fn vr_main() -> Result<()> {
    // Load OpenXR from platform-specific location
    let entry = xr::Entry::load()?;

    // Application info
    let app_info = xr::ApplicationInfo {
        application_name: "Ugly OpenGL",
        application_version: 0,
        engine_name: "Ugly Engine",
        engine_version: 0,
    };

    // Ensure we have the OpenGL extension
    let available_extensions = entry.enumerate_extensions()?;
    assert!(available_extensions.khr_opengl_enable);

    // Enable the OpenGL extension
    let mut extensions = xr::ExtensionSet::default();
    extensions.khr_opengl_enable = true;

    // Create instance
    let xr_instance = entry.create_instance(&app_info, &extensions, &[])?;
    let instance_props = xr_instance.properties().unwrap();
    println!(
        "loaded OpenXR runtime: {} {}",
        instance_props.runtime_name, instance_props.runtime_version
    );

    // Get headset system
    let xr_system = xr_instance.system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)?;

    const VIEW_TYPE: xr::ViewConfigurationType = xr::ViewConfigurationType::PRIMARY_STEREO;

    // Check what blend mode is valid for this device (opaque vs transparent displays). We'll just
    // take the first one available!
    let environment_blend_mode = xr_instance
        .enumerate_environment_blend_modes(xr_system, VIEW_TYPE)?[0];

    let requirements = xr_instance.graphics_requirements::<xr::OpenGL>(xr_system)?;

    dbg!(requirements.min_api_version_supported.major());
    dbg!(requirements.min_api_version_supported.minor());
    dbg!(requirements.min_api_version_supported.patch());

    dbg!(requirements.max_api_version_supported.major());
    dbg!(requirements.max_api_version_supported.minor());
    dbg!(requirements.max_api_version_supported.patch());

    Ok(())
}
