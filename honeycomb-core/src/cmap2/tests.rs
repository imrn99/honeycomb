// ------ IMPORTS

use crate::{CMap2, FloatType, Orbit2, OrbitPolicy, Vertex2};

// ------ CONTENT

#[test]
fn example_test() {
    // build a triangle
    let mut map: CMap2<FloatType> = CMap2::new(3);
    map.one_link(1, 2);
    map.one_link(2, 3);
    map.one_link(3, 1);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (1.0, 0.0));
    map.insert_vertex(3, (0.0, 1.0));

    // checks
    let faces = map.fetch_faces();
    assert_eq!(faces.identifiers.len(), 1);
    assert_eq!(faces.identifiers[0], 1);
    let mut face = Orbit2::new(&map, OrbitPolicy::Face, 1);
    assert_eq!(face.next(), Some(1));
    assert_eq!(face.next(), Some(2));
    assert_eq!(face.next(), Some(3));
    assert_eq!(face.next(), None);

    // build a second triangle
    map.add_free_darts(3);
    map.one_link(4, 5);
    map.one_link(5, 6);
    map.one_link(6, 4);
    map.insert_vertex(4, (0.0, 2.0));
    map.insert_vertex(5, (2.0, 0.0));
    map.insert_vertex(6, (1.0, 1.0));

    // checks
    let faces = map.fetch_faces();
    assert_eq!(&faces.identifiers, &[1, 4]);
    let mut face = Orbit2::new(&map, OrbitPolicy::Face, 4);
    assert_eq!(face.next(), Some(4));
    assert_eq!(face.next(), Some(5));
    assert_eq!(face.next(), Some(6));
    assert_eq!(face.next(), None);

    // sew both triangles
    map.two_sew(2, 4);

    // checks
    assert_eq!(map.beta::<2>(2), 4);
    assert_eq!(map.vertex_id(2), 2);
    assert_eq!(map.vertex_id(5), 2);
    assert_eq!(map.vertex(2), Vertex2::from((1.5, 0.0)));
    assert_eq!(map.vertex_id(3), 3);
    assert_eq!(map.vertex_id(4), 3);
    assert_eq!(map.vertex(3), Vertex2::from((0.0, 1.5)));
    let edges = map.fetch_edges();
    assert_eq!(&edges.identifiers, &[1, 2, 3, 5, 6]);

    // adjust bottom-right & top-left vertex position
    assert_eq!(
        map.replace_vertex(2, Vertex2::from((1.0, 0.0))),
        Ok(Vertex2::from((1.5, 0.0)))
    );
    assert_eq!(map.vertex(2), Vertex2::from((1.0, 0.0)));
    assert_eq!(
        map.replace_vertex(3, Vertex2::from((0.0, 1.0))),
        Ok(Vertex2::from((0.0, 1.5)))
    );
    assert_eq!(map.vertex(3), Vertex2::from((0.0, 1.0)));

    // separate the diagonal from the rest
    map.one_unsew(1);
    map.one_unsew(2);
    map.one_unsew(6);
    map.one_unsew(4);
    // break up & remove the diagonal
    map.two_unsew(2); // this makes dart 2 and 4 free
    map.remove_free_dart(2);
    map.remove_free_dart(4);
    // sew the square back up
    map.one_sew(1, 5);
    map.one_sew(6, 3);

    // i-cells
    let faces = map.fetch_faces();
    assert_eq!(&faces.identifiers, &[1]);
    let edges = map.fetch_edges();
    assert_eq!(&edges.identifiers, &[1, 3, 5, 6]);
    let vertices = map.fetch_vertices();
    assert_eq!(&vertices.identifiers, &[1, 3, 5, 6]);
    assert_eq!(map.vertex(1), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.vertex(5), Vertex2::from((1.0, 0.0)));
    assert_eq!(map.vertex(6), Vertex2::from((1.0, 1.0)));
    assert_eq!(map.vertex(3), Vertex2::from((0.0, 1.0)));
    // darts
    assert_eq!(map.n_unused_darts(), 2); // there are unused darts since we removed the diagonal
    assert_eq!(map.beta_runtime(1, 1), 5);
    assert_eq!(map.beta_runtime(1, 5), 6);
    assert_eq!(map.beta_runtime(1, 6), 3);
    assert_eq!(map.beta_runtime(1, 3), 1);
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: UndefinedVertex")]
fn remove_vertex_twice() {
    // in its default state, all darts/vertices of a map are considered to be used
    let mut map: CMap2<FloatType> = CMap2::new(4);
    // set vertex 1 as unused
    map.remove_vertex(1).unwrap();
    // set vertex 1 as unused, again
    map.remove_vertex(1).unwrap(); // this should panic
}

