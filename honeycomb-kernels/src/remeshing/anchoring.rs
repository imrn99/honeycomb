//! geometrical anchoring code

use honeycomb_core::{
    attributes::{AttrSparseVec, AttributeBind, AttributeError, AttributeUpdate},
    cmap::{EdgeIdType, FaceIdType, OrbitPolicy, VertexIdType},
};

// --- Vertex anchors

/// Geometrical mesh anchor.
///
/// This enum is used as an attribute to link mesh vertices to entities of the represented geometry.
///
/// The `AttributeUpdate` implementation is used to enforce the dimensional conditions required to
/// merge two anchors. The merge-ability of two anchors also depends on their intersection; we
/// expect this to be handled outside of the merge functor, as doing it inside would require leaking
/// map data into the trait's methods.
#[derive(Debug, Copy, Clone)]
pub enum VertexAnchor {
    /// Vertex is linked to a node.
    Node(usize),
    /// Vertex is linked to a curve.
    Curve(usize),
    /// Vertex is linked to a surface.
    Surface(usize),
    /// Vertex is linked to a 3D body.
    Body(usize),
}

impl AttributeBind for VertexAnchor {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = VertexIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}

impl AttributeUpdate for VertexAnchor {
    fn merge(attr1: Self, attr2: Self) -> Result<Self, AttributeError> {
        match attr1 {
            Self::Node(id1) => match attr2 {
                Self::Node(id2) => {
                    if id1 == id2 {
                        Ok(Self::Node(id1))
                    } else {
                        Err(AttributeError::FailedMerge(
                            std::any::type_name::<Self>(),
                            "anchors have the same dimension but different IDs",
                        ))
                    }
                }
                Self::Curve(_) | Self::Surface(_) | Self::Body(_) => Ok(attr1),
            },
            Self::Curve(id1) => match attr2 {
                Self::Node(_) => Ok(attr2),
                Self::Curve(id2) => {
                    if id1 == id2 {
                        Ok(Self::Curve(id1))
                    } else {
                        Err(AttributeError::FailedMerge(
                            std::any::type_name::<Self>(),
                            "anchors have the same dimension but different IDs",
                        ))
                    }
                }
                Self::Surface(_) | Self::Body(_) => Ok(attr1),
            },
            Self::Surface(id1) => match attr2 {
                Self::Node(_) | Self::Curve(_) => Ok(attr2),
                Self::Surface(id2) => {
                    if id1 == id2 {
                        Ok(Self::Surface(id1))
                    } else {
                        Err(AttributeError::FailedMerge(
                            std::any::type_name::<Self>(),
                            "anchors have the same dimension but different IDs",
                        ))
                    }
                }
                Self::Body(_) => Ok(attr1),
            },
            Self::Body(id1) => match attr2 {
                Self::Node(_) | Self::Curve(_) | Self::Surface(_) => Ok(attr2),
                Self::Body(id2) => {
                    if id1 == id2 {
                        Ok(Self::Body(id1))
                    } else {
                        Err(AttributeError::FailedMerge(
                            std::any::type_name::<Self>(),
                            "anchors have the same dimension but different IDs",
                        ))
                    }
                }
            },
        }
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        Ok((attr, attr))
    }
}

// --- Edge anchors

/// Geometrical mesh anchor.
///
/// This enum is used as an attribute to link mesh edges to entities of the represented geometry..
///
/// The `AttributeUpdate` implementation is used to enforce the dimensional conditions required to
/// merge two anchors. The merge-ability of two anchors also depends on their intersection; we
/// expect this to be handled outside of the merge functor, as doing it inside would require leaking
/// map data into the trait's methods.
#[derive(Debug, Copy, Clone)]
pub enum EdgeAnchor {
    /// Vertex is linked to a curve.
    Curve(usize),
    /// Vertex is linked to a surface.
    Surface(usize),
    /// Vertex is linked to a 3D body.
    Body(usize),
}

impl AttributeBind for EdgeAnchor {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = EdgeIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Edge;
}

impl AttributeUpdate for EdgeAnchor {
    fn merge(attr1: Self, attr2: Self) -> Result<Self, AttributeError> {
        match attr1 {
            Self::Curve(id1) => match attr2 {
                Self::Curve(id2) => {
                    if id1 == id2 {
                        Ok(Self::Curve(id1))
                    } else {
                        Err(AttributeError::FailedMerge(
                            std::any::type_name::<Self>(),
                            "anchors have the same dimension but different IDs",
                        ))
                    }
                }
                Self::Surface(_) | Self::Body(_) => Ok(attr1),
            },
            Self::Surface(id1) => match attr2 {
                Self::Curve(_) => Ok(attr2),
                Self::Surface(id2) => {
                    if id1 == id2 {
                        Ok(Self::Surface(id1))
                    } else {
                        Err(AttributeError::FailedMerge(
                            std::any::type_name::<Self>(),
                            "anchors have the same dimension but different IDs",
                        ))
                    }
                }
                Self::Body(_) => Ok(attr1),
            },
            Self::Body(id1) => match attr2 {
                Self::Curve(_) | Self::Surface(_) => Ok(attr2),
                Self::Body(id2) => {
                    if id1 == id2 {
                        Ok(Self::Body(id1))
                    } else {
                        Err(AttributeError::FailedMerge(
                            std::any::type_name::<Self>(),
                            "anchors have the same dimension but different IDs",
                        ))
                    }
                }
            },
        }
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        Ok((attr, attr))
    }
}

// --- Face anchors

/// Geometrical mesh anchor.
///
/// This enum is used as an attribute to link mesh faces to entities of the represented geometry..
///
/// The `AttributeUpdate` implementation is used to enforce the dimensional conditions required to
/// merge two anchors. The merge-ability of two anchors also depends on their intersection; we
/// expect this to be handled outside of the merge functor, as doing it inside would require leaking
/// map data into the trait's methods.
#[derive(Debug, Copy, Clone)]
pub enum FaceAnchor {
    /// Vertex is linked to a surface.
    Surface(usize),
    /// Vertex is linked to a 3D body.
    Body(usize),
}

impl AttributeBind for FaceAnchor {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = FaceIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Face;
}

impl AttributeUpdate for FaceAnchor {
    fn merge(attr1: Self, attr2: Self) -> Result<Self, AttributeError> {
        match attr1 {
            Self::Surface(id1) => match attr2 {
                Self::Surface(id2) => {
                    if id1 == id2 {
                        Ok(Self::Surface(id1))
                    } else {
                        Err(AttributeError::FailedMerge(
                            std::any::type_name::<Self>(),
                            "anchors have the same dimension but different IDs",
                        ))
                    }
                }
                Self::Body(_) => Ok(attr1),
            },
            Self::Body(id1) => match attr2 {
                Self::Surface(_) => Ok(attr2),
                Self::Body(id2) => {
                    if id1 == id2 {
                        Ok(Self::Body(id1))
                    } else {
                        Err(AttributeError::FailedMerge(
                            std::any::type_name::<Self>(),
                            "anchors have the same dimension but different IDs",
                        ))
                    }
                }
            },
        }
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        Ok((attr, attr))
    }
}
