//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{AttributeBind, AttributeStorage, OrbitPolicy, UnknownAttributeStorage};
use std::any::{Any, TypeId};
use std::collections::HashMap;

// ------ CONTENT

/// Attribute manager error enum.
pub enum ManagerError {
    /// Storage of a given type already exists in the structure.
    DuplicateStorage,
}

/// Main attribute storage structure.
///
/// This structure is used to store all generic attributes that the user may add to the
/// combinatorial map he's building.
///
/// # Implementation
///
/// The structure uses hashmaps in order to store each attribute's dedicated storage. Which storage
/// is used is determined by the associated type [`AttributeBind::StorageType`].
///
/// The key type used by the map is each attribute's [`TypeId`]. This implies that all attributes
/// must have a different (unique) type, i.e. two decimal-valued attribute will need to be wrapped
/// in respective dedicated structures.
///
/// Using the [`TypeId`] as the key value for collections yields a cleaner API, where the only
/// argument passed to access methods is the ID of the cell of which they want the attribute. The
/// actual attribute type is specified by passing a generic to the method. This bypasses any issues
/// linked to literal-typed keys, such as typos, naming conventions, portability, etc.
///
/// Generics passed in access methods also have a secondary usage. To store heterogeneous
/// collections, the internal hashmaps uses `Box<dyn Any>` as their value type. This requires us
/// to cast back the stored object (implementing `Any`) to the correct collection type. This is
/// achieved by using the associated storage type [`AttributeBind::StorageType`]. The code would
/// look like this:
///
/// ```
/// # use std::any::{Any, TypeId};
/// # use std::collections::HashMap;
/// # use honeycomb_core::{AttributeBind, AttributeStorage, UnknownAttributeStorage};
/// pub struct Manager {
///     inner: HashMap<TypeId, Box<dyn Any>>,
/// }
///
/// impl Manager {
///     pub fn add_storage<A: AttributeBind + 'static>(
///         &mut self,
///         size: usize,
///     ) {
///         let typeid = TypeId::of::<A>();
///         let new_storage = <A as AttributeBind>::StorageType::new(size);
///         self.inner.insert(typeid, Box::new(new_storage));
///     }
///
///     pub fn get_storage<A: AttributeBind>(&self) -> &<A as AttributeBind>::StorageType {
///         let probably_storage = &self.inner[&TypeId::of::<A>()];
///         probably_storage
///             .downcast_ref::<<A as AttributeBind>::StorageType>()
///             .expect("E: could not downcast generic storage to specified attribute type")
///     }
/// }
/// ```
#[derive(Default)]
pub struct AttrStorageManager {
    /// Vertex attributes' storages.
    vertices: HashMap<TypeId, Box<dyn Any>>,
    /// Edge attributes' storages.
    edges: HashMap<TypeId, Box<dyn Any>>,
    /// Face attributes' storages.
    faces: HashMap<TypeId, Box<dyn Any>>,
    /// Other storages.
    others: HashMap<TypeId, Box<dyn Any>>, // Orbit::Custom
}

macro_rules! get_storage {
    ($slf: ident, $id: ident) => {
        let probably_storage = match A::binds_to() {
            OrbitPolicy::Vertex => $slf.vertices.get(&TypeId::of::<A>()),
            OrbitPolicy::Edge => $slf.edges.get(&TypeId::of::<A>()),
            OrbitPolicy::Face => $slf.faces.get(&TypeId::of::<A>()),
            OrbitPolicy::Custom(_) => $slf.others.get(&TypeId::of::<A>()),
        };
        let $id = probably_storage
            .expect("E: could not find storage associated to the specified attribute type")
            .downcast_ref::<<A as AttributeBind>::StorageType>()
            .expect("E: could not downcast generic storage to specified attribute type");
    };
}

macro_rules! get_storage_mut {
    ($slf: ident, $id: ident) => {
        let probably_storage = match A::binds_to() {
            OrbitPolicy::Vertex => $slf.vertices.get_mut(&TypeId::of::<A>()),
            OrbitPolicy::Edge => $slf.edges.get_mut(&TypeId::of::<A>()),
            OrbitPolicy::Face => $slf.faces.get_mut(&TypeId::of::<A>()),
            OrbitPolicy::Custom(_) => $slf.others.get_mut(&TypeId::of::<A>()),
        };
        let $id = probably_storage
            .expect("E: could not find storage associated to the specified attribute type")
            .downcast_mut::<<A as AttributeBind>::StorageType>()
            .expect("E: could not downcast generic storage to specified attribute type");
    };
}

