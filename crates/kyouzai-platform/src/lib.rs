mod runtime;

#[cfg(target_os = "windows")]
mod platform_windows;

use glam::Vec2;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};

pub use winit::dpi::PhysicalSize;

// -----------------------------------------------------------------------------
// Platform integration
// -----------------------------------------------------------------------------

#[cfg(target_os = "windows")]
pub type PlatformHandle = platform_windows::WindowsPlatform;

/// Creates the native platform integration for a window.
#[cfg(target_os = "windows")]
pub fn create_platform(handle: RawWindowHandle) -> Result<PlatformHandle, PlatformError> {
    PlatformHandle::new(handle)
}

/// Native window operations provided by the current platform backend.
pub trait PlatformWindowExt {
    fn enable_layered(&mut self) -> Result<(), PlatformError>;

    fn monitors(&self) -> Vec<MonitorInfo>;

    fn set_click_through(&mut self, enabled: bool) -> Result<(), PlatformError>;

    /// Returns the cursor position in the platform-defined window coordinate space.
    fn cursor_position(&self) -> Option<(f64, f64)>;

    /// Returns the cursor position in global screen coordinates.
    fn global_cursor_position(&self) -> Option<(f64, f64)>;

    fn is_fullscreen_app_foreground(&self) -> bool;

    fn register_autostart(&self, enabled: bool) -> Result<(), PlatformError>;
}

/// A rectangular region in global physical screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlatformRect {
    pub min: Vec2,
    pub max: Vec2,
}

impl PlatformRect {
    pub const fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }
}

/// Errors produced by platform integration and window management.
#[derive(Debug)]
pub enum PlatformError {
    EventLoopCreation,
    EventLoopRuntime,
    Unsupported,
    OsError(String),
}

impl std::fmt::Display for PlatformError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EventLoopCreation => write!(formatter, "failed to create event loop"),
            Self::EventLoopRuntime => {
                write!(formatter, "window event loop terminated unexpectedly")
            }
            Self::Unsupported => {
                write!(
                    formatter,
                    "this platform feature is not implemented for the current OS"
                )
            }
            Self::OsError(message) => write!(formatter, "platform API error: {message}"),
        }
    }
}

impl std::error::Error for PlatformError {}

// -----------------------------------------------------------------------------
// Window configuration and properties
// -----------------------------------------------------------------------------

/// Configuration used when creating the application window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowConfig {
    pub physical_width: u32,
    pub physical_height: u32,
    pub transparent: bool,
    pub decorations: bool,
    pub resizable: bool,
    pub always_on_top: bool,
    pub visible: bool,
    pub skip_taskbar: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            physical_width: 256,
            physical_height: 256,
            transparent: true,
            decorations: false,
            resizable: false,
            always_on_top: true,
            visible: false,
            skip_taskbar: true,
        }
    }
}

/// Z-order level of a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowLevel {
    Normal,
    AlwaysOnTop,
}

/// Identifier used to distinguish windows managed by the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

/// Mouse buttons reported by the platform event system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

// -----------------------------------------------------------------------------
// Window events and event handling
// -----------------------------------------------------------------------------

/// Events reported by the platform for an application window.
#[derive(Debug)]
pub enum WindowEvent {
    CloseRequested,
    Resized {
        width: u32,
        height: u32,
    },
    RedrawRequested,
    ScaleFactorChanged {
        scale_factor: f64,
        width: u32,
        height: u32,
    },
    CursorMoved {
        position: Vec2,
    },
    CursorLeft,
    MouseInput {
        button: MouseButton,
        pressed: bool,
    },
}

/// Receives window lifecycle and event notifications from the platform runtime.
pub trait WindowHandler {
    fn resumed(&mut self, window: &Window);

    fn window_event(&mut self, window: &Window, event: WindowEvent);

    fn about_to_wait(&mut self, window: &Window) -> Option<std::time::Instant>;
}

// -----------------------------------------------------------------------------
// Window runtime
// -----------------------------------------------------------------------------

/// Owns the configuration and starts the platform window event loop.
pub struct WindowManager {
    config: WindowConfig,
}

impl WindowManager {
    pub fn new(config: WindowConfig) -> Self {
        Self { config }
    }

    pub fn run<H>(self, handler: H) -> Result<(), PlatformError>
    where
        H: WindowHandler + 'static,
    {
        runtime::run(self.config, handler)
    }
}

// -----------------------------------------------------------------------------
// Window handles and wrapper
// -----------------------------------------------------------------------------

/// Raw handles required by graphics and native platform integrations.
#[derive(Debug, Clone, Copy)]
pub struct WindowHandles {
    pub window: RawWindowHandle,
    pub display: RawDisplayHandle,
}

/// Application-facing wrapper around the platform runtime window.
#[derive(Debug)]
pub struct Window {
    inner: winit::window::Window,
}

impl Window {
    pub(crate) fn new(inner: winit::window::Window) -> Self {
        Self { inner }
    }

    pub fn id(&self) -> WindowId {
        WindowId(self.inner.id().into())
    }

    pub fn inner_size(&self) -> PhysicalSize<u32> {
        self.inner.inner_size()
    }

    pub fn scale_factor(&self) -> f64 {
        self.inner.scale_factor()
    }

    pub fn request_redraw(&self) {
        self.inner.request_redraw();
    }

    pub fn set_visible(&self, visible: bool) {
        self.inner.set_visible(visible);
    }

    pub fn set_outer_position(&self, position: Vec2) {
        let physical = winit::dpi::PhysicalPosition::new(position.x as i32, position.y as i32);
        self.inner.set_outer_position(physical);
    }

    pub(crate) fn inner(&self) -> &winit::window::Window {
        &self.inner
    }

    pub fn handles(&self) -> Result<WindowHandles, raw_window_handle::HandleError> {
        Ok(WindowHandles {
            window: self.inner.window_handle()?.as_raw(),
            display: self.inner.display_handle()?.as_raw(),
        })
    }

    pub fn outer_position(&self) -> Option<Vec2> {
        let position = self.inner.outer_position().ok()?;

        Some(Vec2::new(position.x as f32, position.y as f32))
    }

    pub fn set_window_level(&self, level: WindowLevel) {
        let level = match level {
            WindowLevel::Normal => winit::window::WindowLevel::Normal,
            WindowLevel::AlwaysOnTop => winit::window::WindowLevel::AlwaysOnTop,
        };

        self.inner.set_window_level(level);
    }
}

// -----------------------------------------------------------------------------
// Monitor information
// -----------------------------------------------------------------------------

/// Position and dimensions of a monitor in global physical screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonitorInfo {
    pub position: Vec2,
    pub size: Vec2,
    pub work_area_position: Vec2,
    pub work_area_size: Vec2,
}
