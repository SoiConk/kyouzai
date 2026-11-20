use std::time::{Duration, Instant};

use glam::Vec2;

use kyouzai_core::{
    Companion, CompanionEvent, CompanionState, FALLING_STATE, MonitorArea, MonitorAreas,
    ScreenBounds,
};
use kyouzai_platform::{
    MonitorInfo, PlatformHandle, PlatformWindowExt, Window, WindowConfig, WindowEvent,
    WindowHandler, WindowLevel, WindowManager,
};
use kyouzai_renderer::{
    RenderError, RenderObject, RenderTransform, Renderer, TextureHandle, TextureRegion,
};

use crate::alpha_mask::AlphaMask;
use crate::input::{InputContext, InputController};
use crate::persistence::{self, AppState};
use crate::pet;
use crate::sleep_manager::SleepManager;
use crate::tray::{Tray, TrayCommand};
use crate::window::{ApplicationWindow, WindowCommand};

// -----------------------------------------------------------------------------
// Application configuration
// -----------------------------------------------------------------------------

const MIN_TICK_INTERVAL: Duration = Duration::from_millis(16);

// -----------------------------------------------------------------------------
// Event loop scheduling
// -----------------------------------------------------------------------------

const MOTION_TICK_INTERVAL: Duration = Duration::from_micros(16_667);
const CLICK_THROUGH_POLL_INTERVAL: Duration = Duration::from_millis(50);

// -----------------------------------------------------------------------------
// Application
// -----------------------------------------------------------------------------

pub struct Application {
    // Platform and window
    platform: Option<PlatformHandle>,
    window_state: Option<ApplicationWindow>,
    window_level: WindowLevel,
    monitors: MonitorAreas,
    visible: bool,

    // Rendering
    renderer: Option<Renderer>,
    texture_handle: Option<TextureHandle>,

    // Companion
    companion: Option<Companion>,
    alpha_mask: Option<AlphaMask>,
    window_center: Vec2,
    last_synced_position: Vec2,

    // Input
    input: InputController,
    click_through: Option<bool>,

    // Scheduling
    sleep_manager: SleepManager,
    last_tick: Instant,

    // Application services and persistence
    tray: Option<Tray>,
    app_state: AppState,
}

impl Application {
    // -------------------------------------------------------------------------
    // Construction and lifecycle
    // -------------------------------------------------------------------------

    pub fn new() -> Self {
        Self {
            // Platform and window
            platform: None,
            window_state: None,
            window_level: WindowLevel::AlwaysOnTop,
            monitors: MonitorAreas::default(),
            visible: true,

            // Rendering
            renderer: None,
            texture_handle: None,

            // Companion
            companion: None,
            alpha_mask: None,
            window_center: Vec2::ZERO,
            last_synced_position: Vec2::ZERO,

            // Input
            input: InputController::new(),
            click_through: None,

            // Scheduling
            sleep_manager: SleepManager::new(),
            last_tick: Instant::now(),

            // Application services and persistence
            tray: None,
            app_state: persistence::load(),
        }
    }

    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = WindowConfig::default();

        match pet::default_sprite_size() {
            Ok((width, height)) => {
                config.physical_width = width;
                config.physical_height = height;
            }
            Err(error) => {
                tracing::warn!("could not pre-read pet manifest for window sizing: {error}");
            }
        }

        WindowManager::new(config).run(self)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Rendering
    // -------------------------------------------------------------------------

    fn redraw(&mut self, window: &Window) {
        let Some(window_position) = window.outer_position() else {
            tracing::warn!("failed to get window position");
            return;
        };

        let (Some(renderer), Some(companion), Some(texture_handle)) = (
            self.renderer.as_mut(),
            self.companion.as_ref(),
            self.texture_handle,
        ) else {
            return;
        };

        let transform = companion.transform();

        let Some(sprite) = companion.sprite() else {
            return;
        };

        let object = RenderObject {
            texture: texture_handle,
            transform: RenderTransform {
                position: transform.position() - window_position,
                size: transform.effective_size(),
                rotation: transform.rotation(),
            },
            region: TextureRegion {
                uv_min: sprite.region().uv_min(),
                uv_max: sprite.region().uv_max(),
            },
            tint: sprite.tint(),
        };

        match renderer.render(&[object]) {
            Ok(()) => {}
            Err(RenderError::SurfaceLost | RenderError::SurfaceOutdated) => {
                let size = window.inner_size();
                renderer.resize(size.width, size.height);
            }
            Err(RenderError::Occluded | RenderError::Timeout) => {}
            Err(RenderError::Validation) => {
                tracing::error!("renderer validation error");
            }
            Err(RenderError::VertexBufferCapacityOverflow | RenderError::VertexBufferTooLarge) => {
                tracing::error!("renderer vertex buffer allocation failed");
            }
        }
    }

