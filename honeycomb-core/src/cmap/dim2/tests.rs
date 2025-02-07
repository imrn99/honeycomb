// ------ IMPORTS

use crate::{
    attributes::{AttrSparseVec, AttributeError},
    cmap::{LinkError, SewError, VertexIdType},
    prelude::{AttributeBind, AttributeUpdate, CMap2, CMapBuilder, Orbit2, OrbitPolicy, Vertex2},
    stm::{atomically, atomically_with_err, StmError, TransactionError},
};

// ------ CONTENT

// --- GENERAL

#[test]
fn example_test() {
    // build a triangle
    let mut map: CMap2<f64> = CMapBuilder::default().n_darts(3).build().unwrap();
    map.force_link::<1>(1, 2).unwrap();
    map.force_link::<1>(2, 3).unwrap();
    map.force_link::<1>(3, 1).unwrap();
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (1.0, 0.0));
    map.force_write_vertex(3, (0.0, 1.0));

    // checks
    let faces: Vec<_> = map.iter_faces().collect();
    assert_eq!(faces.len(), 1);
    assert_eq!(faces[0], 1);
    let mut face = Orbit2::new(&map, OrbitPolicy::Face, 1);
    assert_eq!(face.next(), Some(1));
    assert_eq!(face.next(), Some(2));
    assert_eq!(face.next(), Some(3));
    assert_eq!(face.next(), None);

    // build a second triangle
    map.add_free_darts(3);
    map.force_link::<1>(4, 5).unwrap();
    map.force_link::<1>(5, 6).unwrap();
    map.force_link::<1>(6, 4).unwrap();
    map.force_write_vertex(4, (0.0, 2.0));
    map.force_write_vertex(5, (2.0, 0.0));
    map.force_write_vertex(6, (1.0, 1.0));

    // checks
    let faces: Vec<_> = map.iter_faces().collect();
    assert_eq!(&faces, &[1, 4]);
    let mut face = Orbit2::new(&map, OrbitPolicy::Face, 4);
    assert_eq!(face.next(), Some(4));
    assert_eq!(face.next(), Some(5));
    assert_eq!(face.next(), Some(6));
    assert_eq!(face.next(), None);

    // sew both triangles
    map.force_sew::<2>(2, 4).unwrap();

    // checks
    assert_eq!(map.beta::<2>(2), 4);
    assert_eq!(map.vertex_id(2), 2);
    assert_eq!(map.vertex_id(5), 2);
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((1.5, 0.0)));
    assert_eq!(map.vertex_id(3), 3);
    assert_eq!(map.vertex_id(4), 3);
    assert_eq!(map.force_read_vertex(3).unwrap(), Vertex2::from((0.0, 1.5)));
    let edges: Vec<_> = map.iter_edges().collect();
    assert_eq!(&edges, &[1, 2, 3, 5, 6]);

    // adjust bottom-right & top-left vertex position
    assert_eq!(
        map.force_write_vertex(2, Vertex2::from((1.0, 0.0))),
        Some(Vertex2::from((1.5, 0.0)))
    );
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((1.0, 0.0)));
    assert_eq!(
        map.force_write_vertex(3, Vertex2::from((0.0, 1.0))),
        Some(Vertex2::from((0.0, 1.5)))
    );
    assert_eq!(map.force_read_vertex(3).unwrap(), Vertex2::from((0.0, 1.0)));

    // separate the diagonal from the rest
    map.force_unsew::<1>(1).unwrap();
    map.force_unsew::<1>(2).unwrap();
    map.force_unsew::<1>(6).unwrap();
    map.force_unsew::<1>(4).unwrap();
    // break up & remove the diagonal
    map.force_unsew::<2>(2).unwrap(); // this makes dart 2 and 4 free
    map.remove_free_dart(2);
    map.remove_free_dart(4);
    // sew the square back up
    map.force_sew::<1>(1, 5).unwrap();
    map.force_sew::<1>(6, 3).unwrap();

    // i-cells
    let faces: Vec<_> = map.iter_faces().collect();
    assert_eq!(&faces, &[1]);
    let edges: Vec<_> = map.iter_edges().collect();
    assert_eq!(&edges, &[1, 3, 5, 6]);
    let vertices: Vec<_> = map.iter_vertices().collect();
    assert_eq!(&vertices, &[1, 3, 5, 6]);
    assert_eq!(map.force_read_vertex(1).unwrap(), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.force_read_vertex(5).unwrap(), Vertex2::from((1.0, 0.0)));
    assert_eq!(map.force_read_vertex(6).unwrap(), Vertex2::from((1.0, 1.0)));
    assert_eq!(map.force_read_vertex(3).unwrap(), Vertex2::from((0.0, 1.0)));
    // darts
    assert_eq!(map.n_unused_darts(), 2); // there are unused darts since we removed the diagonal
    assert_eq!(map.beta_rt(1, 1), 5);
    assert_eq!(map.beta_rt(1, 5), 6);
    assert_eq!(map.beta_rt(1, 6), 3);
    assert_eq!(map.beta_rt(1, 3), 1);
}

