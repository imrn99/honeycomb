//! # honeycomb-benches
//!
//! This crate contains all benchmarks of the project. It also contains simple binaries used to
//! profile and further optimize the implementation.
//!
//! ## Available benchmarks
//!
//! ### Criterion-based
//!
//! - `builder` - grid building routines at fixed size
//! - `builder-grid-size` - grid building routines over a range of grid sizes
//! - `fetch_icells` - `CMap2::iter_<CELL>` methods
//! - `grisubal` - grisubal kernel with a fixed size grid
//! - `grisubal-grid-size` - grisubal kernel over a range of grid granularity
//! - `triangulate-quads` - triangulate all cells of a mixed-mesh
//!
//! ### Iai-callgrind-based
//!
//! - `prof-dim2-basic` - `CMap2` basic operations benchmarks
//! - `prof-dim2-build` - `CMap2` constructor & building functions benchmarks
//! - `prof-dim2-sewing-unsewing` - `CMap2` (un)sewing & (un)linking methods benchmarks
//!
//! ## Available binaries
//!
//! - `builder` - Build a 2-map grid using dimensions passed as argument
//! - `grisubal` - Run the `grisubal` algorithm
//! - `shift` - Run a simple vertex relaxation algorithm in parallel (naively)
//! - `shift-nc` - Run a simple vertex relaxation algorithm in parallel (using independent set of
//!   vertices)

use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::Read;

cfg_if::cfg_if! {
    if #[cfg(feature = "_single_precision")] {
        /// Floating-point type alias.
        ///
        /// This is mostly used to run tests using both `f64` and `f32`.
        pub type FloatType = f32;
    } else {
        /// Floating-point type alias.
        ///
        /// This is mostly used to run tests using both `f64` and `f32`.
        pub type FloatType = f64;
    }
}

#[doc(hidden)]
pub fn hash_file(path: &str) -> Result<u64, std::io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = DefaultHasher::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.write(&buffer[..bytes_read]);
    }

    Ok(hasher.finish())
}
