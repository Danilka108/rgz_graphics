use gl_window_provider::Renderer;
use glam::{Mat4, Vec3};
use std::ffi::CString;

use crate::{
    array::{Array, AttribPointer, Size},
    shader_program::{ShaderProgram, ShaderProgramBuilder},
};

pub(crate) struct RgzRenderer {
    gl: gl::Gl,
    array: Array,
    program: ShaderProgram,
}

fn generate_vertices(iters_count: isize) -> Vec<f32> {
    let inclination_step = 180.0 / (iters_count as f32);
    let azimuth_step = 360.0 / (iters_count as f32);

    let radius = 1f32;
    let x_center = 0.0;
    let y_center = 0.0;
    let z_center = 0.0;

    let mut vertex_data = Vec::new();

    for angle_i in 0..iters_count {
        for height_i in 0..(iters_count + 1) {
            let angle = angle_i as f32 * 360.0 / (iters_count as f32);
            let height = (height_i as f32) * 2.0 * radius / (iters_count as f32);

            vertex_data.push(radius * angle.cos());
            vertex_data.push(radius * angle.sin());
            vertex_data.push(height);
        }
    }

    // for inclination_index in 0..(iters_count + 1) {
    //     for azimuth_index in 0..(iters_count + 1) {
    //         let inclination = inclination_index as f32 * inclination_step;
    //         let azimuth = azimuth_index as f32 * azimuth_step;
    //
    //         dbg!(inclination, azimuth);
    //         let x_initial = radius * inclination.sin() * azimuth.cos();
    //         let y_initial = radius * inclination.sin() * azimuth.sin();
    //         let z_initial = radius * inclination.cos();
    //
    //         // let x_deformed = x_center + x_initial;
    //         // let y_deformed = y_center + y_initial;
    //         // let z_deformed = z_center + z_initial - radius * (latitude / 90.0).powi(3);
    //         let x_deformed = x_center + x_initial;
    //         let y_deformed = y_center + y_initial;
    //         let z_deformed = z_center + z_initial;
    //
    //         vertex_data.push(x_deformed);
    //         vertex_data.push(y_deformed);
    //         vertex_data.push(z_deformed);
    //     }
    // }

    vertex_data
}

const S: isize = 100;

impl Renderer for RgzRenderer {
    fn new<D>(gl_display: &D) -> Self
    where
        D: glutin::prelude::GlDisplay,
    {
        let gl = gl::Gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        let vertex_shader = {
            let mut source_code = Vec::new();
            source_code.extend_from_slice(include_bytes!("vertex_shader.glsl"));
            source_code.push(b'\0');
            source_code
        };

        let fragment_shader = {
            let mut source_code = Vec::new();
            source_code.extend_from_slice(include_bytes!("fragment_shader.glsl"));
            source_code.push(b'\0');
            source_code
        };

        let program = ShaderProgramBuilder::new(gl.clone())
            .vertex_shader(&vertex_shader[..])
            .fragment_shader(&fragment_shader[..])
            .build()
            .unwrap();

        program.use_program();

        let v = generate_vertices(S);
        let array = Array::new(gl.clone(), v);
        array.use_array();

        array.set_attrib_pointer(
            program.get_location_of("pointPos"),
            AttribPointer {
                size: Size::Three,
                stride: 0,
                offset: 0,
                ty: gl::FLOAT,
            },
        );

        Self { gl, array, program }
    }

    fn draw(&mut self) {
        unsafe {
            self.gl.ClearColor(0.0, 0.0, 0.0, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
        }

        self.program.use_program();

        self.program.set_uniform_mat4(
            "uRotation",
            Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2 / 3.0).to_cols_array(),
        );
        self.program.set_uniform_mat4(
            "uScale",
            Mat4::from_scale(Vec3::new(0.4, 0.4, 0.4)).to_cols_array(),
        );

        self.array.use_array();

        unsafe {
            self.gl.DrawArrays(gl::POINTS, 0, (S * S) as i32);
        }
    }

    fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}