#[test]
fn example_test_transactional() {
    // build a triangle
    let mut map: CMap2<f64> = CMapBuilder::default().n_darts(3).build().unwrap();
    let res = atomically_with_err(|trans| {
        map.link::<1>(trans, 1, 2)?;
        map.link::<1>(trans, 2, 3)?;
        map.link::<1>(trans, 3, 1)?;
        map.write_vertex(trans, 1, (0.0, 0.0))?;
        map.write_vertex(trans, 2, (1.0, 0.0))?;
        map.write_vertex(trans, 3, (0.0, 1.0))?;
        Ok(())
    });
    assert!(res.is_ok());

    // checks
    let faces: Vec<_> = map.iter_faces().collect();
    assert_eq!(faces.len(), 1);
    assert_eq!(faces[0], 1);
    let mut face = Orbit2::new(&map, OrbitPolicy::Face, 1);
    assert_eq!(face.next(), Some(1));
    assert_eq!(face.next(), Some(2));
    assert_eq!(face.next(), Some(3));
    assert_eq!(face.next(), None);

    // build a second triangle
    map.add_free_darts(3);
    let res = atomically_with_err(|trans| {
        map.link::<1>(trans, 4, 5)?;
        map.link::<1>(trans, 5, 6)?;
        map.link::<1>(trans, 6, 4)?;
        map.write_vertex(trans, 4, (0.0, 2.0))?;
        map.write_vertex(trans, 5, (2.0, 0.0))?;
        map.write_vertex(trans, 6, (1.0, 1.0))?;
        Ok(())
    });
    assert!(res.is_ok());

    // checks
    let faces: Vec<_> = map.iter_faces().collect();
    assert_eq!(&faces, &[1, 4]);
    let mut face = Orbit2::new(&map, OrbitPolicy::Face, 4);
    assert_eq!(face.next(), Some(4));
    assert_eq!(face.next(), Some(5));
    assert_eq!(face.next(), Some(6));
    assert_eq!(face.next(), None);

    // sew both triangles
    atomically(|trans| {
        // normally the erro should be handled, but we're in a seq context
        assert!(map.sew::<2>(trans, 2, 4).is_ok());
        Ok(())
    });

    // checks
    atomically(|trans| {
        assert_eq!(map.beta_transac::<2>(trans, 2)?, 4);
        assert_eq!(map.vertex_id_transac(trans, 2)?, 2);
        assert_eq!(map.vertex_id_transac(trans, 5)?, 2);
        assert_eq!(map.read_vertex(trans, 2)?, Some(Vertex2::from((1.5, 0.0))));
        assert_eq!(map.vertex_id_transac(trans, 3)?, 3);
        assert_eq!(map.vertex_id_transac(trans, 4)?, 3);
        assert_eq!(map.read_vertex(trans, 3)?, Some(Vertex2::from((0.0, 1.5))));
        Ok(())
    });
    let edges: Vec<_> = map.iter_edges().collect();
    assert_eq!(&edges, &[1, 2, 3, 5, 6]);

    // adjust bottom-right & top-left vertex position
    atomically(|trans| {
        assert_eq!(
            map.write_vertex(trans, 2, (1.0, 0.0))?,
            Some(Vertex2::from((1.5, 0.0)))
        );
        assert_eq!(map.read_vertex(trans, 2)?, Some(Vertex2::from((1.0, 0.0))));
        assert_eq!(
            map.write_vertex(trans, 3, (0.0, 1.0))?,
            Some(Vertex2::from((0.0, 1.5)))
        );
        assert_eq!(map.read_vertex(trans, 3)?, Some(Vertex2::from((0.0, 1.0))));
        Ok(())
    });

    // separate the diagonal from the rest
    atomically(|trans| {
        assert!(map.unsew::<1>(trans, 1).is_ok());
        assert!(map.unsew::<1>(trans, 2).is_ok());
        assert!(map.unsew::<1>(trans, 6).is_ok());
        assert!(map.unsew::<1>(trans, 4).is_ok());
        assert!(map.unsew::<2>(trans, 2).is_ok()); // this makes dart 2 and 4 free
        Ok(())
    });
    map.remove_free_dart(2);
    map.remove_free_dart(4);
    atomically(|trans| {
        // sew the square back up
        assert!(map.sew::<1>(trans, 1, 5).is_ok());
        assert!(map.sew::<1>(trans, 6, 3).is_ok());
        Ok(())
    });

    // i-cells
    let faces: Vec<_> = map.iter_faces().collect();
    assert_eq!(&faces, &[1]);
    let edges: Vec<_> = map.iter_edges().collect();
    assert_eq!(&edges, &[1, 3, 5, 6]);
    let vertices: Vec<_> = map.iter_vertices().collect();
    assert_eq!(&vertices, &[1, 3, 5, 6]);
    atomically(|trans| {
        assert_eq!(map.read_vertex(trans, 1)?, Some(Vertex2::from((0.0, 0.0))));
        assert_eq!(map.read_vertex(trans, 5)?, Some(Vertex2::from((1.0, 0.0))));
        assert_eq!(map.read_vertex(trans, 6)?, Some(Vertex2::from((1.0, 1.0))));
        assert_eq!(map.read_vertex(trans, 3)?, Some(Vertex2::from((0.0, 1.0))));
        Ok(())
    });
    // darts
    assert_eq!(map.n_unused_darts(), 2); // there are unused darts since we removed the diagonal
    atomically(|trans| {
        assert_eq!(map.beta_rt_transac(trans, 1, 1)?, 5);
        assert_eq!(map.beta_rt_transac(trans, 1, 5)?, 6);
        assert_eq!(map.beta_rt_transac(trans, 1, 6)?, 3);
        assert_eq!(map.beta_rt_transac(trans, 1, 3)?, 1);
        Ok(())
    });
}
#[test]
#[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
fn remove_vertex_twice() {
    // in its default state, all darts/vertices of a map are considered to be used
    let mut map: CMap2<f64> = CMap2::new(4);
    // set vertex 1 as unused
    map.force_remove_vertex(1).unwrap();
    // set vertex 1 as unused, again
    map.force_remove_vertex(1).unwrap(); // this should panic
}

