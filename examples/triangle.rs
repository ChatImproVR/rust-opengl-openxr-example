extern crate glow as gl;
extern crate openxr as xr;

use anyhow::{bail, format_err, Result};
use gl::HasContext;

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

const VERTEX_SHADER_SOURCE: &str = r#"
    #version 450
    const vec2 verts[3] = vec2[3](
        vec2(0.5f, 1.0f),
        vec2(0.0f, 0.0f),
        vec2(1.0f, 0.0f)
    );
    out vec2 vert;
    void main() {
        vert = verts[gl_VertexID];
        gl_Position = vec4(vert - 0.5, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 450
    precision mediump float;
    in vec2 vert;
    out vec4 color;
    void main() {
        color = vec4(vert, 0.5, 1.0);
    }
"#;

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

    let gl = gl::Context::from_loader_function(|s| glutin_ctx.get_proc_address(s) as *const _);

    let vertex_array = gl
        .create_vertex_array()
        .expect("Cannot create vertex array");
    gl.bind_vertex_array(Some(vertex_array));

    let program = compile_glsl_program(
        &gl,
        &[
            (gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE),
            (gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE),
        ],
    )?;

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
                gl.clear(gl::COLOR_BUFFER_BIT);
                gl.draw_arrays(gl::TRIANGLES, 0, 3);
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
    #[cfg(target_os = "linux")]
    let entry = xr::Entry::load()?;

    #[cfg(target_os = "windows")]
    let entry = xr::Entry::linked();

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

    let xr_view_configs = xr_instance.enumerate_view_configurations(xr_system)?;
    assert_eq!(xr_view_configs.len(), 1);
    let xr_view_type = xr_view_configs[0];

    let xr_views = xr_instance.enumerate_view_configuration_views(xr_system, xr_view_type)?;

    // Check what blend mode is valid for this device (opaque vs transparent displays). We'll just
    // take the first one available!
    let xr_environment_blend_mode =
        xr_instance.enumerate_environment_blend_modes(xr_system, xr_view_type)?[0];

    // TODO: Check this???
    let _xr_opengl_requirements = xr_instance.graphics_requirements::<xr::OpenGL>(xr_system)?;

    // Create window
    let event_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title("Hello world!")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0f32, 768.0));

    let windowed_context = glutin::ContextBuilder::new()
        .build_windowed(window_builder, &event_loop)
        .unwrap();

    let (ctx, window) = windowed_context.split();
    let ctx = ctx.make_current().unwrap();

    // Load OpenGL
    let gl = gl::Context::from_loader_function(|s| ctx.get_proc_address(s) as *const _);

    let session_create_info = glutin_openxr_opengl_helper::session_create_info(&ctx, &window)?;

    // Create vertex array
    let vertex_array = gl
        .create_vertex_array()
        .expect("Cannot create vertex array");
    gl.bind_vertex_array(Some(vertex_array));

    // Create session
    let (xr_session, mut xr_frame_waiter, mut xr_frame_stream) =
        xr_instance.create_session::<xr::OpenGL>(xr_system, &session_create_info)?;

    // Determine swapchain formats
    let xr_swapchain_formats = xr_session.enumerate_swapchain_formats()?;

    let color_swapchain_format = xr_swapchain_formats
        .iter()
        .copied()
        .find(|&f| f == gl::SRGB8_ALPHA8)
        .unwrap_or(xr_swapchain_formats[0]);

    /*
    let depth_swapchain_format = xr_swapchain_formats
        .iter()
        .copied()
        .find(|&f| f == glow::DEPTH_COMPONENT16)
        .expect("No suitable depth format found");
    */

    // Create color swapchain
    let mut swapchain_images = vec![];
    let mut xr_swapchains = vec![];

    // Set up swapchains and get images
    for &xr_view in &xr_views {
        let xr_swapchain_create_info = xr::SwapchainCreateInfo::<xr::OpenGL> {
            create_flags: xr::SwapchainCreateFlags::EMPTY,
            usage_flags: xr::SwapchainUsageFlags::SAMPLED
                | xr::SwapchainUsageFlags::COLOR_ATTACHMENT,
            format: color_swapchain_format,
            sample_count: xr_view.recommended_swapchain_sample_count,
            width: xr_view.recommended_image_rect_width,
            height: xr_view.recommended_image_rect_height,
            face_count: 1,
            array_size: 1,
            mip_count: 1,
        };

        let xr_swapchain = xr_session.create_swapchain(&xr_swapchain_create_info)?;

        let images = xr_swapchain.enumerate_images()?;

        swapchain_images.push(images);
        xr_swapchains.push(xr_swapchain);
    }

    // Create OpenGL framebuffers
    let mut gl_framebuffers = vec![];
    for _ in &xr_views {
        gl_framebuffers.push(
            gl.create_framebuffer()
                .map_err(|s| format_err!("Failed to create framebuffer; {}", s))?,
        );
    }

    // Compile shaders
    let gl_program = compile_glsl_program(
        &gl,
        &[
            (gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE),
            (gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE),
        ],
    )?;

    let xr_play_space =
        xr_session.create_reference_space(xr::ReferenceSpaceType::LOCAL, xr::Posef::IDENTITY)?;

    let mut xr_event_buf = xr::EventDataBuffer::default();

    'main: loop {
        // Handle OpenXR Events
        while let Some(event) = xr_instance.poll_event(&mut xr_event_buf)? {
            match event {
                xr::Event::InstanceLossPending(_) => break 'main,
                xr::Event::SessionStateChanged(delta) => {
                    match delta.state() {
                        xr::SessionState::IDLE | xr::SessionState::UNKNOWN => {
                            continue 'main;
                        }
                        //xr::SessionState::FOCUSED | xr::SessionState::SYNCHRONIZED | xr::SessionState::VISIBLE => (),
                        xr::SessionState::STOPPING => {
                            xr_session.end()?;
                            break 'main;
                        }
                        xr::SessionState::LOSS_PENDING | xr::SessionState::EXITING => {
                            // ???
                        }
                        xr::SessionState::READY => {
                            dbg!(delta.state());
                            xr_session.begin(xr_view_type)?;
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }

        // --- Wait for our turn to do head-pose dependent computation and render a frame
        let frame_state = xr_frame_waiter.wait()?;
        dbg!(frame_state);

        // Get OpenXR Views
        // TODO: Do this as close to render-time as possible!!
        let (_xr_view_state_flags, xr_view_poses) = xr_session.locate_views(
            xr_view_type,
            frame_state.predicted_display_time,
            &xr_play_space,
        )?;

        // Signal to OpenXR that we are beginning graphics work
        xr_frame_stream.begin()?;

        for view_idx in 0..xr_views.len() {
            // Acquire image
            let xr_swapchain_img_idx = xr_swapchains[view_idx].acquire_image()?;
            xr_swapchains[view_idx].wait_image(xr::Duration::from_nanos(1_000_000_000_000))?;

            // Bind framebuffer
            gl.bind_framebuffer(gl::FRAMEBUFFER, Some(gl_framebuffers[view_idx]));

            // Set scissor and viewport
            let view = xr_views[view_idx];
            let w = view.recommended_image_rect_width as i32;
            let h = view.recommended_image_rect_height as i32;
            gl.viewport(0, 0, w, h);
            gl.scissor(0, 0, w, h);

            // Set the texture as the render target
            let texture = swapchain_images[view_idx][xr_swapchain_img_idx as usize];
            let texture = unsafe { gl::Context::create_texture_from_gl_name(texture) };

            gl.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                Some(texture),
                0,
            );

            // Draw!
            gl.use_program(Some(gl_program));
            gl.clear_color(0.1, 0.2, 0.3, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT);
            gl.draw_arrays(gl::TRIANGLES, 0, 3);
        
            // Unbind framebuffer
            gl.bind_framebuffer(gl::FRAMEBUFFER, None);

            // Release image
            xr_swapchains[view_idx].release_image()?;
        }

        // Set up projection views
        let mut xr_projection_views = vec![];
        for view_idx in 0..xr_views.len() {
            // Set up projection view
            let xr_sub_image = xr::SwapchainSubImage::<xr::OpenGL>::new()
                .swapchain(&xr_swapchains[view_idx])
                .image_array_index(0)
                .image_rect(xr::Rect2Di {
                    offset: xr::Offset2Di { x: 0, y: 0 },
                    extent: xr::Extent2Di {
                        width: xr_views[view_idx].recommended_image_rect_width as i32,
                        height: xr_views[view_idx].recommended_image_rect_height as i32,
                    },
                });

            let xr_proj_view =
                xr::CompositionLayerProjectionView::<xr::OpenGL>::new()
                    .pose(xr_view_poses[view_idx].pose)
                    .fov(xr_view_poses[view_idx].fov)
                    .sub_image(xr_sub_image);

            xr_projection_views.push(xr_proj_view);
        }


        let layers = xr::CompositionLayerProjection::new()
            .space(&xr_play_space)
            .views(&xr_projection_views);

        xr_frame_stream.end(
            frame_state.predicted_display_time,
            xr_environment_blend_mode,
            &[&layers],
        )?;
    }

    Ok(())
}

/// Compiles (*_SHADER, <source>) into a shader program for OpenGL
fn compile_glsl_program(gl: &gl::Context, sources: &[(u32, &str)]) -> Result<gl::Program> {
    // Compile default shaders
    unsafe {
        let program = gl.create_program().expect("Cannot create program");

        let mut shaders = vec![];

        for (stage, shader_source) in sources {
            let shader = gl.create_shader(*stage).expect("Cannot create shader");

            gl.shader_source(shader, shader_source);

            gl.compile_shader(shader);

            if !gl.get_shader_compile_status(shader) {
                bail!(
                    "Failed to compile shader;\n{}",
                    gl.get_shader_info_log(shader)
                );
            }

            gl.attach_shader(program, shader);

            shaders.push(shader);
        }

        gl.link_program(program);

        if !gl.get_program_link_status(program) {
            bail!("{}", gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        Ok(program)
    }
}