#[test]
#[should_panic(expected = "assertion failed: self.unused_darts.insert(dart_id)")]
fn remove_dart_twice() {
    // in its default state, all darts/vertices of a map are considered to be used
    // darts are also free
    let mut map: CMap2<FloatType> = CMap2::new(4);
    // set dart 1 as unused
    map.remove_free_dart(1);
    // set dart 1 as unused, again
    map.remove_free_dart(1); // this should panic
}

#[test]
fn two_sew_complete() {
    let mut map: CMap2<FloatType> = CMap2::new(4);
    map.one_link(1, 2);
    map.one_link(3, 4);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (0.0, 1.0));
    map.insert_vertex(3, (1.0, 1.0));
    map.insert_vertex(4, (1.0, 0.0));
    map.two_sew(1, 3);
    assert_eq!(map.vertex(1), Vertex2::from((0.5, 0.0)));
    assert_eq!(map.vertex(2), Vertex2::from((0.5, 1.0)));
}

#[test]
fn two_sew_incomplete() {
    let mut map: CMap2<FloatType> = CMap2::new(3);
    map.one_link(1, 2);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (0.0, 1.0));
    map.insert_vertex(3, (1.0, 1.0));
    map.two_sew(1, 3);
    // missing beta1 image for dart 3
    assert_eq!(map.vertex(1), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.vertex(2), Vertex2::from((0.5, 1.0)));
    map.two_unsew(1);
    assert_eq!(map.add_free_dart(), 4);
    map.one_link(3, 4);
    map.two_sew(1, 3);
    // missing vertex for dart 4
    assert_eq!(map.vertex(1), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.vertex(2), Vertex2::from((0.5, 1.0)));
}

#[test]
fn two_sew_no_b1() {
    let mut map: CMap2<FloatType> = CMap2::new(2);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (1.0, 1.0));
    map.two_sew(1, 2);
    assert_eq!(map.vertex(1), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.vertex(2), Vertex2::from((1.0, 1.0)));
}

#[test]
#[should_panic(
    expected = "No vertices defined on either darts 1/2 , use `two_link` instead of `two_sew`"
)]
fn two_sew_no_attributes() {
    let mut map: CMap2<FloatType> = CMap2::new(2);
    map.two_sew(1, 2); // should panic
}

#[test]
#[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
fn two_sew_no_attributes_bis() {
    let mut map: CMap2<FloatType> = CMap2::new(4);
    map.one_link(1, 2);
    map.one_link(3, 4);
    map.two_sew(1, 3); // panic
}

#[test]
#[should_panic(expected = "Dart 1 and 3 do not have consistent orientation for 2-sewing")]
fn two_sew_bad_orientation() {
    let mut map: CMap2<FloatType> = CMap2::new(4);
    map.one_link(1, 2);
    map.one_link(3, 4);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (0.0, 1.0)); // 1->2 goes up
    map.insert_vertex(3, (1.0, 0.0));
    map.insert_vertex(4, (1.0, 1.0)); // 3->4 also goes up
    map.two_sew(1, 3); // panic
}

#[test]
fn one_sew_complete() {
    let mut map: CMap2<FloatType> = CMap2::new(3);
    map.two_link(1, 2);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (0.0, 1.0));
    map.insert_vertex(3, (0.0, 2.0));
    map.one_sew(1, 3);
    assert_eq!(map.vertex(2), Vertex2::from((0.0, 1.5)));
}

#[test]
fn one_sew_incomplete_attributes() {
    let mut map: CMap2<FloatType> = CMap2::new(3);
    map.two_link(1, 2);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (0.0, 1.0));
    map.one_sew(1, 3);
    assert_eq!(map.vertex(2), Vertex2::from((0.0, 1.0)));
}

#[test]
fn one_sew_incomplete_beta() {
    let mut map: CMap2<FloatType> = CMap2::new(3);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (0.0, 1.0));
    map.one_sew(1, 2);
    assert_eq!(map.vertex(2), Vertex2::from((0.0, 1.0)));
}
#[test]
#[should_panic(expected = "No vertex defined on dart 2, use `one_link` instead of `one_sew`")]
fn one_sew_no_attributes() {
    let mut map: CMap2<FloatType> = CMap2::new(2);
    map.one_sew(1, 2); // should panic
}

#[test]
#[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
fn one_sew_no_attributes_bis() {
    let mut map: CMap2<FloatType> = CMap2::new(3);
    map.two_link(1, 2);
    map.one_sew(1, 3); // panic
}