#[test]
#[should_panic(expected = "assertion failed")]
fn remove_dart_twice() {
    // in its default state, all darts/vertices of a map are considered to be used
    // darts are also free
    let mut map: CMap2<f64> = CMap2::new(4);
    // set dart 1 as unused
    map.remove_free_dart(1);
    // set dart 1 as unused, again
    map.remove_free_dart(1); // this should panic
}

// --- (UN)SEW

#[test]
fn two_sew_complete() {
    let mut map: CMap2<f64> = CMap2::new(4);
    map.force_link::<1>(1, 2).unwrap();
    map.force_link::<1>(3, 4).unwrap();
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (0.0, 1.0));
    map.force_write_vertex(3, (1.0, 1.0));
    map.force_write_vertex(4, (1.0, 0.0));
    map.force_sew::<2>(1, 3).unwrap();
    assert_eq!(map.force_read_vertex(1).unwrap(), Vertex2::from((0.5, 0.0)));
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((0.5, 1.0)));
}

#[test]
fn two_sew_incomplete() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.force_link::<1>(1, 2).unwrap();
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (0.0, 1.0));
    map.force_write_vertex(3, (1.0, 1.0));
    map.force_sew::<2>(1, 3).unwrap();
    // missing beta1 image for dart 3
    assert_eq!(map.force_read_vertex(1).unwrap(), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((0.5, 1.0)));
    map.force_unsew::<2>(1).unwrap();
    assert_eq!(map.add_free_dart(), 4);
    map.force_link::<1>(3, 4).unwrap();
    map.force_sew::<2>(1, 3).unwrap();
    // missing vertex for dart 4
    assert_eq!(map.force_read_vertex(1).unwrap(), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((0.5, 1.0)));
}

