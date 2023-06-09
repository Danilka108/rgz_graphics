use gl_window_provider::Renderer;
use glam::{Mat4, Vec3};
use std::ffi::CString;
use winit::{
    dpi::PhysicalPosition,
    event::{
        ElementState, KeyboardInput, MouseButton, MouseScrollDelta, TouchPhase, VirtualKeyCode,
    },
};

use crate::{
    array::{AttribPointer, Size, VerticesArray},
    shader_program::{ShaderProgram, ShaderProgramBuilder},
    texture::Texture,
};

enum Projection {
    Perspective,
    Axonometric,
}

enum ModelKind {
    Color,
    Texture,
}

struct DirLight {
    direction: Vec3,
    ambient: Vec3,
    diffuse: Vec3,
    specular: Vec3,
}

struct PointLight {
    position: Vec3,
    ambient: Vec3,
    diffuse: Vec3,
    specular: Vec3,
    constant: f32,
    linear: f32,
    quadratic: f32,
}

struct Material {
    ambient: Vec3,
    diffuse: Vec3,
    specular: Vec3,
    shininess: f32,
}

pub(crate) struct RgzRenderer {
    gl: gl::Gl,

    texture: Texture,

    model_slices_count: u32,
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

    projection: Projection,
    show_mesh: bool,
    show_model: bool,
    model_kind: ModelKind,
}

impl RgzRenderer {
    const USE_PERSPECTIVE_PROJ_KEYCODE: VirtualKeyCode = VirtualKeyCode::Key1;
    const USE_AXONOMETRIC_PROJ_KEYCODE: VirtualKeyCode = VirtualKeyCode::Key2;

    const TOGGLE_MESH_DISPLAY_KEYCODE: VirtualKeyCode = VirtualKeyCode::Key3;
    const TOGGLE_MODEL_DISPLAY_KEYCODE: VirtualKeyCode = VirtualKeyCode::Key4;

    const USE_COLOR_MODEL_KEYCODE: VirtualKeyCode = VirtualKeyCode::Key5;
    const USE_TEXTURE_MODEL_KEYCODE: VirtualKeyCode = VirtualKeyCode::Key6;

    const MODEL_SLICES_COUNT: u32 = 1000;
    const MESH_SLICES_COUNT: u32 = 100;
    const FIGURE_RADIUS: f32 = 1.0;

    const ZOOM_FACTOR: f32 = 1.0 / 10.0;
    const ZOOM_MIN: f32 = -20.0;
    const ZOOM_DEFAULT: f32 = -4.5;
    const ZOOM_MAX: f32 = -2.0;

    const DELTA_X_INTO_DELTA_ANGLE_FACTOR: f32 = std::f32::consts::FRAC_PI_2 / (1920.0 / 2.0);
    const DELTA_Y_INTO_DELTA_ANGLE_FACTOR: f32 = std::f32::consts::FRAC_PI_2 / (1280.0 / 2.0);

    const POLAR_ANGLE_MAX: f32 = std::f32::consts::FRAC_PI_2 * 0.9;
    const POLAR_ANGLE_MIN: f32 = -Self::POLAR_ANGLE_MAX;

    const MATERIAL: Material = Material {
        ambient: Vec3::new(0.1745, 0.01175, 0.01175),
        diffuse: Vec3::new(0.61424, 0.04136, 0.04136),
        specular: Vec3::new(0.727811, 0.626959, 0.626959),
        shininess: 0.6,
    };

    const DIR_LIGHT: DirLight = DirLight {
        direction: Vec3::new(0.0, 1.0, 0.0),
        ambient: Vec3::splat(0.1),
        diffuse: Vec3::splat(0.2),
        specular: Vec3::splat(0.4),
    };

    const POINT_LIGHT: PointLight = PointLight {
        position: Vec3::new(1.0, 1.0, 1.0),
        ambient: Vec3::splat(0.1),
        diffuse: Vec3::splat(0.4),
        specular: Vec3::splat(0.5),
        constant: 1.0,
        linear: 0.09,
        quadratic: 0.032,
    };
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

        let polygon_slices_count = Self::MODEL_SLICES_COUNT;
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

        let texture = Texture::open(gl.clone(), "/home/danilka108/labs/graphics_rgz/texture.jpg");

        Self {
            texture,
            gl,

            figure_radius: radius,
            model_slices_count: polygon_slices_count,
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

            projection: Projection::Perspective,
            show_mesh: true,
            show_model: true,
            model_kind: ModelKind::Color,
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
            .clamp(Self::ZOOM_MIN, Self::ZOOM_MAX);
    }

    fn keyboard_input_hook(&mut self, input: KeyboardInput) {
        match input.virtual_keycode {
            Some(Self::USE_AXONOMETRIC_PROJ_KEYCODE)
                if matches!(input.state, ElementState::Pressed) =>
            {
                self.projection = Projection::Axonometric
            }
            Some(Self::USE_PERSPECTIVE_PROJ_KEYCODE)
                if matches!(input.state, ElementState::Pressed) =>
            {
                self.projection = Projection::Perspective
            }
            Some(Self::TOGGLE_MESH_DISPLAY_KEYCODE)
                if matches!(input.state, ElementState::Pressed) =>
            {
                self.show_mesh = !self.show_mesh
            }
            Some(Self::TOGGLE_MODEL_DISPLAY_KEYCODE)
                if matches!(input.state, ElementState::Pressed) =>
            {
                self.show_model = !self.show_model;
            }
            Some(Self::USE_COLOR_MODEL_KEYCODE) if matches!(input.state, ElementState::Pressed) => {
                self.model_kind = ModelKind::Color;
            }
            Some(Self::USE_TEXTURE_MODEL_KEYCODE)
                if matches!(input.state, ElementState::Pressed) =>
            {
                self.model_kind = ModelKind::Texture;
            }
            _ => (),
        };
    }

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