    // -------------------------------------------------------------------------
    // Platform and window synchronization
    // -------------------------------------------------------------------------

    fn sync_click_through(&mut self) {
        let desired = self.input.should_be_click_through();

        if self.click_through == Some(desired) {
            return;
        }

        let Some(platform) = self.platform.as_mut() else {
            return;
        };

        match platform.set_click_through(desired) {
            Ok(()) => self.click_through = Some(desired),
            Err(error) => tracing::trace!("set_click_through: {error}"),
        }
    }

    fn sync_window_position(&mut self, window: &Window) {
        let Some(companion) = self.companion.as_ref() else {
            return;
        };

        let position = companion.transform().position();

        if position == self.last_synced_position {
            return;
        }

        window.set_outer_position(position - self.window_center);
        self.last_synced_position = position;
    }

    // -------------------------------------------------------------------------
    // Input polling
    // -------------------------------------------------------------------------

    fn poll_click_through(&mut self) {
        if !self.input.should_be_click_through() {
            return;
        }

        let Some(cursor) = self
            .platform
            .as_ref()
            .and_then(PlatformWindowExt::cursor_position)
        else {
            return;
        };

        let Some(companion) = self.companion.as_ref() else {
            return;
        };

        self.input.poll_global_cursor(
            Vec2::new(cursor.0 as f32, cursor.1 as f32),
            companion,
            self.alpha_mask.as_ref(),
            self.window_center,
        );

        self.sync_click_through();
    }

    // -------------------------------------------------------------------------
    // Companion and scheduling
    // -------------------------------------------------------------------------

    fn is_sleeping(&self) -> bool {
        self.companion
            .as_ref()
            .map(|companion| companion.state_machine().current() == CompanionState::Sleep)
            .unwrap_or(false)
    }

    fn next_wait(&self, now: Instant) -> Duration {
        let is_sleeping = self.is_sleeping();

        let moving = self.companion.as_ref().is_some_and(|companion| {
            matches!(
                companion.state_machine().current(),
                CompanionState::Walk | CompanionState::Custom(FALLING_STATE)
            )
        });

        let mut wait = self
            .companion
            .as_ref()
            .and_then(Companion::seconds_until_next_update)
            .map(Duration::from_secs_f32)
            .unwrap_or(Duration::MAX);

        let sleep_check_wait = Duration::from_secs_f32(
            self.sleep_manager
                .seconds_until_next_check(now, is_sleeping)
                .max(0.0),
        );

        wait = wait.min(sleep_check_wait);

        if self.input.should_be_click_through() {
            wait = wait.min(CLICK_THROUGH_POLL_INTERVAL);
        }

        if moving {
            wait = wait.min(MOTION_TICK_INTERVAL);
        }

        wait.max(Duration::from_millis(1))
    }

    fn poll_sleep(&mut self, now: Instant, window: &Window) {
        let is_sleeping = self.is_sleeping();

        let Some(should_sleep) = self
            .platform
            .as_ref()
            .and_then(|platform| self.sleep_manager.poll(now, is_sleeping, platform))
        else {
            return;
        };

        let desired_level = if should_sleep {
            WindowLevel::Normal
        } else {
            WindowLevel::AlwaysOnTop
        };

        if self.window_level != desired_level {
            window.set_window_level(desired_level);
            self.window_level = desired_level;
        }

        // Fullscreen ended: restore window level, but never wake the companion.
        if !should_sleep || is_sleeping {
            return;
        }

        let Some(companion) = self.companion.as_mut() else {
            return;
        };

        if matches!(
            companion.state_machine().current(),
            CompanionState::Custom(id) if id == FALLING_STATE
        ) {
            return;
        }

        Self::log_events(&companion.request_state(CompanionState::Sleep));
    }

    // -------------------------------------------------------------------------
    // Persistence and utilities
    // -------------------------------------------------------------------------

