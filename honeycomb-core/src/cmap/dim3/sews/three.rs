//! 3D sew implementations

use crate::{
    attributes::{AttributeStorage, UnknownAttributeStorage},
    cmap::{
        CMap3, DartIdType, EdgeIdType, Orbit3, OrbitPolicy, SewError, VertexIdType, NULL_DART_ID,
    },
    geometry::CoordsFloat,
    stm::{abort, atomically_with_err, try_or_coerce, Transaction, TransactionClosureResult},
};

/// 3-sews
impl<T: CoordsFloat> CMap3<T> {
    /// 3-sew operation.
    pub(crate) fn three_sew(
        &self,
        trans: &mut Transaction,
        ld: DartIdType,
        rd: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        // using these custom orbits, I can get both dart of all sides, directly ordered
        // for the merges
        let l_side = Orbit3::new(self, OrbitPolicy::Custom(&[1, 0]), ld);
        let r_side = Orbit3::new(self, OrbitPolicy::Custom(&[0, 1]), rd);
        let l_face = l_side.clone().min().expect("E: unreachable");
        let r_face = r_side.clone().min().expect("E: unreachable");
        let mut edges: Vec<(EdgeIdType, EdgeIdType)> = Vec::with_capacity(10);
        let mut vertices: Vec<(VertexIdType, VertexIdType)> = Vec::with_capacity(10);

        // read edge + vertex on the b1ld side. if b0ld == NULL, we need to read the left vertex
        for (l, r) in l_side.zip(r_side) {
            edges.push((
                self.edge_id_transac(trans, l)?,
                self.edge_id_transac(trans, r)?,
            ));
            let b1l = self.beta_transac::<1>(trans, l)?;
            let b2l = self.beta_transac::<2>(trans, l)?;
            // this monster statement is necessary to handle open faces
            vertices.push((
                self.vertex_id_transac(trans, if b1l == NULL_DART_ID { b2l } else { b1l })?,
                self.vertex_id_transac(trans, r)?,
            ));
            // one more for good measures (aka open faces)
            if self.beta_transac::<0>(trans, l)? == NULL_DART_ID {
                let b1r = self.beta_transac::<1>(trans, r)?;
                let b2r = self.beta_transac::<2>(trans, r)?;
                vertices.push((
                    self.vertex_id_transac(trans, l)?,
                    self.vertex_id_transac(trans, if b1r == NULL_DART_ID { b2r } else { b1r })?,
                ));
            }
        }

        // FIXME: we only check orientation of the arg darts
        // ideally, we want to check every sewn pair
        {
            let (l, r) = (ld, rd);
            let (b1l, b2l, b1r, b2r) = (
                self.beta_transac::<1>(trans, l)?,
                self.beta_transac::<2>(trans, l)?,
                self.beta_transac::<1>(trans, r)?,
                self.beta_transac::<2>(trans, r)?,
            );
            let (vid_l, vid_r, vid_b1l, vid_b1r) = (
                self.vertex_id_transac(trans, l)?,
                self.vertex_id_transac(trans, r)?,
                self.vertex_id_transac(trans, if b1l == NULL_DART_ID { b2l } else { b1l })?,
                self.vertex_id_transac(trans, if b1r == NULL_DART_ID { b2r } else { b1r })?,
            );

            if let (
                // (lhs/b1rhs) vertices
                Some(l_vertex),
                Some(b1r_vertex),
                // (b1lhs/rhs) vertices
                Some(b1l_vertex),
                Some(r_vertex),
            ) = (
                // (lhs/b1rhs)
                self.vertices.read(trans, vid_l)?,
                self.vertices.read(trans, vid_b1r)?,
                // (b1lhs/rhs)
                self.vertices.read(trans, vid_b1l)?,
                self.vertices.read(trans, vid_r)?,
            ) {
                let lhs_vector = b1l_vertex - l_vertex;
                let rhs_vector = b1r_vertex - r_vertex;
                // dot product should be negative if the two darts have opposite direction
                // we could also put restriction on the angle made by the two darts to prevent
                // drastic deformation
                if lhs_vector.dot(&rhs_vector) >= T::zero() {
                    abort(SewError::BadGeometry(3, ld, rd))?;
                }
            };
        }

        // (*): these branch corresponds to incomplete merges (at best),
        //      or incorrect structure (at worst). that's not a problem
        //      because `three_link` will detect inconsistencies
        try_or_coerce!(self.three_link(trans, ld, rd), SewError);

        // merge face, edge, vertex attributes
        try_or_coerce!(
            self.attributes
                .try_merge_face_attributes(trans, l_face.min(r_face), l_face, r_face),
            SewError
        );

        for (eid_l, eid_r) in edges.into_iter().filter(|&(eid_l, eid_r)| {
            eid_l != eid_r && eid_l != NULL_DART_ID && eid_r != NULL_DART_ID
        }) {
            try_or_coerce!(
                self.attributes
                    .try_merge_edge_attributes(trans, eid_l.min(eid_r), eid_l, eid_r),
                SewError
            );
        }
        for (vid_l, vid_r) in vertices.into_iter().filter(|&(vid_l, vid_r)| {
            vid_l != vid_r && vid_l != NULL_DART_ID && vid_r != NULL_DART_ID
        }) {
            try_or_coerce!(
                self.vertices
                    .try_merge(trans, vid_l.min(vid_r), vid_l, vid_r),
                SewError
            );
            try_or_coerce!(
                self.attributes
                    .try_merge_vertex_attributes(trans, vid_l.min(vid_r), vid_l, vid_r),
                SewError
            );
        }

        Ok(())
    }

