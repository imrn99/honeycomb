use crate::stm::{try_or_coerce, Transaction, TransactionClosureResult};

use crate::{
    attributes::UnknownAttributeStorage,
    cmap::{CMap2, DartIdType, SewError, NULL_DART_ID},
    prelude::CoordsFloat,
};

#[doc(hidden)]
/// **1-(un)sews internals**
impl<T: CoordsFloat> CMap2<T> {
    /// 1-sew transactional implementation.
    pub(super) fn one_sew(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
        rhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let b2lhs_dart_id = self.betas[(2, lhs_dart_id)].read(trans)?;
        if b2lhs_dart_id == NULL_DART_ID {
            try_or_coerce!(
                self.betas.one_link_core(trans, lhs_dart_id, rhs_dart_id),
                SewError
            );
        } else {
            let b2lhs_vid_old = self.vertex_id_transac(trans, b2lhs_dart_id)?;
            let rhs_vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;

            try_or_coerce!(
                self.betas.one_link_core(trans, lhs_dart_id, rhs_dart_id),
                SewError
            );

            let new_vid = self.vertex_id_transac(trans, rhs_dart_id)?;

            try_or_coerce!(
                self.vertices
                    .try_merge(trans, new_vid, b2lhs_vid_old, rhs_vid_old),
                SewError
            );
            try_or_coerce!(
                self.attributes.try_merge_vertex_attributes(
                    trans,
                    new_vid,
                    b2lhs_vid_old,
                    rhs_vid_old,
                ),
                SewError
            );
        }
        Ok(())
    }

    /// 1-unsew transactional implementation.
    pub(super) fn one_unsew(
        &self,
        trans: &mut Transaction,
        lhs_dart_id: DartIdType,
    ) -> TransactionClosureResult<(), SewError> {
        let b2lhs_dart_id = self.betas[(2, lhs_dart_id)].read(trans)?;
        if b2lhs_dart_id == NULL_DART_ID {
            try_or_coerce!(self.betas.one_unlink_core(trans, lhs_dart_id), SewError);
        } else {
            // fetch IDs before topology update
            let rhs_dart_id = self.betas[(1, lhs_dart_id)].read(trans)?;
            let vid_old = self.vertex_id_transac(trans, rhs_dart_id)?;
            // update the topology
            try_or_coerce!(self.betas.one_unlink_core(trans, lhs_dart_id), SewError);
            // split vertices & attributes from the old ID to the new ones
            let (new_lhs, new_rhs) = (
                self.vertex_id_transac(trans, b2lhs_dart_id)?,
                self.vertex_id_transac(trans, rhs_dart_id)?,
            );
            try_or_coerce!(
                self.vertices.try_split(trans, new_lhs, new_rhs, vid_old),
                SewError
            );
            try_or_coerce!(
                self.attributes
                    .try_split_vertex_attributes(trans, new_lhs, new_rhs, vid_old),
                SewError
            );
        }
        Ok(())
    }
}
