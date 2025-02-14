use honeycomb::prelude::{
    grisubal::{grisubal, Clip},
    CMap2, CoordsFloat,
};

use crate::cli::GrisubalArgs;

impl From<crate::cli::Clip> for Clip {
    fn from(value: crate::cli::Clip) -> Self {
        match value {
            crate::cli::Clip::Left => Clip::Left,
            crate::cli::Clip::Right => Clip::Right,
        }
    }
}

pub fn bench_grisubal<T: CoordsFloat>(args: GrisubalArgs) -> CMap2<T> {
    grisubal(
        args.input,
        [T::from(args.lx).unwrap(), T::from(args.ly).unwrap()],
        args.clip.map(Clip::from).unwrap_or_default(),
    )
    .unwrap()
}
