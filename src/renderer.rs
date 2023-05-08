use gl_window_provider::Renderer;
use glam::{Mat4, Vec3};
use std::ffi::CString;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase},
};

use crate::{
    array::{AttribPointer, Size, VerticesArray},
    shader_program::{ShaderProgram, ShaderProgramBuilder},
};

pub(crate) struct RgzRenderer {
    gl: gl::Gl,

    polygon_slices_count: u32,
    mesh_slices_count: u32,
    figure_radius: f32,

    mesh_array: VerticesArray,
    mesh_program: ShaderProgram,

    polygon_array: VerticesArray,
    polygon_program: ShaderProgram,

    left_mouse_btn_pressed: bool,
    last_cursor_pos: Option<PhysicalPosition<f64>>,
    cursor_left: bool,

    camera_polar_angle: f32,
    camera_azimuthal_angle: f32,
    camera_zoom: f32,
}

impl RgzRenderer {
    const POLYGON_SLICES_COUNT: u32 = 1000;
    const MESH_SLICES_COUNT: u32 = 100;
    const FIGURE_RADIUS: f32 = 1.0;

    const ZOOM_FACTOR: f32 = 1.0 / 25.0;
    const ZOOM_MIN: f32 = -0.02;
    const ZOOM_DEFAULT: f32 = -0.5;
    const ZOOM_MAX: f32 = -3.00;

    const DELTA_X_INTO_DELTA_ANGLE_FACTOR: f32 = std::f32::consts::FRAC_PI_2 / (1920.0 / 2.0);
    const DELTA_Y_INTO_DELTA_ANGLE_FACTOR: f32 = std::f32::consts::FRAC_PI_2 / (1280.0 / 2.0);
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

        let polygon_slices_count = Self::POLYGON_SLICES_COUNT;
        let mesh_slices_count = Self::MESH_SLICES_COUNT;
        let radius = Self::FIGURE_RADIUS;

        let mut polygon_angles = Vec::new();
        let mut mesh_angles = Vec::new();

        for polar_index in 0..polygon_slices_count {
            for azimuth_index in 0..polygon_slices_count {
                polygon_angles.push(polar_index);
                polygon_angles.push(azimuth_index);
            }
        }