    /// 3-sew operation.
    pub(crate) fn force_three_sew(&self, ld: DartIdType, rd: DartIdType) -> Result<(), SewError> {
        atomically_with_err(|trans| {
            // using these custom orbits, I can get both dart of all sides, directly ordered
            // for the merges
            let l_side = Orbit3::new(self, OrbitPolicy::Custom(&[1, 0]), ld);
            let r_side = Orbit3::new(self, OrbitPolicy::Custom(&[0, 1]), rd);
            let l_face = l_side.clone().min().expect("E: unreachable");
            let r_face = r_side.clone().min().expect("E: unreachable");
            let mut edges: Vec<(EdgeIdType, EdgeIdType)> = Vec::with_capacity(10);
            let mut vertices: Vec<(VertexIdType, VertexIdType)> = Vec::with_capacity(10);

            // read edge + vertex on the b1ld side. if b0ld == NULL, we need to read the left vertex
            for (l, r) in l_side.zip(r_side) {
                edges.push((
                    self.edge_id_transac(trans, l)?,
                    self.edge_id_transac(trans, r)?,
                ));
                let b1l = self.beta_transac::<1>(trans, l)?;
                let b2l = self.beta_transac::<2>(trans, l)?;
                vertices.push((
                    self.vertex_id_transac(trans, if b1l == NULL_DART_ID { b2l } else { b1l })?,
                    self.vertex_id_transac(trans, r)?,
                ));
                // handle open face
                if self.beta_transac::<0>(trans, l)? == NULL_DART_ID {
                    let b1r = self.beta_transac::<1>(trans, r)?;
                    let b2r = self.beta_transac::<2>(trans, r)?;
                    vertices.push((
                        self.vertex_id_transac(trans, l)?,
                        self.vertex_id_transac(trans, if b1r == NULL_DART_ID { b2r } else { b1r })?,
                    ));
                }
            }

            // FIXME: we only check orientation of the arg darts
            // ideally, we want to check every sewn pair
            {
                let (l, r) = (ld, rd);
                let (b1l, b2l, b1r, b2r) = (
                    self.beta_transac::<1>(trans, l)?,
                    self.beta_transac::<2>(trans, l)?,
                    self.beta_transac::<1>(trans, r)?,
                    self.beta_transac::<2>(trans, r)?,
                );
                let (vid_l, vid_r, vid_b1l, vid_b1r) = (
                    self.vertex_id_transac(trans, l)?,
                    self.vertex_id_transac(trans, r)?,
                    self.vertex_id_transac(trans, if b1l == NULL_DART_ID { b2l } else { b1l })?,
                    self.vertex_id_transac(trans, if b1r == NULL_DART_ID { b2r } else { b1r })?,
                );

                if let (
                    // (lhs/b1rhs) vertices
                    Some(l_vertex),
                    Some(b1r_vertex),
                    // (b1lhs/rhs) vertices
                    Some(b1l_vertex),
                    Some(r_vertex),
                ) = (
                    // (lhs/b1rhs)
                    self.vertices.read(trans, vid_l)?,
                    self.vertices.read(trans, vid_b1r)?,
                    // (b1lhs/rhs)
                    self.vertices.read(trans, vid_b1l)?,
                    self.vertices.read(trans, vid_r)?,
                ) {
                    let lhs_vector = b1l_vertex - l_vertex;
                    let rhs_vector = b1r_vertex - r_vertex;
                    // dot product should be negative if the two darts have opposite direction
                    // we could also put restriction on the angle made by the two darts to prevent
                    // drastic deformation
                    if lhs_vector.dot(&rhs_vector) >= T::zero() {
                        abort(SewError::BadGeometry(3, ld, rd))?;
                    }
                };
            }

            // (*): these branch corresponds to incomplete merges (at best),
            //      or incorrect structure (at worst). that's not a problem
            //      because `three_link` will detect inconsistencies
            try_or_coerce!(self.three_link(trans, ld, rd), SewError);

            // merge face, edge, vertex attributes
            self.attributes
                .merge_face_attributes(trans, l_face.min(r_face), l_face, r_face)?;
            for (eid_l, eid_r) in edges.into_iter().filter(|&(eid_l, eid_r)| {
                eid_l != eid_r && eid_l != NULL_DART_ID && eid_r != NULL_DART_ID
            }) {
                self.attributes
                    .merge_edge_attributes(trans, eid_l.min(eid_r), eid_l, eid_r)?;
            }
            for (vid_l, vid_r) in vertices.into_iter().filter(|&(vid_l, vid_r)| {
                vid_l != vid_r && vid_l != NULL_DART_ID && vid_r != NULL_DART_ID
            }) {
                self.vertices.merge(trans, vid_l.min(vid_r), vid_l, vid_r)?;
                self.attributes
                    .merge_vertex_attributes(trans, vid_l.min(vid_r), vid_l, vid_r)?;
            }

            Ok(())
        })
    }
}

