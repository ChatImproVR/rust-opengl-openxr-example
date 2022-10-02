extern crate openxr as xr;
use anyhow::Result;
use glutin::{window::Window, ContextWrapper, PossiblyCurrent};
use xr::opengl::SessionCreateInfo;

/// Given a `glutin` Context, and `winit` Window, returns an appropriate SessionCreateInfo for the target OS
pub fn session_create_info<T>(
    ctx: &ContextWrapper<PossiblyCurrent, T>,
    #[allow(unused_variables)] window: &Window,
) -> Result<SessionCreateInfo> {
    #[cfg(target_os = "windows")]
    unsafe {
        use glutin::platform::windows::RawHandle;
        use glutin::platform::windows::WindowExtWindows;
        use glutin::platform::ContextTraitExt;

        let hwnd = window.hwnd();
        let h_glrc = match ctx.raw_handle() {
            RawHandle::Wgl(h) => h,
            _ => panic!("EGL not supported here"),
        };

        let h_dc = windows_sys::Win32::Graphics::Gdi::GetDC(hwnd);

        Ok(SessionCreateInfo::Windows {
            h_dc: std::mem::transmute(h_dc),
            h_glrc: std::mem::transmute(h_glrc),
        })
    }

    #[cfg(target_os = "linux")]
    unsafe {
        // See https://gitlab.freedesktop.org/monado/demos/openxr-simple-example/-/blob/master/main.c
        use std::ffi::c_void;
        use glutin_glx_sys::glx::Glx;
        let glx = Glx::load_with(|addr| ctx.get_proc_address(addr));

        let xlib = glutin_glx_sys::Xlib::open()?;

        let x_display = (xlib.XOpenDisplay)(std::ptr::null());
        let glx_drawable = glx.GetCurrentDrawable();
        let glx_context = glx.GetCurrentContext();

        Ok(SessionCreateInfo::Xlib {
            x_display: std::mem::transmute(x_display),
            visualid: 0,
            glx_fb_config: std::ptr::null::<c_void>() as _,
            glx_drawable,
            glx_context: std::mem::transmute(glx_context),
        })
    }
}