        for polar_index in 0..mesh_slices_count {
            for azimuth_index in 0..mesh_slices_count {
                mesh_angles.push(polar_index);
                mesh_angles.push(azimuth_index);
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

        let polygon_array = VerticesArray::new(gl.clone(), polygon_angles);
        let mesh_array = VerticesArray::new(gl.clone(), mesh_angles);

        polygon_array.use_array();
        mesh_array.use_array();

        polygon_array.set_attrib_int_pointer(
            polygon_program.attrib_location_of("iPolarAngleIndex"),
            AttribPointer {
                size: Size::One,
                stride: 2 * std::mem::size_of::<u32>(),
                offset: 0,
                ty: gl::UNSIGNED_INT,
            },
        );
        mesh_array.set_attrib_int_pointer(
            mesh_program.attrib_location_of("iPolarAngleIndex"),
            AttribPointer {
                size: Size::One,
                stride: 2 * std::mem::size_of::<u32>(),
                offset: 0,
                ty: gl::UNSIGNED_INT,
            },
        );

        polygon_array.set_attrib_int_pointer(
            polygon_program.attrib_location_of("iAzimuthAngleIndex"),
            AttribPointer {
                size: Size::One,
                stride: 2 * std::mem::size_of::<u32>(),
                offset: std::mem::size_of::<u32>(),
                ty: gl::UNSIGNED_INT,
            },
        );
        mesh_array.set_attrib_int_pointer(
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
            polygon_slices_count,
            mesh_slices_count,

            polygon_array,
            polygon_program,

            mesh_array,
            mesh_program,

            left_mouse_btn_pressed: false,
            last_cursor_pos: None,
            cursor_left: false,

            camera_polar_angle: 0.0,
            camera_azimuthal_angle: 0.0,
            camera_zoom: Self::ZOOM_DEFAULT,
        }
    }

    fn mouse_input_hook(&mut self, state: ElementState, button: MouseButton) {
        self.left_mouse_btn_pressed =
            matches!(state, ElementState::Pressed) && matches!(button, MouseButton::Left);
    }

    fn mouse_wheel_hook(&mut self, delta: MouseScrollDelta, phase: TouchPhase) {
        let MouseScrollDelta::LineDelta(_, vertical_delta) = delta else {
            return;
        };

        let TouchPhase::Moved = phase else {
            return;
        };

        self.camera_zoom = (self.camera_zoom + vertical_delta * Self::ZOOM_FACTOR)
            .clamp(Self::ZOOM_MAX, Self::ZOOM_MIN);
    }

    // fn keyboard_input_hook(&mut self, input: KeyboardInput) {}

    fn cursor_move_hook(&mut self, next_pos: PhysicalPosition<f64>) {
        if let Some(prev_pos) = self.last_cursor_pos {
            self.update_rotation_angles(prev_pos, next_pos);
        }

        self.last_cursor_pos = Some(next_pos);
    }

    fn cursor_enter_hook(&mut self) {
        self.cursor_left = false;
    }

    fn cursor_left_hook(&mut self) {
        self.cursor_left = true;
    }

    fn draw(&mut self) {
        let view_matrix = Mat4::from_rotation_x(self.camera_polar_angle)
            * Mat4::from_rotation_y(self.camera_azimuthal_angle);
        let model_matrix = Mat4::from_scale(Vec3::new(
            self.camera_zoom.exp(),
            self.camera_zoom.exp(),
            self.camera_zoom.exp(),
        ));

        unsafe {
            self.gl.Enable(gl::DEPTH_TEST);
            self.gl.ClearColor(0.0, 0.0, 0.0, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        self.polygon_array.use_array();

        self.polygon_program.use_program();
        self.polygon_program
            .set_uniform_vec3("uDirLight.direction", [1.0, 0.0, 0.0]);
        self.polygon_program
            .set_uniform_vec3("uDirLight.ambient", [0.0, 0.3, 0.0]);
        self.polygon_program
            .set_uniform_vec3("uDirLight.diffuse", [0.0, 0.3, 0.0]);
        self.polygon_program
            .set_uniform_vec3("uDirLight.specular", [0.0, 0.3, 0.0]);

        // emerald material
        self.polygon_program
            .set_uniform_vec3("uMaterial.ambient", [0.0215, 0.1745, 0.0215]);
        self.polygon_program
            .set_uniform_vec3("uMaterial.diffuse", [0.07568, 0.61424, 0.07568]);
        self.polygon_program
            .set_uniform_vec3("uMaterial.specular", [0.633, 0.727811, 0.633]);
        self.polygon_program
            .set_uniform_f32("uMaterial.shininess", 0.6);

        let view_pos = self.calc_view_pos();
        self.polygon_program
            .set_uniform_vec3("uViewPos", [view_pos.0, view_pos.1, view_pos.2]);

        self.polygon_program
            .set_uniform_mat4("uViewMat", view_matrix.to_cols_array());
        self.polygon_program
            .set_uniform_mat4("uModelMat", model_matrix.to_cols_array());
        self.polygon_program
            .set_uniform_f32("uRadius", self.figure_radius);
        self.polygon_program
            .set_uniform_u32("uSlicesCount", self.polygon_slices_count);

        unsafe {
            self.gl
                .DrawArrays(gl::POINTS, 0, self.polygon_array.len() as i32);
        }

        self.mesh_array.use_array();

        self.mesh_program.use_program();
        self.mesh_program
            .set_uniform_mat4("uViewMat", view_matrix.to_cols_array());
        self.mesh_program
            .set_uniform_mat4("uModelMat", model_matrix.to_cols_array());
        self.mesh_program
            .set_uniform_f32("uRadius", self.figure_radius);
        self.mesh_program
            .set_uniform_u32("uSlicesCount", self.mesh_slices_count);

        unsafe {
            self.gl
                .DrawArrays(gl::POINTS, 0, self.mesh_array.len() as i32);
        }
    }

    fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}

impl RgzRenderer {
    fn calc_view_pos(&self) -> (f32, f32, f32) {
        let radius = 1.0;

        let x = radius * self.camera_polar_angle.sin() * self.camera_azimuthal_angle.cos();
        let y = radius * self.camera_polar_angle.sin() * self.camera_azimuthal_angle.sin();
        let z = radius * self.camera_polar_angle.cos();

        (x, y, z)
    }

    fn update_rotation_angles(
        &mut self,
        prev_pos: PhysicalPosition<f64>,
        next_pos: PhysicalPosition<f64>,
    ) {
        if !self.left_mouse_btn_pressed || self.cursor_left {
            return;
        }

        let delta_x = prev_pos.x - next_pos.x;
        let delta_y = prev_pos.y - next_pos.y;

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