#[test]
fn link_twice() {
    let mut map: CMap2<f64> = CMap2::new(3);
    assert!(map.force_link::<1>(1, 2).is_ok());
    assert!(map
        .force_link::<1>(1, 3)
        .is_err_and(|e| e == LinkError::NonFreeBase(1, 1, 3)));
    assert!(map
        .force_link::<1>(3, 2)
        .is_err_and(|e| e == LinkError::NonFreeImage(0, 3, 2)));
}

#[test]
fn sew_twice() {
    let mut map: CMap2<f64> = CMap2::new(3);
    assert!(map.force_link::<2>(1, 3).is_ok());
    map.force_write_vertex(3, (0.0, 0.0));
    map.force_write_vertex(2, (0.0, 0.0));
    assert!(map.force_sew::<1>(1, 2).is_ok());
    assert!(map
        .force_sew::<1>(1, 2)
        .is_err_and(|e| e == SewError::FailedLink(LinkError::NonFreeBase(1, 1, 2))));
}

#[test]
fn two_sew_no_b1() {
    let mut map: CMap2<f64> = CMap2::new(2);
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (1.0, 1.0));
    map.force_sew::<2>(1, 2).unwrap();
    assert_eq!(map.force_read_vertex(1).unwrap(), Vertex2::from((0.0, 0.0)));
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((1.0, 1.0)));
}

#[test]
fn two_sew_no_attributes() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.force_link::<2>(1, 3).unwrap();
    let res = atomically_with_err(|trans| map.sew::<1>(trans, 1, 2));
    assert!(res.is_err_and(|e| e
        == SewError::FailedAttributeOp(AttributeError::InsufficientData(
            "merge",
            std::any::type_name::<Vertex2<f64>>()
        ))));
    assert!(map.force_sew::<1>(1, 2).is_ok());
}

#[test]
fn two_sew_no_attributes_bis() {
    let mut map: CMap2<f64> = CMap2::new(4);
    map.force_link::<1>(1, 2).unwrap();
    map.force_link::<1>(3, 4).unwrap();
    let res = atomically_with_err(|trans| map.sew::<2>(trans, 1, 3));
    assert!(res.is_err_and(|e| e
        == SewError::FailedAttributeOp(AttributeError::InsufficientData(
            "merge",
            std::any::type_name::<Vertex2<f64>>()
        ))));
    assert!(map.force_sew::<2>(1, 3).is_ok());
}

#[test]
fn two_sew_bad_orientation() {
    let mut map: CMap2<f64> = CMap2::new(4);
    map.force_link::<1>(1, 2).unwrap();
    map.force_link::<1>(3, 4).unwrap();
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (0.0, 1.0)); // 1->2 goes up
    map.force_write_vertex(3, (1.0, 0.0));
    map.force_write_vertex(4, (1.0, 1.0)); // 3->4 also goes up
    assert!(map
        .force_sew::<2>(1, 3)
        .is_err_and(|e| e == SewError::BadGeometry(2, 1, 3)));
}

