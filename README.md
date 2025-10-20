[![Tests](https://github.com/gillesvink/bokeh-creator/actions/workflows/test.yaml/badge.svg)](https://github.com/gillesvink/bokeh-creator/actions/workflows/test.yaml) 
[![License](https://img.shields.io/crates/l/bokeh-creator)](https://crates.io/crates/bokeh-creator) 
[![Version](https://img.shields.io/crates/v/bokeh-creator)](https://crates.io/crates/bokeh-creator) 


# Bokeh Creator

Simple library to replicate real world lens kernels written in Rust. Currently it requires the `std` library.

## Install
Add this crate to your project by adding it in your `Cargo.toml`:
```bash
cargo add bokeh-creator
# or when using rayon for multithreaded rendering
cargo add --features rayon
```

The [Image](https://crates.io/crates/image) crate is optional, as there is a method to render when you specify coordinates only.


## Usage

```rust
use bokeh_creator::{Renderer, Settings};
use image::Rgba32FImage;
use glam::UVec2;

fn main() {
    let settings = Settings {
        resolution: UVec2::new(256, 256),
        angle: 195.3,
        curvature: 0.1,
        ..Default::default()
    };
    let renderer = Renderer::new(settings);    
    let result: Rgba32FImage = renderer.render_image();
    // Do whatever you need to do with the result :)
}
```


## Examples

![1_expected.jpg](https://codeberg.org/gillesvink/bokeh-creator/raw/branch/main/test/images/1_expected.jpg) 
```rust
use bokeh_creator::Settings;
Settings::default();
```

![2_expected.jpg](https://codeberg.org/gillesvink/bokeh-creator/raw/branch/main/test/images/2_expected.jpg) 
```rust
use bokeh_creator::{FilterType, Settings};
Settings {
    filter_type: FilterType::BLADE,
    angle: 195.3,
    curvature: 0.1,
    ..Default::default()
};
```

![3_expected.jpg](https://codeberg.org/gillesvink/bokeh-creator/raw/branch/main/test/images/3_expected.jpg) 
```rust
use bokeh_creator::{FilterType, Settings};
Settings {
    filter_type: FilterType::BLADE,
    angle: 90.0,
    blades: 15,
    ..Default::default()
};
```


![4_expected.jpg](https://codeberg.org/gillesvink/bokeh-creator/raw/branch/main/test/images/4_expected.jpg) 
```rust
use bokeh_creator::Settings;
Settings {
    aspect_ratio: 0.5,
    ..Default::default()
};
```

![5_expected.jpg](https://codeberg.org/gillesvink/bokeh-creator/raw/branch/main/test/images/5_expected.jpg) 
```rust
use bokeh_creator::Settings;
Settings {
    aspect_ratio: 2.0,
    ..Default::default()
};
```

![6_expected.jpg](https://codeberg.org/gillesvink/bokeh-creator/raw/branch/main/test/images/6_expected.jpg) 
```rust
use bokeh_creator::Settings;
Settings {
    ring_color: 0.5,
    inner_color: 0.9,
    ring_size: 0.5,
    ..Default::default()
};
```

![7_expected.jpg](https://codeberg.org/gillesvink/bokeh-creator/raw/branch/main/test/images/7_expected.jpg) 
```rust
use bokeh_creator::{Noise, Settings};
Settings {
    noise: {
        Noise { 
            size: 0.3, 
            intensity: 1.0, 
            ..Default::default()
        }
    },
    ..Default::default()
};
```

![8_expected.jpg](https://codeberg.org/gillesvink/bokeh-creator/raw/branch/main/test/images/8_expected.jpg) 
```rust
use bokeh_creator::{Noise, Settings};
Settings {
    noise: {
        Noise { 
            intensity: 0.0, 
            ..Default::default()
        }
    },
    ..Default::default()
};
```

![9_expected.jpg](https://codeberg.org/gillesvink/bokeh-creator/raw/branch/main/test/images/9_expected.jpg) 
```rust
use bokeh_creator::{Noise, Settings};
Settings {
    noise: {
        Noise { 
            seed: 30, 
            ..Default::default()
        }
    },
    ..Default::default()
};
```
