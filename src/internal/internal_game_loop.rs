use crate::{
    common::{
        application_builder::ApplicationBuilder,
        errors::{DuendeError, UnsupportedDevice},
        game::Game,
    },
    three_d::three_d_application_context::{Command, Event, ThreeDApplicationContext},
};
use bumpalo::Bump;
use glutin::{
    config::{Config, ConfigTemplateBuilder, GlConfig},
    context::{
        ContextApi, ContextAttributesBuilder, GlProfile, NotCurrentContext, NotCurrentGlContext,
        PossiblyCurrentContext, Version,
    },
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SwapInterval, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use std::num::NonZeroU32;
use tracing::{error, info};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    keyboard::Key,
    window::{CursorGrabMode, Window, WindowAttributes},
};

pub(crate) struct InnerApplication<'a, G> {
    template: ConfigTemplateBuilder,
    display_builder: DisplayBuilder,
    game_loop: G,
    context: Option<ThreeDApplicationContext<'a>>,
    window_attributes: WindowAttributes,
    not_current_gl_context: Option<NotCurrentContext>,
    state: Option<AppState>,
    builder: ApplicationBuilder,
    pub(crate) exit_state: Result<(), DuendeError>,
    bump: &'a Bump,
}

impl<'a, G> InnerApplication<'a, G>
where
    G: Game,
{
    pub(crate) fn new(
        template: ConfigTemplateBuilder,
        display_builder: DisplayBuilder,
        game_loop: G,
        window_attributes: WindowAttributes,
        builder: ApplicationBuilder,
        bump: &'a Bump,
    ) -> Self {
        Self {
            template,
            display_builder,
            game_loop,
            context: None,
            window_attributes,
            not_current_gl_context: None,
            state: None,
            builder,
            exit_state: Ok(()),
            bump,
        }
    }
}

impl<'a, G> ApplicationHandler for InnerApplication<'a, G>
where
    G: Game,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let (mut window, gl_config) = match self.display_builder.clone().build(
            event_loop,
            self.template.clone(),
            gl_config_picker,
        ) {
            Ok(ok) => ok,
            Err(e) => {
                self.exit_with_error(event_loop, DuendeError::InternalError(e));
                return;
            }
        };
        info!("Picked a config with {} samples", gl_config.num_samples());
        let raw_window_handle = window
            .as_ref()
            .and_then(|window| window.window_handle().ok())
            .map(|handle| handle.as_raw());
        let gl_display = gl_config.display();
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .with_profile(GlProfile::Compatibility)
            .build(raw_window_handle);
        let not_current_gl_context = self
            .not_current_gl_context
            .take()
            .unwrap_or_else(|| unsafe {
                gl_display
                    .create_context(&gl_config, &context_attributes)
                    .expect("failed to create context")
            });
        let window = window.take().unwrap_or_else(|| {
            let window_attributes = self.window_attributes.clone();
            glutin_winit::finalize_window(event_loop, window_attributes, &gl_config).unwrap()
        });
        if self.builder.grab_mouse && window.set_cursor_grab(CursorGrabMode::None).is_err() {
            self.exit_with_error(
                event_loop,
                DuendeError::UnsupportedDevice(UnsupportedDevice::CursorGrab),
            );
        }
        if !self.builder.mouse_cursor_visible {
            window.set_cursor_visible(false);
        }
        let attrs = window
            .build_surface_attributes(Default::default())
            .expect("Failed to build surface attributes");
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };
        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();
        self.context
            .get_or_insert_with(|| ThreeDApplicationContext::new(&gl_display, self.bump));
        if let Err(res) = gl_surface
            .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        {
            error!("Error setting vsync: {res:?}");
        }
        assert!(self
            .state
            .replace(AppState {
                gl_context,
                gl_surface,
                window
            })
            .is_none());
        self.game_loop.setup(self.context.as_mut().unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                if let Some(AppState {
                    gl_context,
                    gl_surface,
                    window: _,
                }) = self.state.as_ref()
                {
                    gl_surface.resize(
                        gl_context,
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    );
                    let renderer = self.context.as_ref().unwrap();
                    renderer.resize(size.width as i32, size.height as i32);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(key),
                        ..
                    },
                ..
            } => self
                .context
                .as_mut()
                .unwrap()
                .add_event(Event::KeyPress(key)),
            WindowEvent::CloseRequested => {
                self.exit(event_loop);
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(AppState {
            gl_context,
            gl_surface,
            window,
        }) = self.state.as_ref()
        {
            let context = self.context.as_mut().unwrap();
            self.game_loop.game_loop(context);
            let commands = context.pop_all_commands();
            let mut exit = false;
            let mut error = Ok(());
            for command in commands {
                match command {
                    Command::CursorGrab(enable) => {
                        let result = if enable {
                            window
                                .set_cursor_grab(CursorGrabMode::None)
                                .map_err(|_| UnsupportedDevice::CursorGrab)
                        } else {
                            window
                                .set_cursor_grab(CursorGrabMode::Confined)
                                .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked))
                                .map_err(|_| UnsupportedDevice::CursorGrab)
                        };
                        if let Err(e) = result {
                            error = Err(DuendeError::UnsupportedDevice(e));
                        }
                    }
                    Command::CursorVisible(enable) => {
                        window.set_cursor_visible(enable);
                    }
                    Command::Exit => {
                        exit = true;
                    }
                }
            }
            unsafe {
                if let Err(e) = context.draw() {
                    error = Err(DuendeError::GlError(e));
                }
            }
            window.request_redraw();
            gl_surface.swap_buffers(gl_context).unwrap();
            if exit {
                self.exit(event_loop);
            }
            if let Err(e) = error {
                self.exit_with_error(event_loop, e);
            }
        }
    }
}

impl<'a, G> InnerApplication<'a, G>
where
    G: Game,
{
    fn exit(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(ref mut context) = self.context {
            self.game_loop.teardown(context);
            event_loop.exit();
        }
    }

    fn exit_with_error(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        error: DuendeError,
    ) {
        if let Some(ref mut context) = self.context {
            self.exit_state = Err(error);
            self.game_loop.teardown(context);
            event_loop.exit();
        }
    }
}

struct AppState {
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    window: Window,
}

fn gl_config_picker(mut configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    const DEFAULT_MSAA: u8 = 4;

    configs
        .find(|config| config.num_samples() == DEFAULT_MSAA)
        .expect(&format!("unsupported msaa: {DEFAULT_MSAA}"))
}
