//! Main grid descriptor implementation

// ------ IMPORTS

use crate::{BuilderError, CoordsFloat};

// ------ CONTENT

/// Grid description used to generate maps via [`CMapBuilder`].
///
/// The user must specify two out of these three characteristics:
///
/// - `n_cells: [usize; 3]` - The number of cells per axis
/// - `len_per_cell: [T; 3]` - The dimensions of cells per axis
/// - `lens: [T; 3]` -The dimensions of the grid per axis
///
/// # Generics
///
/// - `T: CoordsFloat` -- Generic type of the future map object.
#[derive(Default, Clone)]
pub struct GridDescriptor<T: CoordsFloat> {
    pub(crate) n_cells: Option<[usize; 3]>,
    pub(crate) len_per_cell: Option<[T; 3]>,
    pub(crate) lens: Option<[T; 3]>,
    pub(crate) split_quads: bool,
}

// --- setters

macro_rules! setters {
    ($fld: ident, $fldx: ident, $fldy: ident, $fldz: ident, $zero: expr, $fldty: ty) => {
        /// Set values for all dimensions
        #[must_use = "unused builder object, consider removing this method call"]
        pub fn $fld(mut self, $fld: [$fldty; 3]) -> Self {
            self.$fld = Some($fld);
            self
        }

        /// Set x-axis value
        #[must_use = "unused builder object, consider removing this method call"]
        pub fn $fldx(mut self, $fld: $fldty) -> Self {
            if let Some([ptr, _, _]) = &mut self.$fld {
                *ptr = $fld;
            } else {
                self.$fld = Some([$fld, $zero, $zero]);
            }
            self
        }

        /// Set y-axis value
        #[must_use = "unused builder object, consider removing this method call"]
        pub fn $fldy(mut self, $fld: $fldty) -> Self {
            if let Some([_, ptr, _]) = &mut self.$fld {
                *ptr = $fld;
            } else {
                self.$fld = Some([$zero, $fld, $zero]);
            }
            self
        }

        /// Set z-axis value
        #[must_use = "unused builder object, consider removing this method call"]
        pub fn $fldz(mut self, $fld: $fldty) -> Self {
            if let Some([_, _, ptr]) = &mut self.$fld {
                *ptr = $fld;
            } else {
                self.$fld = Some([$zero, $zero, $fld]);
            }
            self
        }
    };
}

impl<T: CoordsFloat> GridDescriptor<T> {
    // n_cells
    setters!(n_cells, n_cells_x, n_cells_y, n_cells_z, 0, usize);

    // len_per_cell
    setters!(
        len_per_cell,
        len_per_cell_x,
        len_per_cell_y,
        len_per_cell_z,
        T::zero(),
        T
    );

    // lens
    setters!(lens, lens_x, lens_y, lens_z, T::zero(), T);

    /// Indicate whether to split quads of the grid
    #[must_use = "unused builder object, consider removing this method call"]
    pub fn split_quads(mut self, split: bool) -> Self {
        self.split_quads = split;
        self
    }
}

// --- parsing routine

macro_rules! check_parameters {
    ($id: ident, $msg: expr) => {
        if $id.is_sign_negative() | $id.is_zero() {
            return Err(BuilderError::InvalidParameters($msg));
        }
    };
}

impl<T: CoordsFloat> GridDescriptor<T> {
    /// Parse provided grid parameters to provide what's used to actually generate the grid.
    pub(crate) fn parse_2d(self) -> Result<([usize; 2], [T; 2]), BuilderError> {
        match (self.n_cells, self.len_per_cell, self.lens) {
            // from # cells and lengths per cell
            (Some([nx, ny, _]), Some([lpx, lpy, _]), lens) => {
                if lens.is_some() {
                    println!("W: All three grid parameters were specified, total lengths will be ignored");
                }
                #[rustfmt::skip]
                check_parameters!(lpx, "Specified length per x cell is either null or negative");
                #[rustfmt::skip]
                check_parameters!(lpy, "Specified length per y cell is either null or negative");
                Ok(([nx, ny], [lpx, lpy]))
            }
            // from # cells and total lengths
            (Some([nx, ny, _]), None, Some([lx, ly, _])) => {
                #[rustfmt::skip]
                check_parameters!(lx, "Specified grid length along x is either null or negative");
                #[rustfmt::skip]
                check_parameters!(ly, "Specified grid length along y is either null or negative");
                Ok((
                    [nx, ny],
                    [lx / T::from(nx).unwrap(), ly / T::from(ny).unwrap()],
                ))
            }
            // from lengths per cell and total lengths
            (None, Some([lpx, lpy, _]), Some([lx, ly, _])) => {
                #[rustfmt::skip]
                check_parameters!(lpx, "Specified length per x cell is either null or negative");
                #[rustfmt::skip]
                check_parameters!(lpy, "Specified length per y cell is either null or negative");
                #[rustfmt::skip]
                check_parameters!(lx, "Specified grid length along x is either null or negative");
                #[rustfmt::skip]
                check_parameters!(ly, "Specified grid length along y is either null or negative");
                Ok((
                    [
                        (lx / lpx).ceil().to_usize().unwrap(),
                        (ly / lpy).ceil().to_usize().unwrap(),
                    ],
                    [lpx, lpy],
                ))
            }
            (_, _, _) => Err(BuilderError::MissingParameters(
                "GridBuilder: insufficient building parameters",
            )),
        }
    }
}