/// 3-unsews
impl<T: CoordsFloat> CMap3<T> {
    /// 3-unsew operation.
    pub(crate) fn three_unsew(
        &self,
        trans: &mut Transaction,
        ld: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let rd = self.beta_transac::<3>(trans, ld)?;

        try_or_coerce!(self.unlink::<3>(trans, ld), SewError);

        let l_side = Orbit3::new(self, OrbitPolicy::Custom(&[1, 0]), ld);
        let r_side = Orbit3::new(self, OrbitPolicy::Custom(&[0, 1]), rd);

        // faces
        let l_face = l_side.clone().min().expect("E: unreachable");
        let r_face = r_side.clone().min().expect("E: unreachable");
        try_or_coerce!(
            self.attributes
                .try_split_face_attributes(trans, l_face, r_face, l_face.max(r_face)),
            SewError
        );

        for (l, r) in l_side.zip(r_side) {
            // edge
            let (eid_l, eid_r) = (
                self.edge_id_transac(trans, l)?,
                self.edge_id_transac(trans, r)?,
            );
            try_or_coerce!(
                self.attributes
                    .try_split_edge_attributes(trans, eid_l, eid_r, eid_l.max(eid_r)),
                SewError
            );

            // vertices
            let b1l = self.beta_transac::<1>(trans, l)?;
            let b2l = self.beta_transac::<2>(trans, l)?;
            let (vid_l, vid_r) = (
                self.vertex_id_transac(trans, if b1l == NULL_DART_ID { b2l } else { b1l })?,
                self.vertex_id_transac(trans, r)?,
            );
            try_or_coerce!(
                self.vertices
                    .try_split(trans, vid_l, vid_r, vid_l.max(vid_r)),
                SewError
            );
            try_or_coerce!(
                self.attributes
                    .try_split_vertex_attributes(trans, vid_l, vid_r, vid_l.max(vid_r)),
                SewError
            );
            if self.beta_transac::<0>(trans, l)? == NULL_DART_ID {
                let b1r = self.beta_transac::<1>(trans, r)?;
                let b2r = self.beta_transac::<2>(trans, r)?;
                let (lvid_l, lvid_r) = (
                    self.vertex_id_transac(trans, l)?,
                    self.vertex_id_transac(trans, if b1r == NULL_DART_ID { b2r } else { b1r })?,
                );
                try_or_coerce!(
                    self.vertices
                        .try_split(trans, lvid_l, lvid_r, lvid_l.max(lvid_r)),
                    SewError
                );
                try_or_coerce!(
                    self.attributes.try_split_vertex_attributes(
                        trans,
                        lvid_l,
                        lvid_r,
                        lvid_l.max(lvid_r),
                    ),
                    SewError
                );
            }
        }
        Ok(())
    }

