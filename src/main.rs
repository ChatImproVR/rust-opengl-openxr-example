extern crate openxr as xr;
use core::ffi::c_int;
use std::ffi::c_void;

use anyhow::{Context, Result};
use glow::HasContext;
use glutin::platform::ContextTraitExt;
use glutin_glx_sys::glx::Glx;

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

    let glutin_ctx = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window_builder, &event_loop)?
        .make_current()
        .unwrap();

    let gl = glow::Context::from_loader_function(|s| glutin_ctx.get_proc_address(s) as *const _);

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
                glutin_ctx.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                gl.clear(glow::COLOR_BUFFER_BIT);
                gl.draw_arrays(glow::TRIANGLES, 0, 3);
                glutin_ctx.swap_buffers().unwrap();
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    glutin_ctx.resize(*physical_size);
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
    let xr_environment_blend_mode =
        xr_instance.enumerate_environment_blend_modes(xr_system, VIEW_TYPE)?[0];

    let xr_opengl_requirements = xr_instance.graphics_requirements::<xr::OpenGL>(xr_system)?;

    // Create window
    let event_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title("Hello world!")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));

    let windowed_context = glutin::ContextBuilder::new()
        .build_windowed(window_builder, &event_loop)
        .unwrap();

    let session_create_info;

    #[cfg(target_os = "windows")]
    {
        use glutin::platform::windows::RawHandle;
        let hwnd = window.hwnd();
        let raw_handle = window.raw_handle();
        let hglrc = match RawHandle {
            RawHandle::Wgl(h) => h,
            _ => panic!("EGL not supported here"),
        };

        let hdc = todo!();

        session_create_info = xr::opengl::SessionCreateInfo::Wgl {
            h_dc: hdc,
            h_glrc: hglrc,
        };
    }

    #[cfg(target_os = "linux")]
    {
        use glutin::platform::unix::RawHandle;
        use glutin::platform::unix::WindowExtUnix;

        let (ctx, window) = windowed_context.split();
        let ctx = ctx.make_current().unwrap();
        let glx = Glx::load_with(|addr| ctx.get_proc_address(addr));

        let xlib = glutin_glx_sys::Xlib::open()?;

        let x_display = (xlib.XOpenDisplay)(std::ptr::null());
        let glx_drawable = glx.GetCurrentDrawable();
        let glx_context = glx.GetCurrentContext();

        session_create_info = xr::opengl::SessionCreateInfo::Xlib {
            x_display: std::mem::transmute(x_display),
            visualid: 0,
            glx_fb_config: std::ptr::null::<c_void>() as _,
            glx_drawable,
            glx_context: std::mem::transmute(glx_context),
        };
    }

    let xr_session = xr_instance.create_session::<xr::OpenGL>(xr_system, &session_create_info)?;

    Ok(())
}
