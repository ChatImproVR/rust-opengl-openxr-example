use glutin::{event::{WindowEvent, ElementState, MouseButton, MouseScrollDelta}, dpi::PhysicalPosition};
use nalgebra::{Matrix4, Point3, Vector3, Vector4};
use std::f32::consts::FRAC_PI_2;

/// Camera controller and parameters
#[derive(Default, Copy, Clone)]
pub struct Camera {
    proj: Perspective,
    view: ArcBall,
    control: ArcBallController,

    last_mouse_position: Option<(f64, f64)>,
    width: u32,
    height: u32,
    left_is_clicked: bool,
    right_is_clicked: bool,

}

impl Camera {
    /// Return the projection matrix of this camera
    pub fn projection(&self, width: f32, height: f32) -> Matrix4<f32> {
        self.proj.matrix(width, height)
    }

    /// Return the view matrix of this camera
    pub fn view(&self) -> Matrix4<f32> {
        self.view.matrix()
    }

    /// Handle a WindowEvent. Returns `true` if the event was consumed and `false` otherwise.
    pub fn handle_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let &PhysicalPosition { x, y } = position;
                if let Some((last_x, last_y)) = self.last_mouse_position {
                    let x_delta = (last_x - x) as f32;
                    let y_delta = (last_y - y) as f32;
                    if self.left_is_clicked {
                        self.control.pivot(&mut self.view, x_delta, y_delta);
                    } else if self.right_is_clicked {
                        self.control.pan(&mut self.view, x_delta, y_delta, 1.);
                    }
                }
                self.last_mouse_position = Some((x, y));
                true
            }
            WindowEvent::MouseInput { state, button, .. } => match button {
                MouseButton::Left => {
                    self.left_is_clicked = *state == ElementState::Pressed;
                    true
                },
                MouseButton::Right => {
                    self.right_is_clicked = *state == ElementState::Pressed;
                    true
                }
                _ => false,
            },
            WindowEvent::MouseWheel { delta, .. } => {
                if let MouseScrollDelta::LineDelta(_x, y) = delta {
                    self.view.distance += y * 0.3;
                    if self.view.distance <= 0.01 {
                        self.view.distance = 0.01;
                    }
                }
                true
            }
            WindowEvent::Resized(size) => {
                self.width = size.width;
                self.height = size.height;
                true
            }
            _ => false
        }
    }
}

/// Perspective projection parameters
#[derive(Copy, Clone)]
pub struct Perspective {
    pub fov: f32,
    pub clip_near: f32,
    pub clip_far: f32,
}

/// Arcball camera parameters
#[derive(Copy, Clone)]
pub struct ArcBall {
    pub pivot: Point3<f32>,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
}

/// Arcball camera controller parameters
#[derive(Copy, Clone)]
pub struct ArcBallController {
    pub pan_sensitivity: f32,
    pub swivel_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub closest_zoom: f32,
}

impl Perspective {
    pub fn matrix(&self, width: f32, height: f32) -> Matrix4<f32> {
        Matrix4::new_perspective(width / height, self.fov, self.clip_near, self.clip_far)
    }
}

impl ArcBall {
    pub fn matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(
            &(self.pivot + self.eye()),
            &self.pivot,
            &Vector3::new(0.0, 1.0, 0.0),
        )
    }

    pub fn eye(&self) -> Vector3<f32> {
        Vector3::new(
            self.yaw.cos() * self.pitch.cos().abs(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos().abs(),
        ) * self.distance
    }
}

impl ArcBallController {
    pub fn pivot(&mut self, arcball: &mut ArcBall, delta_x: f32, delta_y: f32) {
        arcball.yaw += delta_x * self.swivel_sensitivity;
        arcball.pitch += delta_y * self.swivel_sensitivity;

        arcball.pitch = arcball.pitch.clamp(-FRAC_PI_2, FRAC_PI_2);
    }

    pub fn pan(&mut self, arcball: &mut ArcBall, delta_x: f32, delta_y: f32, rate_z: f32) {
        let delta = Vector4::new(
            (delta_x as f32) * arcball.distance,
            (-delta_y as f32) * arcball.distance,
            0.0,
            0.0,
        ) * self.pan_sensitivity;

        // TODO: This is dumb, just use the cross product 4head
        if let Some(inv) = arcball.matrix().try_inverse() {
            let mut delta = (inv * delta).xyz();
            delta.z *= rate_z;
            arcball.pivot += delta;
        } else {
            eprintln!("Failed to invert camera matrix!");
        }
    }

    pub fn zoom(&mut self, arcball: &mut ArcBall, delta: f32) {
        arcball.distance += delta * self.zoom_sensitivity.powf(2.) * arcball.distance;
        arcball.distance = arcball.distance.max(self.closest_zoom);
    }
}

impl Default for ArcBall {
    fn default() -> Self {
        Self {
            pivot: Point3::origin(),
            pitch: 0.3,
            yaw: -1.92,
            distance: 10.,
        }
    }
}

impl Default for Perspective {
    fn default() -> Self {
        Self {
            fov: 45.0f32.to_radians(),
            clip_near: 0.0001,
            clip_far: 20_000.0,
        }
    }
}

impl Default for ArcBallController {
    fn default() -> Self {
        Self {
            pan_sensitivity: 0.0015,
            swivel_sensitivity: 0.005,
            zoom_sensitivity: 0.04,
            closest_zoom: 0.01,
        }
    }
}
