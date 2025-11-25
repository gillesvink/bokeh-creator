#[cfg(feature = "noise")]
use crate::Noise;
use crate::{FilterType, Settings};
use glam::{USizeVec2, Vec2};
#[cfg(feature = "image")]
use image::{ImageBuffer, Pixel};
#[cfg(feature = "noise")]
use libnoise::prelude::*;
#[cfg(feature = "image")]
use ndarray::ArrayViewMut3;

use image_ndarray::prelude::*;
use num_traits::AsPrimitive;

/// Value of RAD, as we don't have a const for that in f32::consts unfortunately
const RAD: f32 = 57.2957;
/// Make the ring size less extreme as its only meant to be for small values
const RING_NORMALIZER: f32 = 0.1;
/// Value to reduce the size of the outer blur
const OUTER_BLUR_LIMIT: f32 = 0.4;
/// Simple value that reduces the overall size of the kernel
const RADIUS_NORMALIZER: f32 = 0.3;

/// Object to render bokeh kernels
///
/// This is the disc that being perceived as the 'Bokeh'
/// This image will be used to convolute the entire image with.
///
///
/// ## Usage
///
/// The object has three entry points (depending on the features enabled).
///
/// Make sure to create the settings first, then pass them to this
/// object when calling `Renderer::new(settings);`
///
/// ### Rendering
/// When not using any features, call the `render_pixel()` method for all coordinates
/// in your image to fetch the result for each pixel.
///
/// Or provide your own ndarray where each value will be computed automatically with
/// `Renderer::render_to_array`
///
/// ### Example
/// ```rust
/// use bokeh_creator::{Renderer, Settings};
/// use glam::USizeVec2;
///
/// let resolution = 64;
/// let settings = Settings::default();
/// let renderer = Renderer::new(settings, [resolution, resolution].into());
/// let mut image = vec![vec![0.0; resolution]; resolution];
///
/// // this is not the most efficient way, its just to showcase basic image processing
/// for (y, row) in image.iter_mut().enumerate() {
///     for (x, pixel) in row.iter_mut().enumerate() {
///         *pixel = renderer.render_pixel(USizeVec2::new(x, y), 0);
///     }
/// }
/// ```
pub struct Renderer {
    /// Settings specified to use for rendering
    settings: Settings,
    /// Resolution of filter image
    resolution: USizeVec2,
    /// Center of the image in x and y
    center: Vec2,
    /// Degrees between each blade
    blade_degree: f32,
    /// Radius of kernel image in pixels
    radius_px: f32,

    #[cfg(feature = "noise")]
    /// Generator from libnoise to create noise with
    noise_generator: Fbm<2, Simplex<2>>,
}

impl Renderer {
    /// Create a new instance of the renderer with the specified settings.
    pub fn new(settings: Settings, resolution: USizeVec2) -> Self {
        let center = resolution.as_vec2() * 0.5;
        let blade_degree = Self::get_blade_degree(settings.blades);

        let radius_px = center * settings.radius - 1.0;
        let radius_px = radius_px - (radius_px * settings.outer_blur.abs() * RADIUS_NORMALIZER);
        Self {
            settings,
            center,
            resolution,
            blade_degree,
            radius_px: radius_px.max_element(),
            #[cfg(feature = "noise")]
            noise_generator: Self::get_noise_generator(settings.noise),
        }
    }

    /// Get the degrees towards the center of the kernel
    fn get_degrees(&self, position: Vec2) -> f32 {
        let relative_position = position - self.center;
        let radians = f32::atan2(relative_position.y, relative_position.x);
        radians * RAD + self.settings.angle
    }

    #[cfg(feature = "noise")]
    /// Configure the noise generator
    ///
    /// TODO: could be improved by specifying settings in the Settings struct to configure the type of noise.
    fn get_noise_generator(settings: Noise) -> Fbm<2, Simplex<2>> {
        Source::simplex(settings.seed.max(0) as u64).fbm(settings.octaves as u32, 0.013, 2.0, 0.5)
    }

    /// To get the blades added, we shift the radius a bit between the blades.
    ///
    /// The amount of curvature defines the amount of radius shift.
    fn get_blade_radius_multiplier(&self, position: Vec2) -> f32 {
        let degrees = self.get_degrees(position);
        let mut blades_offset = ((degrees)
            - (f32::floor(degrees / self.blade_degree) * self.blade_degree))
            / self.blade_degree;
        blades_offset -= 0.5;
        blades_offset = blades_offset.abs();

        let curvature = match self.settings.filter_type() {
            FilterType::Disc => 1.0,
            _ => self.settings.curvature,
        };

        (blades_offset - (blades_offset * blades_offset)) * (1.0 - f32::min(curvature, 1.0)) * 2.0
    }

