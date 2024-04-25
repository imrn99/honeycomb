//! These benchmarks uses iai-callgrind to fetch data from hardware counter
//! & provide accurate insights into the code behavior independently from
//! available computing power.
//!
//! This file contains benchmarks of key editing methods and constructors,
//! classfied into three groups:
//!
//! - `bench_new`: benches constructors and "add" methods.
//! - `bench_insert`: benches insertion methods (both behaviors).
//! - `bench_face_building`: benches the face building methods.
//!
//! Each benchmark is repeated on CMap2 of different sizes.
//!

// ------ IMPORTS

use honeycomb_core::{utils::GridBuilder, CMap2, DartIdentifier, FloatType, Vertex2};
use iai_callgrind::{
    library_benchmark, library_benchmark_group, main, FlamegraphConfig, LibraryBenchmarkConfig,
};
use std::hint::black_box;

// ------ CONTENT

fn get_map(n_square: usize) -> CMap2<FloatType> {
    GridBuilder::unit_squares(n_square).build2().unwrap()
}

fn get_sparse_map(n_square: usize) -> CMap2<FloatType> {
    let mut map = GridBuilder::unit_squares(n_square).build2().unwrap();
    map.set_betas(5, [0; 3]); // free dart 5
    map.remove_free_dart(5);
    // because of the way we built the map in the square_cmap2 function & the ID computation
    // policy, we can safely remove a vertex we know is defined
    assert_eq!(map.remove_vertex(1).unwrap(), Vertex2::from((0.0, 0.0)));
    map
}

fn compute_dims(n_square: usize) -> usize {
    n_square.pow(2) * 4
}

#[library_benchmark]
#[benches::with_setup(args = [16, 32, 64, 128, 256, 512], setup = compute_dims)]
fn constructor(n_darts: usize) -> CMap2<FloatType> {
    black_box(CMap2::new(n_darts))
}

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn add_single_dart(map: &mut CMap2<FloatType>) -> DartIdentifier {
    black_box(map.add_free_dart())
}

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn add_ten_darts(map: &mut CMap2<FloatType>) -> DartIdentifier {
    black_box(map.add_free_darts(10))
}

library_benchmark_group!(
    name = bench_new;
    benchmarks =
        constructor,
        add_single_dart,
        add_ten_darts,
);

#[library_benchmark]
#[bench::small(&mut get_sparse_map(5))]
#[bench::medium(&mut get_sparse_map(50))]
#[bench::large(&mut get_sparse_map(500))]
fn insert_dart(map: &mut CMap2<FloatType>) -> DartIdentifier {
    black_box(map.insert_free_dart())
}

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn insert_dart_full(map: &mut CMap2<FloatType>) -> DartIdentifier {
    black_box(map.insert_free_dart())
}

#[library_benchmark]
#[bench::small(&mut get_sparse_map(5))]
#[bench::medium(&mut get_sparse_map(50))]
#[bench::large(&mut get_sparse_map(500))]
fn insert_vertex(map: &mut CMap2<FloatType>) {
    black_box(map.insert_vertex(1, (0.0, 0.0)));
}

library_benchmark_group!(
    name = bench_insert;
    benchmarks =
        insert_dart,
        insert_dart_full,
        insert_vertex,
);

#[library_benchmark]
#[bench::small(&mut get_map(5))]
#[bench::medium(&mut get_map(50))]
#[bench::large(&mut get_map(500))]
fn build_faces(map: &mut CMap2<FloatType>) {
    black_box(map.fetch_faces());
}

library_benchmark_group!(
    name = bench_face_building;
    benchmarks =
        build_faces,
);

main!(
    config = LibraryBenchmarkConfig::default().flamegraph(FlamegraphConfig::default());
    library_benchmark_groups = bench_new,
    bench_insert,
    bench_face_building,
);
