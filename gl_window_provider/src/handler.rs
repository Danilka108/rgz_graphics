use std::num::NonZeroU32;

use glutin::{
    context::PossiblyCurrentContext,
    display::GetGlDisplay,
    prelude::{GlDisplay, NotCurrentGlContextSurfaceAccessor},
    surface::{GlSurface, Surface, WindowSurface},
};
use glutin_winit::GlWindow;
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

use crate::{GlWindowProvider, Renderer};

pub(crate) struct EventsHandler<R>
where
    R: Renderer,
{
    pub gl_provider: GlWindowProvider,
    pub renderer: Option<R>,
    pub state: Option<State>,
}

pub(crate) struct State {
    gl_surface: Surface<WindowSurface>,
    gl_context: PossiblyCurrentContext,
    window: Window,
}

impl<R> EventsHandler<R>
where
    R: Renderer,
{
    pub fn handle<'handler, 'event, 'win_target, 'control_flow, T>(
        &'handler mut self,
        event: Event<'event, T>,
        win_target: &'win_target EventLoopWindowTarget<T>,
        control_flow: &'control_flow mut ControlFlow,
    ) {
        control_flow.set_wait();

        match event {
            Event::Resumed => self.handle_resumed_event(win_target),
            Event::WindowEvent {
                event: win_event, ..
            } => {
                self.handle_window_event(control_flow, win_event);
            }
            Event::RedrawRequested(_) => self.handle_redraw_event(),
            _ => (),
        }
    }

    fn handle_resumed_event<T>(&mut self, win_target: &EventLoopWindowTarget<T>) {
        let gl_config = &self.gl_provider.gl_config;
        let gl_display = gl_config.display();

        let window = self.gl_provider.window.take().unwrap_or_else(|| {
            glutin_winit::finalize_window(win_target, WindowBuilder::new(), gl_config).unwrap()
        });

        let gl_surface_attrs = window.build_surface_attributes(Default::default());
        let gl_surface =
            unsafe { gl_display.create_window_surface(gl_config, &gl_surface_attrs) }.unwrap();

        let gl_context = self
            .gl_provider
            .not_current_context
            .take()
            .unwrap()
            .make_current(&gl_surface)
            .unwrap();

        self.renderer.get_or_insert_with(|| R::new(&gl_display));

        assert!(self
            .state
            .replace(State {
                gl_context,
                gl_surface,
                window,
            })
            .is_none());
    }

    fn handle_window_event<'win_event>(
        &mut self,
        control_flow: &mut ControlFlow,
        window_event: WindowEvent<'win_event>,
    ) {
        match window_event {
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::CloseRequested => {
                control_flow.set_exit();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.renderer
                    .as_mut()
                    .map(|renderer| renderer.mouse_input_hook(state, button));
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                self.renderer
                    .as_mut()
                    .map(|renderer| renderer.mouse_wheel_hook(delta, phase));
            }
            WindowEvent::KeyboardInput { input, .. } => {
                self.renderer
                    .as_mut()
                    .map(|renderer| renderer.keyboard_input_hook(input));
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.renderer
                    .as_mut()
                    .map(|renderer| renderer.cursor_move_hook(position));
            }
            WindowEvent::CursorEntered { .. } => {
                self.renderer
                    .as_mut()
                    .map(|renderer| renderer.cursor_enter_hook());
            }
            WindowEvent::CursorLeft { .. } => {
                self.renderer
                    .as_mut()
                    .map(|renderer| renderer.cursor_left_hook());
            }
            _ => (),
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        let Some(State { ref gl_surface, ref gl_context, .. }) = self.state else {
            return;
        };

        gl_surface.resize(
            gl_context,
            NonZeroU32::new(size.width).unwrap(),
            NonZeroU32::new(size.height).unwrap(),
        );

        self.renderer
            .as_mut()
            .unwrap()
            .resize(size.width as i32, size.height as i32);
    }

    fn handle_redraw_event(&mut self) {
        let Some(State { ref gl_surface, ref gl_context, ref window }) = self.state else {
            return;
        };

        self.renderer
            .as_mut()
            .unwrap()
            .draw(gl_surface.width(), gl_surface.height());

        window.request_redraw();
        gl_surface.swap_buffers(gl_context).unwrap();
    }
}
