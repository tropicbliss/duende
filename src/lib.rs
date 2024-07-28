#![feature(allocator_api)]

pub mod drawables;
pub mod errors;
pub mod gl;
mod mut_cell;

use bumpalo::Bump;
use drawables::Drawable;
use errors::{DuendeError, GlError, UnsupportedDevice};
use fnv::FnvHashSet;
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
use mut_cell::MutCell;
use raw_window_handle::HasWindowHandle;
use std::{
    ffi::{CStr, CString},
    hash::Hash,
    num::NonZeroU32,
};
use tracing::{error, info};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize, Position, Size},
    event::{KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::{CursorGrabMode, Window, WindowAttributes},
};

pub struct ApplicationBuilder {
    title: String,
    window_size: Option<(u32, u32)>,
    window_position: Option<(i32, i32)>,
    grab_mouse: bool,
    mouse_cursor_visible: bool,
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self {
            title: String::from("duende window"),
            window_size: None,
            window_position: None,
            grab_mouse: false,
            mouse_cursor_visible: true,
        }
    }
}

impl ApplicationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<String>,
    {
        self.title = title.into();
        self
    }

    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        self.window_size = Some((width, height));
        self
    }

    pub fn window_position(mut self, x: i32, y: i32) -> Self {
        self.window_position = Some((x, y));
        self
    }

    pub fn set_grab_mouse(mut self, grab_mouse: bool) -> Self {
        self.grab_mouse = grab_mouse;
        self
    }

    pub fn set_mouse_cursor_visible(mut self, mouse_cursor_visible: bool) -> Self {
        self.mouse_cursor_visible = mouse_cursor_visible;
        self
    }

    pub fn build(self) -> Application {
        Application::new(self)
    }
}

pub struct Application {
    builder: ApplicationBuilder,
}

impl Application {
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::default()
    }

    fn new(builder: ApplicationBuilder) -> Self {
        Self { builder }
    }

    pub fn render<G>(self, game_loop: G) -> Result<(), DuendeError>
    where
        G: Game,
    {
        let event_loop = EventLoop::new().unwrap();
        let mut window_attributes =
            Window::default_attributes().with_title(self.builder.title.clone());
        if let Some((width, height)) = self.builder.window_size {
            window_attributes =
                window_attributes.with_inner_size(Size::Physical(PhysicalSize::new(width, height)));
        }
        if let Some((x, y)) = self.builder.window_position {
            window_attributes =
                window_attributes.with_position(Position::Physical(PhysicalPosition::new(x, y)));
        }
        let template = ConfigTemplateBuilder::new();
        let display_builder =
            DisplayBuilder::new().with_window_attributes(Some(window_attributes.clone()));
        let bump = Bump::new();
        let mut app = InnerApplication::new(
            template,
            display_builder,
            game_loop,
            window_attributes,
            self.builder,
            &bump,
        );
        event_loop.run_app(&mut app).unwrap();
        app.exit_state
    }
}

struct InnerApplication<'a, G> {
    template: ConfigTemplateBuilder,
    display_builder: DisplayBuilder,
    game_loop: G,
    context: Option<ApplicationContext<'a>>,
    window_attributes: WindowAttributes,
    not_current_gl_context: Option<NotCurrentContext>,
    state: Option<AppState>,
    builder: ApplicationBuilder,
    exit_state: Result<(), DuendeError>,
    bump: &'a Bump,
}

impl<'a, G> InnerApplication<'a, G>
where
    G: Game,
{
    fn new(
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
            game_loop: game_loop,
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
        if self.builder.grab_mouse {
            if window.set_cursor_grab(CursorGrabMode::None).is_err() {
                self.exit_with_error(
                    event_loop,
                    DuendeError::UnsupportedDevice(UnsupportedDevice::CursorGrab),
                );
            }
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
            .get_or_insert_with(|| ApplicationContext::new(&gl_display, &self.bump));
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
            let mut context = self.context.as_mut().unwrap();
            self.game_loop.game_loop(&mut context);
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

pub struct RendererContext<'a> {
    bump: &'a Bump,
    command_queue: Vec<Box<dyn FnOnce(), &'a Bump>>,
}

impl<'a> RendererContext<'a> {
    fn new(bump: &'a Bump) -> Self {
        Self {
            bump,
            command_queue: Vec::new(),
        }
    }

    pub fn get_bump_allocator(&self) -> &'a Bump {
        self.bump
    }

    pub fn add_commands<F>(&mut self, queue: F)
    where
        F: FnOnce() + 'static,
    {
        let object = Box::new_in(queue, self.bump);
        self.command_queue.push(object);
    }
}

pub struct ApplicationContext<'a> {
    input_events: FnvHashSet<Event>,
    output_commands: Vec<Command, &'a Bump>,
    background_color: MutCell<InternalColor>,
    renderer_context: RendererContext<'a>,
    exit_status: Result<(), GlError>,
}