    fn draw(&mut self, width: Option<u32>, height: Option<u32>) {
        let Some(width) = width else {
            return;
        };

        let Some(height) = height else {
            return;
        };

        let model_matrix = Mat4::IDENTITY;
        let view_matrix = self.calc_look_at_matrix();
        let projection_matrix = self.get_proj_matrix(width, height);

        unsafe {
            self.gl.Enable(gl::DEPTH_TEST);
            self.gl.ClearColor(0.0, 0.0, 0.0, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        self.texture.bind();
        self.polygon_array.use_array();

        self.polygon_program.use_program();
        self.polygon_program
            .set_uniform_vec3("uDirLight.direction", Self::DIR_LIGHT.direction.to_array());
        self.polygon_program
            .set_uniform_vec3("uDirLight.ambient", Self::DIR_LIGHT.ambient.to_array());
        self.polygon_program
            .set_uniform_vec3("uDirLight.diffuse", Self::DIR_LIGHT.diffuse.to_array());
        self.polygon_program
            .set_uniform_vec3("uDirLight.specular", Self::DIR_LIGHT.specular.to_array());

        self.polygon_program.set_uniform_vec3(
            "uPointLight.position",
            Self::POINT_LIGHT.position.to_array(),
        );
        self.polygon_program
            .set_uniform_vec3("uPointLight.ambient", Self::POINT_LIGHT.ambient.to_array());
        self.polygon_program
            .set_uniform_vec3("uPointLight.diffuse", Self::POINT_LIGHT.diffuse.to_array());
        self.polygon_program.set_uniform_vec3(
            "uPointLight.specular",
            Self::POINT_LIGHT.specular.to_array(),
        );
        self.polygon_program
            .set_uniform_f32("uPointLight.constant", Self::POINT_LIGHT.constant);
        self.polygon_program
            .set_uniform_f32("uPointLight.linear", Self::POINT_LIGHT.linear);
        self.polygon_program
            .set_uniform_f32("uPointLight.quadratic", Self::POINT_LIGHT.quadratic);

        self.polygon_program
            .set_uniform_vec3("uMaterial.colorAmbient", Self::MATERIAL.ambient.to_array());
        self.polygon_program
            .set_uniform_vec3("uMaterial.colorDiffuse", Self::MATERIAL.diffuse.to_array());
        self.polygon_program.set_uniform_vec3(
            "uMaterial.colorSpecular",
            Self::MATERIAL.specular.to_array(),
        );
        self.polygon_program
            .set_uniform_f32("uMaterial.shininess", Self::MATERIAL.shininess);
        self.polygon_program.set_uniform_bool(
            "uMaterial.useColor",
            matches!(self.model_kind, ModelKind::Color),
        );

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
            .set_uniform_u32("uSlicesCount", self.model_slices_count);
        self.polygon_program
            .set_uniform_mat4("uProjectionMat", projection_matrix.to_cols_array());

        unsafe {
            if self.show_model {
                self.gl
                    .DrawArrays(gl::POINTS, 0, self.polygon_array.len() as i32);
            }
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
        self.mesh_program
            .set_uniform_mat4("uProjectionMat", projection_matrix.to_cols_array());

        unsafe {
            if self.show_mesh {
                self.gl
                    .DrawArrays(gl::POINTS, 0, self.mesh_array.len() as i32);
            }
        }
    }

    fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}

impl RgzRenderer {
    fn get_proj_matrix(&self, width: u32, height: u32) -> Mat4 {
        match self.projection {
            Projection::Perspective => Mat4::perspective_lh(
                std::f32::consts::FRAC_PI_4,
                width as f32 / height as f32,
                0.1,
                100.0,
            ),
            Projection::Axonometric => Mat4::orthographic_rh(
                -(width as f32 / 700.0),
                width as f32 / 700.0,
                -(height as f32 / 700.0),
                height as f32 / 700.0,
                40.0,
                -40.0,
            ),
        }
    }

    fn calc_look_at_matrix(&self) -> Mat4 {
        let (x, y, z) = self.calc_view_pos();

        Mat4::look_at_lh(
            Vec3::new(x, y, z),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        )
    }

    fn calc_view_pos(&self) -> (f32, f32, f32) {
        let x =
            self.camera_zoom * self.camera_polar_angle.cos() * self.camera_azimuthal_angle.sin();
        let y = self.camera_zoom * self.camera_polar_angle.sin();
        let z =
            self.camera_zoom * self.camera_polar_angle.cos() * self.camera_azimuthal_angle.cos();

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

        let delta_x = -prev_pos.x + next_pos.x;
        let delta_y = prev_pos.y - next_pos.y;

        let delta_polar_angle = delta_y as f32 * Self::DELTA_Y_INTO_DELTA_ANGLE_FACTOR;
        let delta_azimuthal_angle = delta_x as f32 * Self::DELTA_X_INTO_DELTA_ANGLE_FACTOR;

        let polar_angle = self.camera_polar_angle + delta_polar_angle;
        let polar_angle = if polar_angle > Self::POLAR_ANGLE_MAX {
            Self::POLAR_ANGLE_MAX
        } else if polar_angle < Self::POLAR_ANGLE_MIN {
            Self::POLAR_ANGLE_MIN
        } else {
            polar_angle
        };

        let azimuthal_angle =
            (self.camera_azimuthal_angle + delta_azimuthal_angle) % (std::f32::consts::PI * 2.0);

        self.camera_polar_angle = polar_angle;
        self.camera_azimuthal_angle = azimuthal_angle;
    }
}
