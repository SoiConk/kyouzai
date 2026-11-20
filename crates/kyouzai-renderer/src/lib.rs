mod error;
mod geometry;
mod render_object;
mod renderer;
mod vertex;

pub use error::{RenderError, RendererInitError, TextureError};
pub use render_object::{RenderObject, RenderTransform, TextureHandle, TextureRegion};
pub use renderer::Renderer;

#[cfg(test)]
#[path = "tests/geometry_tests.rs"]
mod geometry_tests;