    /// Simple screen operation (add to image without brightening)
    fn screen(a: f32, b: f32) -> f32 {
        a + b - (a * b)
    }

    /// Calculate the value of the ring, by percentage of the radius.
    fn get_ring_value(&self, pixel_percentage: f32) -> f32 {
        let ring_range = RING_NORMALIZER * self.settings.ring_size;
        let mut ring_multiplier = 1.0 - pixel_percentage;
        ring_multiplier = if ring_multiplier < ring_range && ring_multiplier > 0.0 {
            1.0
        } else {
            0.0
        };
        let mut inner_blur_multiplier = 0.0;
        if self.settings.inner_blur != 0.0 && pixel_percentage < 1.0 {
            inner_blur_multiplier = pixel_percentage / (1.0 - ring_range);
            inner_blur_multiplier = inner_blur_multiplier.clamp(0.0, 1.0);
            inner_blur_multiplier = (inner_blur_multiplier
                - (1.0 - (self.settings.inner_blur * 2.0)))
                / (1.0 - (1.0 - (self.settings.inner_blur * 2.0)));
            inner_blur_multiplier = inner_blur_multiplier.clamp(0.0, 1.0);
            inner_blur_multiplier = inner_blur_multiplier * inner_blur_multiplier;
        }
        let mut outer_blur_multiplier = 0.0;
        if self.settings.outer_blur != 0.0 && pixel_percentage > 1.0 {
            outer_blur_multiplier = (pixel_percentage
                - (1.0 + (self.settings.outer_blur.abs() * OUTER_BLUR_LIMIT)))
                / (1.0 - (1.0 + (self.settings.outer_blur.abs() * OUTER_BLUR_LIMIT)));
            outer_blur_multiplier = outer_blur_multiplier.clamp(0.0, 1.0);
            outer_blur_multiplier = outer_blur_multiplier * outer_blur_multiplier;
        }
        ring_multiplier = Self::screen(ring_multiplier, inner_blur_multiplier);
        ring_multiplier = Self::screen(ring_multiplier, outer_blur_multiplier);
        ring_multiplier
    }

    /// Returns 1 if its within the range of the kernel
    fn get_inner_value(pixel_percentage: f32) -> f32 {
        if pixel_percentage < 1.0 {
            return 1.0;
        }
        0.0
    }

    /// Get degrees per blade
    fn get_blade_degree(blades: u32) -> f32 {
        if blades == 0 {
            return 0.0;
        }
        360.0 / blades as f32
    }

    /// Calculate the value of both ring and inner color.
    ///
    /// TODO: implement channel support
    fn get_bokeh_value(&self, position: Vec2, _channel: usize) -> f32 {
        let radius_multiplier = self.get_blade_radius_multiplier(position);
        let calculated_radius = f32::max(
            self.radius_px - ((self.radius_px / (self.settings.blades as f32)) * radius_multiplier),
            0.0,
        );
        let pixel_percentage = position.distance(self.center).abs() / calculated_radius;
        let ring = self.get_ring_value(pixel_percentage);
        let inner = f32::max(Self::get_inner_value(pixel_percentage) - ring, 0.0);
        Self::screen(
            ring * self.settings.ring_color,
            inner * self.settings.inner_color,
        )
    }

    /// Render a single pixel and include noise if the `noise` feature is enabled.
    pub fn render_pixel(&self, position: USizeVec2, channel: usize) -> f32 {
        let offset_multiplier = Vec2::new(
            3.0 - f32::min(self.settings.aspect_ratio, 1.0) * 2.0,
            f32::max(self.settings.aspect_ratio, 1.0) * 2.0 - 1.0,
        );
        let coordinate = position.as_vec2() * offset_multiplier;
        let bokeh = self.get_bokeh_value(coordinate, channel);
        if self.settings.noise.intensity == 0.0 || self.settings.noise.size == 0.0 {
            return bokeh;
        }

        self.apply_noise(position, offset_multiplier, bokeh)
    }

    #[cfg(feature = "noise")]
    /// Apply noise to the bokeh value
    fn apply_noise(&self, position: USizeVec2, offset_multiplier: Vec2, bokeh: f32) -> f32 {
        let frequency = 1.0 + (1.0 / (self.settings.noise.size * 0.01));
        let noise_sample_position =
            (position.as_vec2() - self.center) * offset_multiplier * frequency
                / self.resolution.as_vec2();

        let mut noise = self.noise_generator.sample([
            noise_sample_position.x as f64,
            noise_sample_position.y as f64,
        ]) as f32;
        noise = noise.clamp(-1.0, 1.0);
        noise = (noise + 1.0) * 0.5;
        noise = noise.powf(2.2);
        noise * bokeh * self.settings.noise.intensity.clamp(0.0, 1.0)
            + (bokeh * (1.0 - self.settings.noise.intensity.clamp(0.0, 1.0)))
    }