impl AttrStorageManager {
    #[allow(clippy::missing_errors_doc)]
    /// Add a new storage to the manager.
    ///
    /// For a breakdown of the principles used for implementation, refer to the *Explanation*
    /// section of the [`AttrStorageManager`] documentation entry.
    ///
    /// # Arguments
    ///
    /// - `size: usize` -- Initial size of the new storage.
    ///
    /// ## Generic
    ///
    /// - `A: AttributeBind + 'static` -- Type of the attribute that will be stored.
    ///
    /// # Return / Error
    ///
    /// The function may return:
    /// - `Ok(())` if the storage was successfully added,
    /// - `Err(ManagerError::DuplicateStorage)` if there was already a storage for the specified
    ///   attribute.
    pub fn add_storage<A: AttributeBind + 'static>(
        &mut self,
        size: usize,
    ) -> Result<(), ManagerError> {
        let typeid = TypeId::of::<A>();
        let new_storage = <A as AttributeBind>::StorageType::new(size);
        if match A::binds_to() {
            OrbitPolicy::Vertex => self.vertices.insert(typeid, Box::new(new_storage)),
            OrbitPolicy::Edge => self.edges.insert(typeid, Box::new(new_storage)),
            OrbitPolicy::Face => self.faces.insert(typeid, Box::new(new_storage)),
            OrbitPolicy::Custom(_) => self.others.insert(typeid, Box::new(new_storage)),
        }
        .is_some()
        {
            Err(ManagerError::DuplicateStorage)
        } else {
            Ok(())
        }
    }

    /// UNIMPLEMENTED
    pub fn extend_storages(&mut self, _length: usize) {
        // not sure if this is actually possible since we need to fetch the attribute from storages,
        // which cannot be interpreted as such without the attribute in the first place
        for _storage in self.vertices.values_mut() {
            todo!()
        }
    }

    /// Extend the size of the storage of a given attribute.
    ///
    /// # Arguments
    ///
    /// - `length: usize` -- Length by which the storage should be extended.
    ///
    /// ## Generic
    ///
    /// - `A: AttributeBind` -- Attribute of which the storage should be extended.
    pub fn extend_storage<A: AttributeBind>(&mut self, length: usize) {
        get_storage_mut!(self, storage);
        storage.extend(length);
    }

    /// Get a reference to the storage of a given attribute.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind` -- Attribute stored by the fetched storage.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - there's no storage associated with the specified attribute
    /// - downcasting `Box<dyn Any>` to `<A as AttributeBind>::StorageType` fails
    #[must_use = "unused getter result - please remove this method call"]
    pub fn get_storage<A: AttributeBind>(&self) -> &<A as AttributeBind>::StorageType {
        let probably_storage = match A::binds_to() {
            OrbitPolicy::Vertex => &self.vertices[&TypeId::of::<A>()],
            OrbitPolicy::Edge => &self.edges[&TypeId::of::<A>()],
            OrbitPolicy::Face => &self.faces[&TypeId::of::<A>()],
            OrbitPolicy::Custom(_) => &self.others[&TypeId::of::<A>()],
        };
        probably_storage
            .downcast_ref::<<A as AttributeBind>::StorageType>()
            .expect("E: could not downcast generic storage to specified attribute type")
    }

    /// Set the value of an attribute.
    ///
    /// # Arguments
    ///
    /// - `id: A::IdentifierType` -- Cell ID to which the attribute is associated.
    /// - `val: A` -- New value of the attribute for the given ID.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind` -- Type of the attribute being set.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - there's no storage associated with the specified attribute
    /// - downcasting `Box<dyn Any>` to `<A as AttributeBind>::StorageType` fails
    /// - the index lands out of bounds
    pub fn set_attribute<A: AttributeBind>(&mut self, id: A::IdentifierType, val: A) {
        get_storage_mut!(self, storage);
        storage.set(id, val);
    }

    /// Set the value of an attribute.
    ///
    /// # Arguments
    ///
    /// - `id: A::IdentifierType` -- Cell ID to which the attribute is associated.
    /// - `val: A` -- New value of the attribute for the given ID.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind` -- Type of the attribute being set.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - **there already is a value associated to the given ID for the specified attribute**
    /// - there's no storage associated with the specified attribute
    /// - downcasting `Box<dyn Any>` to `<A as AttributeBind>::StorageType` fails
    /// - the index lands out of bounds
    pub fn insert_attribute<A: AttributeBind>(&mut self, id: A::IdentifierType, val: A) {
        get_storage_mut!(self, storage);
        storage.insert(id, val);
    }

    /// Get the value of an attribute.
    ///
    /// # Arguments
    ///
    /// - `id: A::IdentifierType` -- Cell ID to which the attribute is associated.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind` -- Type of the attribute fetched.
    ///
    /// # Return
    ///
    /// The method may return:
    /// - `Some(val: A)` if there is an attribute associated with the specified index,
    /// - `None` if there is not.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - there's no storage associated with the specified attribute
    /// - downcasting `Box<dyn Any>` to `<A as AttributeBind>::StorageType` fails
    /// - the index lands out of bounds
    pub fn get_attribute<A: AttributeBind>(&self, id: A::IdentifierType) -> Option<A> {
        get_storage!(self, storage);
        storage.get(id)
    }

    /// Set the value of an attribute.
    ///
    /// # Arguments
    ///
    /// - `id: A::IdentifierType` -- ID of the cell to which the attribute is associated.
    /// - `val: A` -- New value of the attribute for the given ID.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind` -- Type of the attribute being set.
    ///
    /// # Return
    ///
    /// The method should return:
    /// - `Some(val_old: A)` if there was an attribute associated with the specified index,
    /// - `None` if there was not.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - there's no storage associated with the specified attribute
    /// - downcasting `Box<dyn Any>` to `<A as AttributeBind>::StorageType` fails
    /// - the index lands out of bounds
    pub fn replace_attribute<A: AttributeBind>(
        &mut self,
        id: A::IdentifierType,
        val: A,
    ) -> Option<A> {
        get_storage_mut!(self, storage);
        storage.replace(id, val)
    }

    /// Remove the an item from an attribute storage.
    ///
    /// # Arguments
    ///
    /// - `id: A::IdentifierType` -- Cell ID to which the attribute is associated.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind` -- Type of the attribute fetched.
    ///
    /// # Return
    ///
    /// The method may return:
    /// - `Some(val: A)` if was is an attribute associated with the specified index,
    /// - `None` if there was not.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - there's no storage associated with the specified attribute
    /// - downcasting `Box<dyn Any>` to `<A as AttributeBind>::StorageType` fails
    /// - the index lands out of bounds
    pub fn remove_attribute<A: AttributeBind>(&mut self, id: A::IdentifierType) -> Option<A> {
        get_storage_mut!(self, storage);
        storage.remove(id)
    }
}
