use gl_window_provider::Renderer;
use glam::{Mat4, Vec3};
use std::ffi::CString;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton},
};

use crate::{
    array::{AttribPointer, Size, VerticesArray},
    shader_program::{ShaderProgram, ShaderProgramBuilder},
};

pub(crate) struct RgzRenderer {
    gl: gl::Gl,
    figure_slices_count: u32,
    figure_radius: f32,
    vertices_array: VerticesArray,
    mesh_program: ShaderProgram,
    polygon_program: ShaderProgram,
    left_mouse_btn_pressed: bool,
    last_cursor_pos: Option<PhysicalPosition<f64>>,
    camera_polar_angle: f32,
    camera_azimuthal_angle: f32,
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

        let figure_slices_count = 100;
        let radius = 1f32;

        let mut angles_indices = Vec::new();

        for polar_index in 0..(figure_slices_count) {
            for azimuth_index in 0..figure_slices_count {
                angles_indices.push(polar_index);
                angles_indices.push(azimuth_index);
            }
        }

        let mesh_program = ShaderProgramBuilder::new(gl.clone())
            .vertex_shader(include_bytes!("mesh_program/vertex_shader.glsl"))
            .geometry_shader(include_bytes!("mesh_program/geometry_shader.glsl"))
            .fragment_shader(include_bytes!("mesh_program/fragment_shader.glsl"))
            .build()
            .unwrap();

        let polygon_program = ShaderProgramBuilder::new(gl.clone())
            .vertex_shader(include_bytes!("polygon_program/vertex_shader.glsl"))
            .geometry_shader(include_bytes!("polygon_program/geometry_shader.glsl"))
            .fragment_shader(include_bytes!("polygon_program/fragment_shader.glsl"))
            .build()
            .unwrap();

        mesh_program.use_program();
        polygon_program.use_program();

        let vertices_array = VerticesArray::new(gl.clone(), angles_indices);
        vertices_array.use_array();

        vertices_array.set_attrib_int_pointer(
            mesh_program.attrib_location_of("iPolarAngleIndex"),
            AttribPointer {
                size: Size::One,
                stride: 2 * std::mem::size_of::<u32>(),
                offset: 0,
                ty: gl::UNSIGNED_INT,
            },
        );
        vertices_array.set_attrib_int_pointer(
            mesh_program.attrib_location_of("iAzimuthAngleIndex"),
            AttribPointer {
                size: Size::One,
                stride: 2 * std::mem::size_of::<u32>(),
                offset: std::mem::size_of::<u32>(),
                ty: gl::UNSIGNED_INT,
            },
        );

        Self {
            gl,
            figure_radius: radius,
            figure_slices_count,
            vertices_array,
            mesh_program,
            polygon_program,
            left_mouse_btn_pressed: false,
            last_cursor_pos: None,
            camera_polar_angle: 0.0,
            camera_azimuthal_angle: 0.0,
        }
    }

    fn mouse_input_hook(&mut self, state: ElementState, button: MouseButton) {
        self.left_mouse_btn_pressed =
            matches!(state, ElementState::Pressed) && matches!(button, MouseButton::Left);
    }

    // fn mouse_wheel_hook(&mut self, delta: MouseScrollDelta, phase: TouchPhase) {}

    // fn keyboard_input_hook(&mut self, input: KeyboardInput) {}

    fn cursor_move_hook(&mut self, next_pos: PhysicalPosition<f64>) {
        if let Some(prev_pos) = self.last_cursor_pos {
            self.update_rotation_angles(prev_pos, next_pos);
        }

        self.last_cursor_pos = Some(next_pos);
    }

    // fn cursor_enter_hook(&mut self) {}

    // fn cursor_left_hook(&mut self) {}

    fn draw(&mut self) {
        let camera_pos_matrix = Mat4::from_rotation_x(self.camera_polar_angle)
            * Mat4::from_rotation_y(self.camera_azimuthal_angle);
        let scale_matrix = Mat4::from_scale(Vec3::new(0.4, 0.4, 0.4));

        unsafe {
            self.gl.Enable(gl::DEPTH_TEST);
            self.gl.ClearColor(0.0, 0.0, 0.0, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        self.vertices_array.use_array();

        self.mesh_program.use_program();
        self.mesh_program
            .set_uniform_mat4("uCameraPos", camera_pos_matrix.to_cols_array());
        self.mesh_program
            .set_uniform_mat4("uScale", scale_matrix.to_cols_array());
        self.mesh_program
            .set_uniform_f32("uRadius", self.figure_radius);
        self.mesh_program
            .set_uniform_u32("uStepsCount", self.figure_slices_count);

        unsafe {
            // self.gl
            //     .DrawArrays(gl::POINTS, 0, self.vertices_array.len() as i32);
        }

        self.polygon_program.use_program();
        self.polygon_program
            .set_uniform_mat4("uCameraPos", camera_pos_matrix.to_cols_array());
        self.polygon_program
            .set_uniform_mat4("uScale", scale_matrix.to_cols_array());
        self.polygon_program
            .set_uniform_f32("uRadius", self.figure_radius);
        self.polygon_program
            .set_uniform_u32("uStepsCount", self.figure_slices_count);

        unsafe {
            self.gl
                .DrawArrays(gl::POINTS, 0, self.vertices_array.len() as i32);
        }
    }

    fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}

impl RgzRenderer {
    const DELTA_X_INTO_DELTA_ANGLE_FACTOR: f32 = std::f32::consts::FRAC_PI_2 / (1920.0 / 2.0);
    const DELTA_Y_INTO_DELTA_ANGLE_FACTOR: f32 = std::f32::consts::FRAC_PI_2 / (1280.0 / 2.0);

    fn update_rotation_angles(
        &mut self,
        prev_pos: PhysicalPosition<f64>,
        next_pos: PhysicalPosition<f64>,
    ) {
        if !self.left_mouse_btn_pressed {
            return;
        }

        let delta_x = next_pos.x - prev_pos.x;
        let delta_y = next_pos.y - prev_pos.y;

        let delta_polar_angle = delta_y as f32 * Self::DELTA_Y_INTO_DELTA_ANGLE_FACTOR;
        let delta_azimuthal_angle = delta_x as f32 * Self::DELTA_X_INTO_DELTA_ANGLE_FACTOR;

        let polar_angle = self.camera_polar_angle + delta_polar_angle;
        let polar_angle = if polar_angle > std::f32::consts::FRAC_PI_2 {
            std::f32::consts::FRAC_PI_2
        } else if polar_angle < -std::f32::consts::FRAC_PI_2 {
            -std::f32::consts::FRAC_PI_2
        } else {
            polar_angle
        };

        let azimuthal_angle =
            (self.camera_azimuthal_angle + delta_azimuthal_angle) % (std::f32::consts::PI * 2.0);

        self.camera_polar_angle = polar_angle;
        self.camera_azimuthal_angle = azimuthal_angle;
    }
}