    /// 3-unsew operation.
    pub(crate) fn force_three_unsew(&self, ld: DartIdType) -> Result<(), SewError> {
        atomically_with_err(|trans| {
            let rd = self.beta_transac::<3>(trans, ld)?;

            try_or_coerce!(self.unlink::<3>(trans, ld), SewError);

            let l_side = Orbit3::new(self, OrbitPolicy::Custom(&[1, 0]), ld);
            let r_side = Orbit3::new(self, OrbitPolicy::Custom(&[0, 1]), rd);

            // faces
            let l_face = l_side.clone().min().expect("E: unreachable");
            let r_face = r_side.clone().min().expect("E: unreachable");
            self.attributes
                .split_face_attributes(trans, l_face, r_face, l_face.max(r_face))?;

            for (l, r) in l_side.zip(r_side) {
                // edge
                let (eid_l, eid_r) = (
                    self.edge_id_transac(trans, l)?,
                    self.edge_id_transac(trans, r)?,
                );
                self.attributes
                    .split_edge_attributes(trans, eid_l, eid_r, eid_l.max(eid_r))?;

                // vertices
                let b1l = self.beta_transac::<1>(trans, l)?;
                let b2l = self.beta_transac::<2>(trans, l)?;
                let (vid_l, vid_r) = (
                    self.vertex_id_transac(trans, if b1l == NULL_DART_ID { b2l } else { b1l })?,
                    self.vertex_id_transac(trans, r)?,
                );
                self.vertices.split(trans, vid_l, vid_r, vid_l.max(vid_r))?;
                self.attributes
                    .split_vertex_attributes(trans, vid_l, vid_r, vid_l.max(vid_r))?;
                if self.beta_transac::<0>(trans, l)? == NULL_DART_ID {
                    let b1r = self.beta_transac::<1>(trans, r)?;
                    let b2r = self.beta_transac::<2>(trans, r)?;
                    let (lvid_l, lvid_r) = (
                        self.vertex_id_transac(trans, l)?,
                        self.vertex_id_transac(trans, if b1r == NULL_DART_ID { b2r } else { b1r })?,
                    );
                    self.vertices
                        .split(trans, lvid_l, lvid_r, lvid_l.max(lvid_r))?;
                    self.attributes.split_vertex_attributes(
                        trans,
                        lvid_l,
                        lvid_r,
                        lvid_l.max(lvid_r),
                    )?;
                }
            }
            Ok(())
        })
    }
}
