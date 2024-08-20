use honeycomb_kernels::{grisubal, Clip};
use honeycomb_render::{RenderParameters, SmaaMode};

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        let clip = if let Some(val) = args.get(2) {
            match val.as_ref() {
                "left" => Clip::Left,
                "right" => Clip::Right,
                _ => {
                    println!("W: unrecognised clip argument - running kernel without clipping");
                    Clip::None
                }
            }
        } else {
            Clip::None
        };

        let map = grisubal::<f64>(path, [1., 1.], clip).unwrap();

        let render_params = RenderParameters {
            smaa_mode: SmaaMode::Smaa1X,
            relative_resize: false,
            shrink_factor: 0.05,
            arrow_headsize: 0.01,
            arrow_thickness: 0.005,
        };

        honeycomb_render::launch(render_params, Some(&map));
    } else {
        println!("No input geometry specified - you can pass a path to a vtk input as command line argument")
    }
}