#[test]
fn one_sew_complete() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.force_link::<2>(1, 2).unwrap();
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (0.0, 1.0));
    map.force_write_vertex(3, (0.0, 2.0));
    map.force_sew::<1>(1, 3).unwrap();
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((0.0, 1.5)));
}

#[test]
fn one_sew_incomplete_attributes() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.force_link::<2>(1, 2).unwrap();
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (0.0, 1.0));
    map.force_sew::<1>(1, 3).unwrap();
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((0.0, 1.0)));
}

#[test]
fn one_sew_incomplete_beta() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (0.0, 1.0));
    map.force_sew::<1>(1, 2).unwrap();
    assert_eq!(map.force_read_vertex(2).unwrap(), Vertex2::from((0.0, 1.0)));
}
#[test]
fn one_sew_no_attributes() {
    let mut map: CMap2<f64> = CMap2::new(3);
    map.force_link::<2>(1, 3).unwrap();
    let res = atomically_with_err(|trans| map.sew::<1>(trans, 1, 2));
    assert!(res.is_err_and(|e| e
        == SewError::FailedAttributeOp(AttributeError::InsufficientData(
            "merge",
            std::any::type_name::<Vertex2<f64>>()
        ))));
    assert!(map.force_sew::<1>(1, 2).is_ok());
}

// --- IO

#[test]
fn io_write() {
    // build a map looking like this:
    //      15
    //     / \
    //    /   \
    //   /     \
    //  16      14
    //  |       |
    //  4---3---7
    //  |   |  /|
    //  |   | / |
    //  |   |/  |
    //  1---2---6
    let mut cmap: CMap2<f32> = CMap2::new(16);
    // bottom left square
    cmap.force_link::<1>(1, 2).unwrap();
    cmap.force_link::<1>(2, 3).unwrap();
    cmap.force_link::<1>(3, 4).unwrap();
    cmap.force_link::<1>(4, 1).unwrap();
    // bottom right triangles
    cmap.force_link::<1>(5, 6).unwrap();
    cmap.force_link::<1>(6, 7).unwrap();
    cmap.force_link::<1>(7, 5).unwrap();
    cmap.force_link::<2>(7, 8).unwrap();
    cmap.force_link::<1>(8, 9).unwrap();
    cmap.force_link::<1>(9, 10).unwrap();
    cmap.force_link::<1>(10, 8).unwrap();
    // top polygon
    cmap.force_link::<1>(11, 12).unwrap();
    cmap.force_link::<1>(12, 13).unwrap();
    cmap.force_link::<1>(13, 14).unwrap();
    cmap.force_link::<1>(14, 15).unwrap();
    cmap.force_link::<1>(15, 16).unwrap();
    cmap.force_link::<1>(16, 11).unwrap();
    // assemble
    cmap.force_link::<2>(2, 10).unwrap();
    cmap.force_link::<2>(3, 11).unwrap();
    cmap.force_link::<2>(9, 12).unwrap();

    // insert vertices
    cmap.force_write_vertex(1, (0.0, 0.0));
    cmap.force_write_vertex(2, (1.0, 0.0));
    cmap.force_write_vertex(6, (2.0, 0.0));
    cmap.force_write_vertex(4, (0.0, 1.0));
    cmap.force_write_vertex(3, (1.0, 1.0));
    cmap.force_write_vertex(7, (2.0, 1.0));
    cmap.force_write_vertex(16, (0.0, 2.0));
    cmap.force_write_vertex(15, (1.0, 3.0));
    cmap.force_write_vertex(14, (2.0, 2.0));

    // generate VTK data
    let mut res = String::new();
    cmap.to_vtk_ascii(&mut res);
    println!("{res}");

    // check result
    assert!(res.contains("POINTS 9 float"));
    assert!(res.contains("CELLS 12 44"));
    assert!(res.contains("CELL_TYPES 12"));
    // faces
    assert!(res.contains("4 0 1 2 3"));
    assert!(res.contains("3 1 4 5"));
    assert!(res.contains("4 0 1 2 3"));
    assert!(res.contains("4 0 1 2 3"));
    // edges
    assert!(res.contains("2 0 1"));
    assert!(res.contains("2 3 0"));
    assert!(res.contains("2 1 4"));
    assert!(res.contains("2 4 5"));
    assert!(res.contains("2 5 6"));
    assert!(res.contains("2 6 7"));
    assert!(res.contains("2 7 8"));
    assert!(res.contains("2 8 3"));
}

