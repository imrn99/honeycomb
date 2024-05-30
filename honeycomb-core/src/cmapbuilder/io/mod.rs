//! Input/Output features implementation
//!
//! The support for I/O is currently very restricted since this is not the focus of this project.
//! Maps can be built from and serialized to VTK legacy files (both binary and ASCII). The
//! `DATASET` of the VTK file should be `UNSTRUCTURED_GRID`, and only a given set of `CELL_TYPES`
//! are supported, because of orientation and dimension restriction.

// ------ IMPORTS
use crate::{CMap2, CMapBuilder, CoordsFloat, DartIdentifier, Vertex2, VertexIdentifier};
use num::Zero;
use std::collections::BTreeMap;
use vtkio::model::{CellType, DataSet, VertexNumbers};
use vtkio::{IOBuffer, Vtk};

// ------ CONTENT

// will be deleted soon
impl<T: CoordsFloat + 'static> CMap2<T> {
    /// Build a [`CMap2`] from a `vtk` file.
    ///
    /// # Panics
    ///
    /// This function may panic if:
    /// - the file cannot be loaded
    /// - the internal building routine fails, i.e.
    ///     - the file format is XML
    ///     - the mesh contains one type of cell that is not supported (either because of
    ///     dimension or orientation incompatibilities)
    ///     - the file has major inconsistencies / errors
    #[must_use = "constructed object is not used, consider removing this function call"]
    pub fn from_vtk_file(file_path: impl AsRef<std::path::Path> + std::fmt::Debug) -> Self {
        CMapBuilder::from_vtk_file(file_path).build().unwrap()
    }
}

impl<T: CoordsFloat> CMapBuilder<T> {
    /// Import and set the VTK file that will be used when building the map.
    ///
    /// # Panics
    ///
    /// This function may panic if the file cannot be loaded.
    #[must_use = "unused builder object, consider removing this method call"]
    pub fn vtk_file(mut self, file_path: impl AsRef<std::path::Path> + std::fmt::Debug) -> Self {
        let vtk_file =
            Vtk::import(file_path).unwrap_or_else(|e| panic!("E: failed to load file: {e:?}"));
        self.vtk_file = Some(vtk_file);
        self
    }

    /// Create a [`CMapBuilder`] from an imported VTK file.
    ///
    /// This function is roughly equivalent to the following:
    ///
    /// ```rust,should_panic
    /// # use honeycomb_core::CMapBuilder;
    /// // `CMapBuilder::from_vtk_file("some/path/to/file.vtk")`, or:
    /// let builder = CMapBuilder::<f64>::default().vtk_file("some/path/to/file.vtk");
    /// ```
    ///
    /// # Panics
    ///
    /// This function may panic if the file cannot be loaded.
    #[must_use = "unused builder object, consider removing this function call"]
    pub fn from_vtk_file(file_path: impl AsRef<std::path::Path> + std::fmt::Debug) -> Self {
        let vtk_file =
            Vtk::import(file_path).unwrap_or_else(|e| panic!("E: failed to load file: {e:?}"));
        CMapBuilder {
            vtk_file: Some(vtk_file),
            ..Default::default()
        }
    }
}

macro_rules! build_vertices {
    ($v: ident) => {{
        assert!(
            ($v.len() % 3).is_zero(),
            "failed to build vertices list - the point list contains an incomplete tuple"
        );
        $v.chunks_exact(3)
            .map(|slice| {
                // WE IGNORE Z values
                let &[x, y, _] = slice else { panic!() };
                Vertex2::from((T::from(x).unwrap(), T::from(y).unwrap()))
            })
            .collect()
    }};
}