impl<'a> ApplicationContext<'a> {
    fn new<D>(gl_display: &D, bump: &'a Bump) -> Self
    where
        D: GlDisplay,
    {
        unsafe {
            gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });
            if let Some(renderer) = get_gl_string(gl::RENDERER) {
                info!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(gl::VERSION) {
                info!("OpenGL Version {}", version.to_string_lossy());
            }
            if let Some(shaders_version) = get_gl_string(gl::SHADING_LANGUAGE_VERSION) {
                info!("Shaders version on {}", shaders_version.to_string_lossy());
            }
            gl::MatrixMode(gl::PROJECTION);
            gl::LoadIdentity();
            Self {
                input_events: FnvHashSet::default(),
                output_commands: Vec::new_in(bump),
                background_color: MutCell::new(InternalColor::default()),
                renderer_context: RendererContext::new(bump),
                exit_status: Ok(()),
            }
        }
    }

    fn pop_all_commands(&mut self) -> Vec<Command, &'a Bump> {
        let mut output = Vec::new_in(self.renderer_context.bump);
        std::mem::swap(&mut self.output_commands, &mut output);
        output
    }

    pub fn exit(&mut self) {
        self.output_commands.push(Command::Exit);
    }

    fn resize(&self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }

    pub fn set_cursor_grab(&mut self, enable: bool) {
        self.output_commands.push(Command::CursorGrab(enable));
    }

    pub fn set_background_color(&mut self, red: u8, green: u8, blue: u8, alpha: u8) {
        self.background_color.set(InternalColor(
            red as f32 / u8::MAX as f32,
            green as f32 / u8::MAX as f32,
            blue as f32 / u8::MAX as f32,
            alpha as f32 / u8::MAX as f32,
        ));
    }

    pub fn set_cursor_visible(&mut self, enable: bool) {
        self.output_commands.push(Command::CursorVisible(enable));
    }

    fn add_event(&mut self, event: Event) {
        self.input_events.insert(event);
    }

    pub fn draw_game_object<D>(&mut self, object: &D)
    where
        D: Drawable,
    {
        self.exit_status = object.draw(&mut self.renderer_context);
    }

    unsafe fn draw(&mut self) -> Result<(), GlError> {
        if let Err(e) = &self.exit_status {
            return Err(e.clone());
        }
        self.background_color.execute_on_change(|new_value| {
            gl::ClearColor(new_value.0, new_value.1, new_value.2, new_value.3);
        });
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        while let Some(commands) = self.renderer_context.command_queue.pop() {
            (commands)();
        }
        Ok(())
    }

    pub fn is_key_pressed(&self, key: NamedKey) -> bool {
        self.input_events.contains(&Event::KeyPress(key))
    }
}

fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

fn gl_config_picker(mut configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    const DEFAULT_MSAA: u8 = 4;

    configs
        .find(|config| config.num_samples() == DEFAULT_MSAA)
        .expect(format!("unsupported msaa: {DEFAULT_MSAA}").as_str())
}

#[derive(PartialEq, Eq, Hash)]
pub enum Event {
    Quit,
    KeyPress(NamedKey),
}

enum Command {
    Exit,
    CursorGrab(bool),
    CursorVisible(bool),
}

struct InternalColor(f32, f32, f32, f32);

impl Default for InternalColor {
    fn default() -> Self {
        InternalColor(0.1, 0.1, 0.1, 0.9)
    }
}

pub trait Game {
    fn game_loop(&self, context: &mut ApplicationContext);
    fn setup(&self, _context: &mut ApplicationContext) {}
    fn teardown(&self, _context: &mut ApplicationContext) {}
}