// --- PARALLEL

#[derive(Debug, Clone, Copy, Default)]
struct Weight(pub u32);

impl AttributeUpdate for Weight {
    fn merge(attr1: Self, attr2: Self) -> Result<Self, AttributeError> {
        Ok(Self(attr1.0 + attr2.0))
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        // adding the % to keep things conservative
        Ok((Weight(attr.0 / 2 + attr.0 % 2), Weight(attr.0 / 2)))
    }

    fn merge_incomplete(attr: Self) -> Result<Self, AttributeError> {
        Ok(attr)
    }
}

impl AttributeBind for Weight {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = VertexIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}

#[test]
fn sew_ordering() {
    loom::model(|| {
        // setup the map
        let map: CMap2<f64> = CMapBuilder::default().n_darts(5).build().unwrap();
        map.force_link::<2>(1, 2).unwrap();
        map.force_link::<1>(4, 5).unwrap();
        map.force_write_vertex(2, Vertex2(1.0, 1.0));
        map.force_write_vertex(3, Vertex2(1.0, 2.0));
        map.force_write_vertex(5, Vertex2(2.0, 2.0));
        let arc = loom::sync::Arc::new(map);
        let (m1, m2) = (arc.clone(), arc.clone());

        // we're going to do to sew ops:
        // - 1-sew 1 to 3 (t1)
        // - 2-sew 3 to 4 (t2)
        // this will result in a single vertex being define, of ID 2
        // depending on the order of execution of the sews, the value may change
        // 1-sew before 2-sew: (1.5, 1.75)
        // 2-sew before 1-sew: (1.25, 1.5)

        // retry ops until they can be validated
        let t1 = loom::thread::spawn(move || while let Err(_) = m1.force_sew::<1>(1, 3) {});
        let t2 = loom::thread::spawn(move || while let Err(_) = m2.force_sew::<2>(3, 4) {});

        t1.join().unwrap();
        t2.join().unwrap();

        // all path should result in the same topological result here
        let v2 = arc.force_remove_vertex(2);
        let v3 = arc.force_remove_vertex(3);
        let v5 = arc.force_remove_vertex(5);
        assert!(v2.is_some());
        assert!(v3.is_none());
        assert!(v5.is_none());
        assert_eq!(Orbit2::new(arc.as_ref(), OrbitPolicy::Vertex, 2).count(), 3);
        assert_eq!(arc.force_read_vertex(2), None);
        assert_eq!(arc.force_read_vertex(3), None);
        assert_eq!(arc.force_read_vertex(5), None);

        // v2 can have two values though
        let path1 = v2 == Some(Vertex2(1.5, 1.75));
        let path2 = v2 == Some(Vertex2(1.25, 1.5));
        assert!(path1 || path2);
    });
}