    fn resolve_position(position: Option<Vec2>, monitors: &MonitorAreas, half_size: Vec2) -> Vec2 {
        let Some(first_monitor) = monitors.areas().first() else {
            return position
                .filter(|position| position.is_finite())
                .unwrap_or(Vec2::new(150.0, 150.0));
        };

        let fallback = Vec2::new(
            first_monitor.work_area.max.x - 150.0,
            first_monitor.work_area.max.y - 150.0,
        );

        let position = position
            .filter(|position| position.is_finite())
            .unwrap_or(fallback);

        let monitor = monitors
            .monitor_at(position)
            .or_else(|| monitors.nearest_monitor(position))
            .unwrap_or(first_monitor);

        let area = monitor.work_area;

        let min = area.min + half_size;
        let max = area.max - half_size;

        let clamp_axis = |value: f32, min: f32, max: f32| {
            if min <= max {
                value.clamp(min, max)
            } else {
                (min + max) * 0.5
            }
        };

        Vec2::new(
            clamp_axis(position.x, min.x, max.x),
            clamp_axis(position.y, min.y, max.y),
        )
    }

    fn save_state(&mut self) {
        let Some(companion) = self.companion.as_ref() else {
            return;
        };

        let position = Self::resolve_position(
            Some(companion.transform().position()),
            &self.monitors,
            self.window_center,
        );

        self.app_state.last_position = Some([position.x, position.y]);
        persistence::save(&self.app_state);
    }

    fn log_events(events: &[CompanionEvent]) {
        for event in events {
            tracing::debug!(?event, "companion event");
        }
    }

    fn build_monitor_areas(monitors: Vec<MonitorInfo>) -> MonitorAreas {
        MonitorAreas::new(
            monitors
                .into_iter()
                .map(|monitor| MonitorArea {
                    bounds: ScreenBounds::new(monitor.position, monitor.position + monitor.size),
                    work_area: ScreenBounds::new(
                        monitor.work_area_position,
                        monitor.work_area_position + monitor.work_area_size,
                    ),
                })
                .collect(),
        )
    }
}

// -----------------------------------------------------------------------------
// Platform event handling
// -----------------------------------------------------------------------------

impl WindowHandler for Application {
    fn resumed(&mut self, window: &Window) {
        let handles = match window.handles() {
            Ok(handles) => handles,
            Err(error) => {
                tracing::error!("failed to read window handles: {error}");
                return;
            }
        };

        // Initialize platform integration and retrieve monitor information.
        match kyouzai_platform::create_platform(handles.window) {
            Ok(mut platform) => {
                if let Err(error) = platform.enable_layered() {
                    tracing::warn!("failed to enable layered window: {error}");
                }

                self.monitors = Self::build_monitor_areas(platform.monitors());
                self.platform = Some(platform);
            }
            Err(error) => {
                tracing::warn!("platform integration unavailable: {error}");
            }
        }

        let size = window.inner_size();

        let half_size = Vec2::new(size.width as f32, size.height as f32) * 0.5;

        let initial_position = Self::resolve_position(
            self.app_state.last_position.map(|[x, y]| Vec2::new(x, y)),
            &self.monitors,
            half_size,
        );

        let mut renderer = match pollster::block_on(Renderer::new(
            handles.window,
            handles.display,
            (size.width, size.height),
        )) {
            Ok(renderer) => renderer,
            Err(error) => {
                tracing::error!("failed to initialize renderer: {error}");
                return;
            }
        };

        let loaded_pet = match pet::load_default_pet(&mut renderer, initial_position) {
            Ok(pet) => pet,
            Err(error) => {
                tracing::error!("failed to load default pet: {error}");
                return;
            }
        };

        match Tray::new() {
            Ok(tray) => self.tray = Some(tray),
            Err(error) => tracing::warn!("tray icon unavailable: {error}"),
        }

        let half_size = Vec2::new(
            loaded_pet.sprite_size.0 as f32,
            loaded_pet.sprite_size.1 as f32,
        ) * 0.5;

        window.set_outer_position(initial_position - half_size);

        self.window_center = half_size;
        self.last_synced_position = initial_position;

        self.window_state = Some(ApplicationWindow::new(
            window.id(),
            loaded_pet.sprite_size.0,
            loaded_pet.sprite_size.1,
        ));

        self.companion = Some(loaded_pet.companion);
        self.alpha_mask = Some(loaded_pet.alpha_mask);
        self.texture_handle = Some(loaded_pet.texture_handle);
        self.renderer = Some(renderer);

        self.sync_click_through();

        self.last_tick = Instant::now();

        window.set_visible(self.visible);
        window.request_redraw();
    }

