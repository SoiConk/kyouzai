use kyouzai_asset::LoadedImage;

#[derive(Debug, Clone)]
pub struct AlphaMask {
    width: u32,
    height: u32,
    alpha: Vec<u8>,
}

impl AlphaMask {
    pub fn from_image(image: &LoadedImage) -> Self {
        let alpha = image
            .rgba8_data()
            .iter()
            .skip(3)
            .step_by(4)
            .copied()
            .collect();

        Self {
            width: image.width(),
            height: image.height(),
            alpha,
        }
    }

    pub fn sample(&self, u: f32, v: f32) -> u8 {
        if !u.is_finite() || !v.is_finite() {
            return 0;
        }
        if self.width == 0 || self.height == 0 {
            return 0;
        }

        let x = (u.clamp(0.0, 1.0) * self.width as f32) as u32;
        let y = (v.clamp(0.0, 1.0) * self.height as f32) as u32;

        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);

        let index = (y as usize) * (self.width as usize) + x as usize;
        self.alpha.get(index).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_the_correct_pixels_alpha() {
        let image = LoadedImage::new(
            2,
            2,
            vec![
                255, 0, 0, 255, // opaque
                0, 255, 0, 128, // half-transparent
                0, 0, 255, 0, // transparent
                255, 255, 0, 64, // partially transparent
            ],
        );

        let mask = AlphaMask::from_image(&image);

        assert_eq!(mask.sample(0.0, 0.0), 255);
        assert_eq!(mask.sample(1.0, 0.0), 128);
        assert_eq!(mask.sample(0.0, 1.0), 0);
        assert_eq!(mask.sample(1.0, 1.0), 64);
    }

    #[test]
    fn out_of_range_coordinates_are_clamped() {
        let image = LoadedImage::new(2, 1, vec![255, 0, 0, 100, 0, 255, 0, 200]);

        let mask = AlphaMask::from_image(&image);

        assert_eq!(mask.sample(-1.0, 0.0), 100);
        assert_eq!(mask.sample(2.0, 0.0), 200);
    }
}