#[allow(clippy::too_many_lines)]
/// Internal building routine for [`CMap2::from_vtk_file`].
///
/// TODO: change return type to `Result` & propagate return up to the map builder methods.
pub fn build_2d_from_vtk<T: CoordsFloat>(value: Vtk) -> CMap2<T> {
    let mut cmap: CMap2<T> = CMap2::new(0);
    let mut sew_buffer: BTreeMap<(usize, usize), DartIdentifier> = BTreeMap::new();
    match value.data {
        DataSet::ImageData { .. }
        | DataSet::StructuredGrid { .. }
        | DataSet::RectilinearGrid { .. }
        | DataSet::PolyData { .. }
        | DataSet::Field { .. } => {}
        DataSet::UnstructuredGrid { pieces, .. } => pieces.iter().for_each(|piece| {
            // assume inline data
            let tmp = piece
                .load_piece_data(None)
                .expect("failed to load piece data");

            // build vertex list
            // since we're expecting coordinates, we'll assume floating type
            // we're also converting directly to our vertex type since we're building a 2-map
            let vertices: Vec<Vertex2<T>> = match tmp.points {
                IOBuffer::F64(v) => build_vertices!(v),
                IOBuffer::F32(v) => build_vertices!(v),
                _ => unimplemented!(),
            };

            let vtkio::model::Cells { cell_verts, types } = tmp.cells;
            match cell_verts {
                VertexNumbers::Legacy {
                    num_cells,
                    vertices: verts,
                } => {
                    // check basic stuff
                    assert_eq!(
                        num_cells as usize,
                        types.len(),
                        "failed to build cells - inconsistent number of cell between CELLS and CELL_TYPES"
                    );

                    // build a collection of vertex lists corresponding of each cell
                    let mut cell_components: Vec<Vec<usize>> = Vec::new();
                    let mut take_next = 0;
                    verts.iter().for_each(|vertex_id| if take_next.is_zero() {
                        // making it usize since it's a counter
                        take_next = *vertex_id as usize;
                        cell_components.push(Vec::with_capacity(take_next));
                    } else {
                        cell_components.last_mut().unwrap().push(*vertex_id as usize);
                        take_next -= 1;
                    });
                    assert_eq!(num_cells as usize, cell_components.len());

                    types.iter().zip(cell_components.iter()).for_each(|(cell_type, vids)| match cell_type {
                        CellType::Vertex => {
                            assert_eq!(vids.len(), 1, "failed to build cell - `Vertex` has {} instead of 1 vertex", vids.len());
                            // silent ignore
                        }
                        CellType::PolyVertex => unimplemented!(
                            "failed to build cell - `PolyVertex` cell type is not supported because for consistency"
                        ),
                        CellType::Line => {
                            assert_eq!(vids.len(), 2, "failed to build cell - `Line` has {} instead of 2 vertices", vids.len());
                            // silent ignore
                        }
                        CellType::PolyLine => unimplemented!(
                            "failed to build cell - `PolyLine` cell type is not supported because for consistency"
                        ),
                        CellType::Triangle => {
                            // check validity
                            assert_eq!(vids.len(), 3, "failed to build cell - `Triangle` has {} instead of 3 vertices", vids.len());
                            // build the triangle
                            let d0 = cmap.add_free_darts(3);
                            let (d1, d2) = (d0+1, d0+2);
                            cmap.insert_vertex(d0 as VertexIdentifier, vertices[vids[0]]);
                            cmap.insert_vertex(d1 as VertexIdentifier, vertices[vids[1]]);
                            cmap.insert_vertex(d2 as VertexIdentifier, vertices[vids[2]]);
                            cmap.one_link(d0, d1); // edge d0 links vertices vids[0] & vids[1]
                            cmap.one_link(d1, d2); // edge d1 links vertices vids[1] & vids[2]
                            cmap.one_link(d2, d0); // edge d2 links vertices vids[2] & vids[0]
                            // record a trace of the built cell for future 2-sew
                            sew_buffer.insert((vids[0], vids[1]), d0);
                            sew_buffer.insert((vids[1], vids[2]), d1);
                            sew_buffer.insert((vids[2], vids[0]), d2);
                        }
                        CellType::TriangleStrip => unimplemented!(
                            "failed to build cell - `TriangleStrip` cell type is not supported because of orientation requirements"
                        ),
                        CellType::Polygon => {
                            // FIXME: NOT TESTED
                            // operation order should still work, but it would be nice to have
                            // an heterogeneous mesh to test on
                            let n_vertices = vids.len();
                            let d0 = cmap.add_free_darts(n_vertices);
                            (0..n_vertices ).for_each(|i| {
                                let di = d0 + i as DartIdentifier;
                                let dip1 = if i==n_vertices-1 {
                                    d0
                                } else {
                                    di +1
                                };
                                cmap.insert_vertex(di as VertexIdentifier, vertices[vids[i]]);
                                cmap.one_link(di, dip1);
                                sew_buffer.insert((vids[i], vids[(i + 1) % n_vertices]), di);
                            });
                        }
                        CellType::Pixel => unimplemented!(
                            "failed to build cell - `Pixel` cell type is not supported because of orientation requirements"
                        ),
                        CellType::Quad => {
                            assert_eq!(vids.len(), 4,  "failed to build cell - `Quad` has {} instead of 4 vertices", vids.len());
                            // build the quad
                            let d0 = cmap.add_free_darts(4);
                            let (d1, d2, d3) = (d0+1, d0+2, d0+3);
                            cmap.insert_vertex(d0 as VertexIdentifier, vertices[vids[0]]);
                            cmap.insert_vertex(d1 as VertexIdentifier, vertices[vids[1]]);
                            cmap.insert_vertex(d2 as VertexIdentifier, vertices[vids[2]]);
                            cmap.insert_vertex(d3 as VertexIdentifier, vertices[vids[3]]);
                            cmap.one_link(d0, d1); // edge d0 links vertices vids[0] & vids[1]
                            cmap.one_link(d1, d2); // edge d1 links vertices vids[1] & vids[2]
                            cmap.one_link(d2, d3); // edge d2 links vertices vids[2] & vids[3]
                            cmap.one_link(d3, d0); // edge d3 links vertices vids[3] & vids[0]
                            // record a trace of the built cell for future 2-sew
                            sew_buffer.insert((vids[0], vids[1]), d0);
                            sew_buffer.insert((vids[1], vids[2]), d1);
                            sew_buffer.insert((vids[2], vids[3]), d2);
                            sew_buffer.insert((vids[3], vids[0]), d3);
                        }
                        c => unimplemented!(
                            "failed to build cell - {c:#?} is not supported in 2-maps"
                        ),
                    });
                }
                VertexNumbers::XML { .. } => {
                    unimplemented!("XML file format is not currently supported")
                }
            }
        }),
    }
    while let Some(((id0, id1), dart_id0)) = sew_buffer.pop_first() {
        if let Some(dart_id1) = sew_buffer.remove(&(id1, id0)) {
            cmap.two_sew(dart_id0, dart_id1);
        }
    }

    cmap
}

// ------ TESTS
#[cfg(test)]
mod tests;
