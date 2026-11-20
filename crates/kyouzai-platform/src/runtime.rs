use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{
    ElementState, MouseButton as WinitMouseButton, WindowEvent as WinitWindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{WindowAttributes, WindowId as WinitWindowId};

use crate::{MouseButton, PlatformError, Window, WindowConfig, WindowEvent, WindowHandler};

pub(crate) fn run<H>(config: WindowConfig, handler: H) -> Result<(), PlatformError>
where
    H: WindowHandler + 'static,
{
    let event_loop = EventLoop::new().map_err(|_| PlatformError::EventLoopCreation)?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = RuntimeApp {
        config,
        window: None,
        handler,
    };

    event_loop
        .run_app(&mut app)
        .map_err(|_| PlatformError::EventLoopRuntime)
}

struct RuntimeApp<H> {
    config: WindowConfig,
    handler: H,
    window: Option<Window>,
}

impl<H: WindowHandler> RuntimeApp<H> {
    fn window_attributes(&self) -> WindowAttributes {
        let size = PhysicalSize::new(self.config.physical_width, self.config.physical_height);

        let mut attributes = WindowAttributes::default()
            .with_inner_size(size)
            .with_transparent(self.config.transparent)
            .with_decorations(self.config.decorations)
            .with_resizable(self.config.resizable)
            .with_window_level(if self.config.always_on_top {
                winit::window::WindowLevel::AlwaysOnTop
            } else {
                winit::window::WindowLevel::Normal
            })
            .with_visible(self.config.visible);

        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowAttributesExtWindows;
            attributes = attributes.with_skip_taskbar(self.config.skip_taskbar);
        }

        attributes
    }

    fn dispatch(&mut self, id: WinitWindowId, event: WindowEvent) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if window.inner().id() == id {
            self.handler.window_event(window, event);
        }
    }
}

impl<H: WindowHandler> ApplicationHandler for RuntimeApp<H> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = self.window_attributes();
        match event_loop.create_window(attributes) {
            Ok(winit_window) => {
                let window = self.window.insert(Window::new(winit_window));
                self.handler.resumed(window);
            }
            Err(error) => {
                eprintln!("kyouzai-platform: failed to create window: {error}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        id: WinitWindowId,
        event: WinitWindowEvent,
    ) {
        match event {
            WinitWindowEvent::CloseRequested => {
                self.dispatch(id, WindowEvent::CloseRequested);
                event_loop.exit();
            }

            WinitWindowEvent::Resized(size) => {
                self.dispatch(
                    id,
                    WindowEvent::Resized {
                        width: size.width,
                        height: size.height,
                    },
                );
            }

            WinitWindowEvent::RedrawRequested => {
                self.dispatch(id, WindowEvent::RedrawRequested);
            }

            WinitWindowEvent::ScaleFactorChanged {
                scale_factor,
                mut inner_size_writer,
            } => {
                let size = if let Some(window) = self.window.as_ref() {
                    let physical =
                        PhysicalSize::new(self.config.physical_width, self.config.physical_height);

                    let _ = inner_size_writer.request_inner_size(physical);

                    window.inner_size()
                } else {
                    return;
                };

                self.dispatch(
                    id,
                    WindowEvent::ScaleFactorChanged {
                        scale_factor,
                        width: size.width,
                        height: size.height,
                    },
                );
            }

            WinitWindowEvent::CursorMoved { position, .. } => {
                self.dispatch(
                    id,
                    WindowEvent::CursorMoved {
                        position: glam::Vec2::new(position.x as f32, position.y as f32),
                    },
                );
            }

            WinitWindowEvent::CursorLeft { .. } => {
                self.dispatch(id, WindowEvent::CursorLeft);
            }

            WinitWindowEvent::MouseInput { state, button, .. } => {
                self.dispatch(
                    id,
                    WindowEvent::MouseInput {
                        button: convert_mouse_button(button),
                        pressed: state == ElementState::Pressed,
                    },
                );
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        match self.handler.about_to_wait(window) {
            Some(deadline) => {
                event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
            }
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }
}

fn convert_mouse_button(button: WinitMouseButton) -> MouseButton {
    match button {
        WinitMouseButton::Left => MouseButton::Left,
        WinitMouseButton::Right => MouseButton::Right,
        WinitMouseButton::Middle => MouseButton::Middle,
        WinitMouseButton::Back => MouseButton::Other(4),
        WinitMouseButton::Forward => MouseButton::Other(5),
        WinitMouseButton::Other(n) => MouseButton::Other(n),
    }
}
