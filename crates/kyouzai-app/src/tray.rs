//! System tray integration.
//!
//! Menu events are delivered through tray-icon's global channel, so Application
//! polls this module from its event loop.

use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

const ICON_BYTES: &[u8] = include_bytes!("../../../assets/tray-icon.png");

const TOGGLE_VISIBILITY_ID: &str = "toggle_visibility";
const QUIT_ID: &str = "quit";

#[derive(Debug, Clone, Copy)]
pub enum TrayCommand {
    ToggleVisibility,
    Quit,
}

#[derive(Debug)]
pub enum TrayError {
    IconDecode(image::ImageError),
    BadIcon(tray_icon::BadIcon),
    Menu(tray_icon::menu::Error),
    Build(tray_icon::Error),
}

impl std::fmt::Display for TrayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IconDecode(error) => {
                write!(f, "failed to decode tray icon image: {error}")
            }
            Self::BadIcon(error) => {
                write!(f, "failed to create tray icon data: {error}")
            }
            Self::Menu(error) => {
                write!(f, "failed to create tray menu: {error}")
            }
            Self::Build(error) => {
                write!(f, "failed to create tray icon: {error}")
            }
        }
    }
}

impl std::error::Error for TrayError {}

pub struct Tray {
    // Keeping the icon alive keeps it visible in the system tray.
    _icon: TrayIcon,
}

impl Tray {
    pub fn new() -> Result<Self, TrayError> {
        let icon = load_icon()?;
        let menu = build_menu()?;

        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .with_tooltip("Kyouzai")
            .build()
            .map_err(TrayError::Build)?;

        Ok(Self { _icon: icon })
    }

    /// Returns the next command belonging to this tray, if one is available.
    ///
    /// Unrelated events from tray-icon's global channel are discarded.
    pub fn poll(&self) -> Option<TrayCommand> {
        let receiver = MenuEvent::receiver();

        loop {
            let event = receiver.try_recv().ok()?;

            let command = match event.id().as_ref() {
                TOGGLE_VISIBILITY_ID => TrayCommand::ToggleVisibility,
                QUIT_ID => TrayCommand::Quit,
                _ => continue,
            };

            return Some(command);
        }
    }
}

fn build_menu() -> Result<Menu, TrayError> {
    let menu = Menu::new();

    let toggle_item = MenuItem::with_id(TOGGLE_VISIBILITY_ID, "Show/Hide", true, None);
    let quit_item = MenuItem::with_id(QUIT_ID, "Quit", true, None);

    menu.append_items(&[&toggle_item, &quit_item])
        .map_err(TrayError::Menu)?;

    Ok(menu)
}

fn load_icon() -> Result<Icon, TrayError> {
    let decoded = image::load_from_memory(ICON_BYTES)
        .map_err(TrayError::IconDecode)?
        .into_rgba8();

    let (width, height) = (decoded.width(), decoded.height());

    Icon::from_rgba(decoded.into_raw(), width, height).map_err(TrayError::BadIcon)
}
