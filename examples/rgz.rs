use gl_window_provider::Renderer;
use std::ffi::{CStr, CString};

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
    z: f64,
}

pub(crate) struct RgzRenderer {
    gl: gl::Gl,
}

fn main() {
    dbg!(generate_points(100));
}

fn generate_points(iters_count: isize) -> Vec<Point> {
    let step = 180.0 / iters_count as f64;

    let radius = 1f64;
    let x_center = 0.0;
    let y_center = 0.0;
    let z_center = 0.0;

    let mut vertex_data = Vec::new();

    for latitude_index in 0..iters_count {
        for longitude_index in 0..iters_count {
            let latitude = (latitude_index - iters_count / 2) as f64 * step;
            let longitude = longitude_index as f64 * step;
            let x_initial = radius * latitude.cos() * longitude.sin();
            let y_initial = radius * latitude.cos() * longitude.cos();
            let z_initial = radius * latitude.sin();

            let x_deformed = x_center + x_initial;
            let y_deformed = y_center + y_initial;
            let z_deformed = z_center + z_initial - radius * (latitude / 90.0).powi(3);

            vertex_data.push(Point {
                x: x_deformed,
                y: y_deformed,
                z: z_deformed,
            });
        }
    }

    vertex_data
}

impl Renderer for RgzRenderer {
    fn new<D>(gl_display: &D) -> Self
    where
        D: glutin::prelude::GlDisplay,
    {
        let gl = gl::Gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        // todo!()
        Self { gl }
    }

    fn draw(&mut self) {
        unsafe {
            self.gl.ClearColor(0.5, 0.8, 0.1, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
        }
        // todo!()
    }

    fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}
