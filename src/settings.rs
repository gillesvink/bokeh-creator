/// Enum that contains the type of filter used for the bokeh.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(C)]
pub enum FilterType {
    /// Circular bokeh (basically same as curvature set to 1.0)
    DISC,
    /// Blade-based bokeh (for replicating camera blades)
    BLADE,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// Noise specific settings.
///
/// These settings can help to add some artifacts to the kernel,
/// to replicate more natural-like bokeh's.
pub struct Noise {
    /// Size of the noise pattern
    pub size: f32,
    /// Intensity of the noise
    pub intensity: f32,
    /// Number of octaves for noise generation
    pub octaves: i32,
    /// Random seed for noise generation
    pub seed: i32,
}

impl Default for Noise {
    fn default() -> Noise {
        Noise {
            size: 0.1,
            intensity: 0.25,
            octaves: 7,
            seed: 0,
        }
    }
}

/// Data object that contains all bokeh settings.
///
/// This object is used to specify the settings when rendering.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Settings {
    /// Type of filter to use for the bokeh effect
    pub filter_type: FilterType,
    /// Radius of the bokeh effect
    pub radius: f32,
    /// Color of the ring in the bokeh effect
    pub ring_color: f32,
    /// Color of the inner part of the bokeh effect
    pub inner_color: f32,
    /// Size of the ring in the bokeh effect
    pub ring_size: f32,
    /// Amount of blur applied to the outer part of the bokeh
    pub outer_blur: f32,
    /// Amount of blur applied to the inner part of the bokeh
    pub inner_blur: f32,
    /// Number of blades for blade-based bokeh filters
    pub blades: i32,
    /// Angle of the bokeh effect
    pub angle: f32,
    /// Curvature of the bokeh effect
    pub curvature: f32,
    /// Noise settings for the bokeh effect
    pub noise: Noise,
    /// Aspect ratio for the bokeh effect
    pub aspect_ratio: f32,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            // default is just a natural looking bokeh
            filter_type: FilterType::DISC,
            radius: 1.0,
            ring_color: 1.0,
            inner_color: 0.4,
            ring_size: 0.1,
            outer_blur: 0.1,
            inner_blur: 0.05,
            blades: 5,
            angle: 0.0,
            curvature: 0.5,
            noise: Noise::default(),
            aspect_ratio: 1.0,
        }
    }
}
