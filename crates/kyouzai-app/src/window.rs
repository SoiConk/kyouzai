use kyouzai_platform::{WindowEvent, WindowId};

/// Application-level state for the window managed by this application.
///
/// Tracks window identity and physical dimensions. It also tracks the scale
/// factor so DPI changes remain part of the window's state.
pub struct ApplicationWindow {
    id: WindowId,
    width: u32,
    height: u32,
}

impl ApplicationWindow {
    pub fn new(id: WindowId, width: u32, height: u32) -> Self {
        Self { id, width, height }
    }

    /// Returns the identity of the window represented by this state.
    pub fn id(&self) -> WindowId {
        self.id
    }

    /// Updates the tracked state and describes the action required by the application.
    pub fn handle_event(&mut self, event: WindowEvent) -> WindowCommand {
        match event {
            WindowEvent::CloseRequested => WindowCommand::Close,

            WindowEvent::Resized { width, height } => {
                self.width = width;
                self.height = height;

                WindowCommand::Resize {
                    width: self.width,
                    height: self.height,
                }
            }

            WindowEvent::ScaleFactorChanged { width, height, .. } => {
                self.width = width;
                self.height = height;

                WindowCommand::Resize {
                    width: self.width,
                    height: self.height,
                }
            }

            WindowEvent::RedrawRequested => WindowCommand::Redraw,

            // Input events are handled directly by Application and InputController.
            // Keep these variants explicit so new event types require an intentional
            // routing decision.
            WindowEvent::CursorMoved { .. }
            | WindowEvent::CursorLeft
            | WindowEvent::MouseInput { .. } => WindowCommand::Ignored,
        }
    }
}

/// An action that Application must perform in response to a window event.
#[derive(Debug, Clone, Copy)]
pub enum WindowCommand {
    Close,
    Resize {
        width: u32,
        height: u32,
    },
    Redraw,

    /// The event is handled elsewhere and requires no action here.
    Ignored,
}