#[test]
fn sew_ordering_with_transactions() {
    loom::model(|| {
        // setup the map
        let map: CMap2<f64> = CMapBuilder::default().n_darts(5).build().unwrap();
        let res = atomically_with_err(|trans| {
            map.link::<2>(trans, 1, 2)?;
            map.link::<1>(trans, 4, 5)?;
            map.write_vertex(trans, 2, Vertex2(1.0, 1.0))?;
            map.write_vertex(trans, 3, Vertex2(1.0, 2.0))?;
            map.write_vertex(trans, 5, Vertex2(2.0, 2.0))?;
            Ok(())
        });
        assert!(res.is_ok());

        let arc = loom::sync::Arc::new(map);
        let (m1, m2) = (arc.clone(), arc.clone());

        // we're going to do to sew ops:
        // - 1-sew 1 to 3 (t1)
        // - 2-sew 3 to 4 (t2)
        // this will result in a single vertex being define, of ID 2
        // depending on the order of execution of the sews, the value may change
        // 1-sew before 2-sew: (1.5, 1.75)
        // 2-sew before 1-sew: (1.25, 1.5)

        let t1 = loom::thread::spawn(move || {
            atomically(|trans| {
                if let Err(e) = m1.sew::<1>(trans, 1, 3) {
                    match e {
                        TransactionError::Stm(e) => Err(e),
                        TransactionError::Abort(_) => Err(StmError::Retry),
                    }
                } else {
                    Ok(())
                }
            });
        });

        let t2 = loom::thread::spawn(move || {
            atomically(|trans| {
                if let Err(e) = m2.sew::<2>(trans, 3, 4) {
                    match e {
                        TransactionError::Stm(e) => Err(e),
                        TransactionError::Abort(_) => Err(StmError::Retry),
                    }
                } else {
                    Ok(())
                }
            });
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // all path should result in the same topological result here
        let (v2, v3, v5) = atomically(|trans| {
            Ok((
                arc.remove_vertex(trans, 2)?,
                arc.remove_vertex(trans, 3)?,
                arc.remove_vertex(trans, 5)?,
            ))
        });
        assert!(v2.is_some());
        assert!(v3.is_none());
        assert!(v5.is_none());
        assert_eq!(Orbit2::new(arc.as_ref(), OrbitPolicy::Vertex, 2).count(), 3);
        atomically(|trans| {
            assert_eq!(arc.read_vertex(trans, 2)?, None);
            assert_eq!(arc.read_vertex(trans, 3)?, None);
            assert_eq!(arc.read_vertex(trans, 5)?, None);
            Ok(())
        });

        // v2 can have two values though
        let path1 = v2 == Some(Vertex2(1.5, 1.75));
        let path2 = v2 == Some(Vertex2(1.25, 1.5));
        assert!(path1 || path2);
    });
}

#[test]
fn unsew_ordering() {
    loom::model(|| {
        // setup the map
        let map: CMap2<f64> = CMapBuilder::default()
            .n_darts(5)
            .add_attribute::<Weight>()
            .build()
            .unwrap();
        map.force_link::<2>(1, 2).unwrap();
        map.force_link::<2>(3, 4).unwrap();
        map.force_link::<1>(1, 3).unwrap();
        map.force_link::<1>(4, 5).unwrap();
        map.force_write_vertex(2, Vertex2(0.0, 0.0));
        map.force_write_attribute(2, Weight(33));
        let arc = loom::sync::Arc::new(map);
        let (m1, m2) = (arc.clone(), arc.clone());

        // we're going to do to sew ops:
        // - 1-unsew 1 and 3 (t1)
        // - 2-unsew 3 and 4 (t2)
        // this will result in 3 different weights, defined on IDs 2, 3 and 5
        // depending on the order of execution, the final weights will take the following values:
        // 1-unsew before 2-unsew: (W2, W3, W5) = (17, 8, 8)
        // 2-unsew before 1-unsew: (W2, W3, W5) = (9, 8, 16)

        // retry ops until they can be validated
        let t1 = loom::thread::spawn(move || while let Err(_) = m1.force_unsew::<1>(1) {});
        let t2 = loom::thread::spawn(move || while let Err(_) = m2.force_unsew::<2>(3) {});

        t1.join().unwrap();
        t2.join().unwrap();

        // all path should result in the same topological result here
        let w2 = arc.force_remove_attribute::<Weight>(2);
        let w3 = arc.force_remove_attribute::<Weight>(3);
        let w5 = arc.force_remove_attribute::<Weight>(5);
        assert!(w2.is_some());
        assert!(w3.is_some());
        assert!(w5.is_some());
        let w2 = w2.unwrap();
        let w3 = w3.unwrap();
        let w5 = w5.unwrap();
        assert!(arc.force_read_attribute::<Weight>(2).is_none());
        assert!(arc.force_read_attribute::<Weight>(3).is_none());
        assert!(arc.force_read_attribute::<Weight>(5).is_none());

        // check scenarios
        let path1 = w2.0 == 17 && w3.0 == 8 && w5.0 == 8;
        let path2 = w2.0 == 9 && w3.0 == 8 && w5.0 == 16;
        assert!(path1 || path2);
    });
}

#[test]
fn unsew_ordering_with_transactions() {
    loom::model(|| {
        // setup the map
        let map: CMap2<f64> = CMapBuilder::default()
            .n_darts(5)
            .add_attribute::<Weight>()
            .build()
            .unwrap();
        let res = atomically_with_err(|trans| {
            map.link::<2>(trans, 1, 2)?;
            map.link::<2>(trans, 3, 4)?;
            map.link::<1>(trans, 1, 3)?;
            map.link::<1>(trans, 4, 5)?;
            map.write_vertex(trans, 2, Vertex2(0.0, 0.0))?;
            map.write_attribute(trans, 2, Weight(33))?;
            Ok(())
        });
        assert!(res.is_ok());
        let arc = loom::sync::Arc::new(map);
        let (m1, m2) = (arc.clone(), arc.clone());

        // we're going to do to sew ops:
        // - 1-unsew 1 and 3 (t1)
        // - 2-unsew 3 and 4 (t2)
        // this will result in 3 different weights, defined on IDs 2, 3 and 5
        // depending on the order of execution, the final weights will take the following values:
        // 1-unsew before 2-unsew: (W2, W3, W5) = (17, 8, 8)
        // 2-unsew before 1-unsew: (W2, W3, W5) = (9, 8, 16)

        let t1 = loom::thread::spawn(move || {
            atomically(|trans| {
                if let Err(e) = m1.unsew::<1>(trans, 1) {
                    match e {
                        TransactionError::Stm(e) => Err(e),
                        TransactionError::Abort(_) => Err(StmError::Retry),
                    }
                } else {
                    Ok(())
                }
            });
        });

        let t2 = loom::thread::spawn(move || {
            atomically(|trans| {
                if let Err(e) = m2.unsew::<2>(trans, 3) {
                    match e {
                        TransactionError::Stm(e) => Err(e),
                        TransactionError::Abort(_) => Err(StmError::Retry),
                    }
                } else {
                    Ok(())
                }
            });
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // all path should result in the same topological result here
        let (w2, w3, w5) = atomically(|trans| {
            Ok((
                arc.remove_attribute::<Weight>(trans, 2)?,
                arc.remove_attribute::<Weight>(trans, 3)?,
                arc.remove_attribute::<Weight>(trans, 5)?,
            ))
        });
        assert!(w2.is_some());
        assert!(w3.is_some());
        assert!(w5.is_some());
        let w2 = w2.unwrap();
        let w3 = w3.unwrap();
        let w5 = w5.unwrap();
        atomically(|trans| {
            assert!(arc.read_attribute::<Weight>(trans, 2)?.is_none());
            assert!(arc.read_attribute::<Weight>(trans, 3)?.is_none());
            assert!(arc.read_attribute::<Weight>(trans, 5)?.is_none());
            Ok(())
        });

        // check scenarios
        let path1 = w2.0 == 17 && w3.0 == 8 && w5.0 == 8;
        let path2 = w2.0 == 9 && w3.0 == 8 && w5.0 == 16;
        assert!(path1 || path2);
    });
}