    fn window_event(&mut self, window: &Window, event: WindowEvent) {
        if self
            .window_state
            .as_ref()
            .is_some_and(|state| state.id() != window.id())
        {
            return;
        }

        match event {
            WindowEvent::CursorMoved { position } => {
                let Some(companion) = self.companion.as_mut() else {
                    return;
                };

                let global_cursor = self
                    .platform
                    .as_ref()
                    .and_then(PlatformWindowExt::global_cursor_position)
                    .map(|(gx, gy)| Vec2::new(gx as f32, gy as f32));

                let outcome = self.input.handle_cursor_moved(
                    position,
                    companion,
                    self.alpha_mask.as_ref(),
                    self.window_center,
                    global_cursor,
                );

                Self::log_events(&outcome.events);
                self.sync_click_through();

                self.sync_window_position(window);

                if outcome.redraw {
                    window.request_redraw();
                }
            }

            WindowEvent::CursorLeft => {
                self.input.handle_cursor_left();
                self.sync_click_through();
            }

            WindowEvent::MouseInput { button, pressed } => {
                let Some(companion) = self.companion.as_mut() else {
                    return;
                };

                let cursor = self
                    .platform
                    .as_ref()
                    .and_then(PlatformWindowExt::cursor_position)
                    .map(|(x, y)| Vec2::new(x as f32, y as f32));

                let global_cursor = self
                    .platform
                    .as_ref()
                    .and_then(PlatformWindowExt::global_cursor_position)
                    .map(|(x, y)| Vec2::new(x as f32, y as f32));

                let outcome = self.input.handle_mouse_button(
                    button,
                    pressed,
                    InputContext {
                        companion,
                        alpha_mask: self.alpha_mask.as_ref(),
                        window_center: self.window_center,
                        cursor,
                        global_cursor,
                    },
                );

                if outcome.wake_requested {
                    Self::log_events(&companion.request_state(CompanionState::Idle));
                    self.sync_click_through();
                    window.request_redraw();
                    return;
                }

                let dragged_ended = outcome.events.contains(&CompanionEvent::DragEnded);

                Self::log_events(&outcome.events);

                if dragged_ended {
                    self.save_state();
                }

                if outcome.redraw {
                    window.request_redraw();
                }
            }

            other => {
                let Some(app_window) = self.window_state.as_mut() else {
                    return;
                };

                match app_window.handle_event(other) {
                    WindowCommand::Close => self.save_state(),

                    WindowCommand::Resize { width, height } => {
                        if let Some(renderer) = self.renderer.as_mut() {
                            renderer.resize(width, height);
                        }
                    }

                    WindowCommand::Redraw => self.redraw(window),

                    WindowCommand::Ignored => {}
                }
            }
        }
    }

    fn about_to_wait(&mut self, window: &Window) -> Option<Instant> {
        if let Some(tray) = self.tray.as_ref() {
            match tray.poll() {
                Some(TrayCommand::ToggleVisibility) => {
                    self.visible = !self.visible;
                    window.set_visible(self.visible);
                }
                Some(TrayCommand::Quit) => {
                    self.save_state();
                    std::process::exit(0);
                }
                None => {}
            }
        }

        let now = Instant::now();
        let dt = now.saturating_duration_since(self.last_tick);

        if dt < MIN_TICK_INTERVAL {
            return Some(self.last_tick + MIN_TICK_INTERVAL);
        }

        self.last_tick = now;

        self.poll_click_through();
        self.poll_sleep(now, window);

        if let Some(companion) = self.companion.as_mut() {
            let outcome = companion.update(dt, &self.monitors);

            let dragged_ended = outcome.events.contains(&CompanionEvent::DragEnded);

            Self::log_events(&outcome.events);

            if dragged_ended {
                self.save_state();
            }

            if outcome.redraw_needed {
                window.request_redraw();
            }
        }

        self.sync_window_position(window);

        Some(now + self.next_wait(now))
    }
}
