use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum AssetError {
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },
    ImageDecode {
        path: PathBuf,
        source: image::ImageError,
    },
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileRead { path, source } => {
                write!(f, "failed to read asset '{}': {source}", path.display())
            }
            Self::ImageDecode { path, source } => {
                write!(f, "failed to decode image '{}': {source}", path.display())
            }
        }
    }
}

impl std::error::Error for AssetError {}

#[derive(Debug, Clone)]
pub struct LoadedImage {
    width: u32,
    height: u32,
    rgba8_data: Vec<u8>,
}

impl LoadedImage {
    pub fn new(width: u32, height: u32, rgba8_data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            rgba8_data,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn rgba8_data(&self) -> &[u8] {
        &self.rgba8_data
    }

    pub fn into_rgba8_data(self) -> Vec<u8> {
        self.rgba8_data
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AssetLoader;

impl AssetLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn load<P: AsRef<Path>>(&self, path: P) -> Result<LoadedImage, AssetError> {
        let path = path.as_ref();

        let data = std::fs::read(path).map_err(|source| AssetError::FileRead {
            path: path.to_path_buf(),
            source,
        })?;

        let image = image::load_from_memory(&data).map_err(|source| AssetError::ImageDecode {
            path: path.to_path_buf(),
            source,
        })?;

        let rgba8 = image.to_rgba8();
        let (width, height) = rgba8.dimensions();

        Ok(LoadedImage::new(width, height, rgba8.into_raw()))
    }
}

#[cfg(test)]
mod tests {
    use super::AssetLoader;
    use std::io::Write;

    #[test]
    fn missing_file_returns_file_read_error() {
        let loader = AssetLoader::new();

        let result = loader.load("this-file-does-not-exist.png");

        assert!(matches!(result, Err(super::AssetError::FileRead { .. })));
    }

    #[test]
    fn invalid_image_returns_decode_error() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"not an image").unwrap();

        let loader = AssetLoader::new();
        let result = loader.load(file.path());

        assert!(matches!(result, Err(super::AssetError::ImageDecode { .. })));
    }
}