    #[cfg(not(feature = "noise"))]
    /// Replacement for apply_noise if the feature is not enabled, doesn't do anything
    fn apply_noise(&self, _: USizeVec2, _: Vec2, bokeh: f32) -> f32 {
        bokeh
    }

    #[cfg(feature = "image")]
    /// Render the bokeh for the provided image.
    pub fn render_to_image<P, T>(image: &mut ImageBuffer<P, Vec<T>>, settings: Settings)
    where
        P: Pixel<Subpixel = T> + Sync,
        T: Clone + Copy + NormalizedFloat<T> + AsPrimitive<f32> + AsPrimitive<f64> + Default,
    {
        Self::render_to_array(settings, &mut image.as_ndarray_mut().view_mut());
    }

    pub fn render_to_array<T>(settings: Settings, target: &mut ArrayViewMut3<T>)
    where
        T: NormalizedFloat<T> + AsPrimitive<f32> + AsPrimitive<f64> + Default,
    {
        let resolution = USizeVec2::new(target.dim().1, target.dim().0);
        let renderer = Self::new(settings, resolution);
        target
            .indexed_iter_mut()
            .for_each(|((y, x, channel), value)| {
                *value =
                    T::from_f32_normalized(renderer.render_pixel(USizeVec2::new(x, y), channel))
                        .unwrap_or_default()
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Noise;
    use image::{DynamicImage, Rgba32FImage};
    use image_compare::Algorithm;
    use rstest::rstest;
    use std::path::PathBuf;

    /// Just a utility to make it easier to save the test results
    fn store_test_result(image: &Rgba32FImage, path: PathBuf) {
        DynamicImage::from(image.clone())
            .to_rgb8()
            .save(path)
            .unwrap();
    }

    fn get_comparison_score(a: Rgba32FImage, b: Rgba32FImage) -> f64 {
        let a = DynamicImage::from(a.clone()).to_luma8();
        let b = DynamicImage::from(b.clone()).to_luma8();

        image_compare::gray_similarity_structure(&Algorithm::MSSIMSimple, &a, &b)
            .unwrap()
            .score
    }

    fn load_test_image(path: PathBuf) -> Rgba32FImage {
        let image = image::open(path);
        image.unwrap().to_rgba32f()
    }

    #[rstest]
    #[case(Settings::default(), PathBuf::from("./test/images/1_expected.jpg"))]
    #[case(
        Settings {
            filter_type: FilterType::Blade.into(),
            angle: 195.3,
            curvature: 0.1,
            ..Default::default()
        },
        PathBuf::from("./test/images/2_expected.jpg")
    )]
    #[case(
        Settings {
            filter_type: FilterType::Blade.into(),
            angle: 90.0,
            blades: 15,
            ..Default::default()
        },
        PathBuf::from("./test/images/3_expected.jpg")
    )]
    #[case(
        Settings {
            aspect_ratio: 0.5,
            ..Default::default()
        },
        PathBuf::from("./test/images/4_expected.jpg")
    )]
    #[case(
        Settings {
            aspect_ratio: 2.0,
            ..Default::default()
        },
        PathBuf::from("./test/images/5_expected.jpg")
    )]
    #[case(
        Settings {
            ring_color: 0.5,
            inner_color: 0.9,
            ring_size: 0.5,
            ..Default::default()
        },
        PathBuf::from("./test/images/6_expected.jpg")
    )]
    #[case(
        Settings {
            noise: {
                Noise {
                    size: 0.3,
                    intensity: 1.0,
                    ..Default::default()
                }
            },
            ..Default::default()
        },
        PathBuf::from("./test/images/7_expected.jpg")
    )]
    #[case(
        Settings {
            noise: {
                Noise {
                    intensity: 0.0,
                    ..Default::default()
                }
            },
            ..Default::default()
        },
        PathBuf::from("./test/images/8_expected.jpg")
    )]
    #[case(
        Settings {
            noise: {
                Noise {
                    seed: 30,
                    ..Default::default()
                }
            },
            ..Default::default()
        },
        PathBuf::from("./test/images/9_expected.jpg")
    )]

    /// Test result of kernel rendering
    fn test_kernel(#[case] settings: Settings, #[case] expected: PathBuf) {
        let expected_image = match expected.exists() {
            true => load_test_image(expected.clone()),
            false => Rgba32FImage::new(256, 256),
        };

        let mut result = Rgba32FImage::new(256, 256);
        Renderer::render_to_image(&mut result, settings);

        if !(expected.clone().exists()) {
            store_test_result(&result, expected);
        }

        let score = get_comparison_score(expected_image, result);
        println!("Test got score: {}", score);

        assert!(score > 0.9); // Because of compression with jpegs :)
    }
}
