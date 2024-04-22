//! Attribute storage structures
//!
//! This module contains all code used to describe custom collections used to store attributes
//! (see [`AttributeBind`], [`AttributeUpdate`]).

// ------ IMPORTS

use crate::{AttributeBind, AttributeUpdate};
use num::ToPrimitive;

// ------ CONTENT

/// Custom storage structure for attributes
///
/// This structured is used to store user-defined attributes using a vector of `Option<T>` items.
/// This means that valid attributes value may be separated by an arbitrary number of `None`.
///
/// This implementation should favor access logic over locality of reference.
///
/// # Generics
///
/// - `T: AttributeBind + AttributeUpdate` -- Type of the stored attributes.
///
/// # Example
///
/// **Currently, this type is not meant to be used directly** when operating on combinatorial maps,
/// but it is kept public because it should eventually be part of the map building system where
/// the user will add its own attributes and choose how they are stored. As such, no example
/// is provided.
///
#[cfg_attr(feature = "utils", derive(Clone))]
pub struct AttrSparseVec<T: AttributeBind + AttributeUpdate> {
    /// Inner storage.
    data: Vec<Option<T>>,
}

impl<T: AttributeBind + AttributeUpdate> AttrSparseVec<T> {
    /// Constructor
    ///
    /// # Arguments
    ///
    /// - `n_ids: usize` -- Upper bound of IDs used to index the attribute's values (in practice,
    /// the number of darts).
    ///
    /// # Return
    ///
    /// Return a [`AttrSparseVec`] object full of `None`.
    ///
    pub fn new(n_ids: usize) -> Self {
        Self {
            data: (0..n_ids).map(|_| None).collect(),
        }
    }

    /// Extend the inner vector's length
    ///
    /// # Arguments
    ///
    /// - `length: usize` -- number of `None` instances to append to the current storage.
    ///
    pub fn extend(&mut self, length: usize) {
        self.data.extend((0..length).map(|_| None));
    }

    /// Return the number of stored attributes (i.e. number of `Some(_)` instances)
    pub fn n_attributes(&self) -> usize {
        self.data.iter().filter(|val| val.is_some()).count()
    }

    /// Getter
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    ///
    /// # Return
    ///
    /// Return a reference to the value indexed by `index`.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn get(&self, index: T::IdentifierType) -> &Option<T> {
        &self.data[index.to_usize().unwrap()]
    }

    /// Getter
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    ///
    /// # Return
    ///
    /// Return a mutable reference to the value indexed by `index`.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    #[deprecated]
    pub fn get_mut(&mut self, index: T::IdentifierType) -> &mut Option<T> {
        &mut self.data[index.to_usize().unwrap()]
    }

    /// Setter
    ///
    /// Set the value of an element at a given index.
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    /// - `val: T` -- Attribute value.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn set(&mut self, index: T::IdentifierType, val: T) {
        self.data[index.to_usize().unwrap()] = Some(val);
    }

    /// Setter
    ///
    /// Insert a value at a given index.
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    /// - `val: T` -- Attribute value.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - **there is already a value associated to the specified index**
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn insert(&mut self, index: T::IdentifierType, val: T) {
        let tmp = &mut self.data[index.to_usize().unwrap()];
        assert!(tmp.is_none());
        *tmp = Some(val);
    }

    /// Setter
    ///
    /// Replace the value of an element at a given index.
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    /// - `val: T` -- Attribute value.
    ///
    /// # Return
    ///
    /// Return an option containing the old value if it existed.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn replace(&mut self, index: T::IdentifierType, val: T) -> Option<T> {
        self.data.push(Some(val));
        self.data.swap_remove(index.to_usize().unwrap())
    }

    /// Remove an item from the storage and return it
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    ///
    /// # Return
    ///
    /// Return the item associated to the specified index. Note that the method will not panic if
    /// there was not one, it will simply return `None`.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn remove(&mut self, index: T::IdentifierType) -> Option<T> {
        self.data.push(None);
        self.data.swap_remove(index.to_usize().unwrap())
    }
}

#[cfg(feature = "utils")]
impl<T: AttributeBind + AttributeUpdate + Clone> AttrSparseVec<T> {
    /// Return the amount of space allocated for the storage.
    pub fn allocated_size(&self) -> usize {
        self.data.capacity() * std::mem::size_of::<Option<T>>()
    }

    /// Return the total amount of space used by the storage.
    pub fn effective_size(&self) -> usize {
        self.data.len() * std::mem::size_of::<Option<T>>()
    }

    /// Return the amount of space used by valid entries of the storage.
    pub fn used_size(&self) -> usize {
        self.data.iter().filter(|val| val.is_some()).count() * std::mem::size_of::<Option<T>>()
    }
}

/// Custom storage structure for attributes
///
/// This structured is used to store user-defined attributes using two internal collections:
/// - a vector of `Option<usize>`, effectively acting as a map from identifiers to internal indices
/// - a vector of `T` items, indexed by values of the first vector
///
/// This implementation should favor locality of reference over access logic.
///
/// # Generics
///
/// - `T: AttributeBind + AttributeUpdate + Clone` -- Type of the stored attributes. The
/// `Clone` implementation is required in order to return copied values & invalidate internal
/// storage slot.
///
/// # Example
///
/// **Currently, this type is not meant to be used directly** when operating on combinatorial maps,
/// but it is kept public because it should eventually be part of the map building system where
/// the user will add its own attributes and choose how they are stored. As such, no example
/// is provided.
///
#[cfg_attr(feature = "utils", derive(Clone))]
pub struct AttrCompactVec<T: AttributeBind + AttributeUpdate + Clone> {
    /// Tracker of unused internal slots.
    unused_data_slots: Vec<usize>,
    /// Map between attribute index and internal index.
    index_map: Vec<Option<usize>>,
    /// Inner storage.
    data: Vec<T>,
}

impl<T: AttributeBind + AttributeUpdate + Clone> AttrCompactVec<T> {
    /// Constructor
    ///
    /// # Arguments
    ///
    /// - `n_ids: usize` -- Upper bound of IDs used to index the attribute's values (in practice,
    /// the number of darts).
    ///
    /// # Return
    ///
    /// Return a "value-empty" [`AttrSparseVec`] object.
    ///
    pub fn new(n_ids: usize) -> Self {
        Self {
            unused_data_slots: Vec::new(),
            index_map: vec![None; n_ids],
            data: Vec::new(),
        }
    }

    /// Extend the inner vector's length
    ///
    /// # Arguments
    ///
    /// - `length: usize` -- number of `None` instances to append to the current storage.
    ///
    pub fn extend(&mut self, length: usize) {
        self.index_map.extend((0..length).map(|_| None));
    }

    /// Return the number of stored attributes in the internal storage.
    pub fn n_attributes(&self) -> usize {
        self.data.len()
    }

    /// Return the number of stored, used attributes in the internal storage.
    pub fn n_used_attributes(&self) -> usize {
        self.data.len() - self.unused_data_slots.len()
    }

    /// Getter
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    ///
    /// # Return
    ///
    /// Return an `Option` that may contain a reference to the value associated to `index`, if
    /// it exists.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn get(&self, index: T::IdentifierType) -> Option<&T> {
        self.index_map[index.to_usize().unwrap()].map(|idx| &self.data[idx])
    }

    /// Getter
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    ///
    /// # Return
    ///
    /// Return an `Option` that may contain a mutable reference to the value associated to `index`,
    /// if it exists.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    #[deprecated]
    pub fn get_mut(&mut self, index: T::IdentifierType) -> Option<&mut T> {
        self.index_map[index.to_usize().unwrap()].map(|idx| &mut self.data[idx])
    }

    /// Setter
    ///
    /// Set the value of an element at a given index.
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    /// - `val: T` -- Attribute value.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn set(&mut self, index: T::IdentifierType, val: T) {
        if let Some(idx) = self.index_map[index.to_usize().unwrap()] {
            // internal index is defined => there should be associated data
            self.data[idx] = val;
        } else if let Some(unused_idx) = self.unused_data_slots.pop() {
            // internal index is undefined => a) there is an unused internal slot
            self.data[unused_idx] = val;
            self.index_map[index.to_usize().unwrap()] = Some(unused_idx);
        } else {
            // internal index is undefined => b) there is no unused internal slot
            self.data.push(val);
            self.index_map[index.to_usize().unwrap()] = Some(self.data.len() - 1);
        }
    }

    /// Setter
    ///
    /// Insert a value at a given index.
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    /// - `val: T` -- Attribute value.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - **there is already a value associated to the specified index**
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn insert(&mut self, index: T::IdentifierType, val: T) {
        let idx = &mut self.index_map[index.to_usize().unwrap()];
        assert!(idx.is_none());
        *idx = if let Some(unused_idx) = self.unused_data_slots.pop() {
            self.data[unused_idx] = val;
            Some(unused_idx)
        } else {
            self.data.push(val);
            Some(self.data.len() - 1)
        };
    }

    /// Setter
    ///
    /// Replace the value of an element at a given index.
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    /// - `val: T` -- Attribute value.
    ///
    /// # Return
    ///
    /// Return an option containing the old value if it existed.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn replace(&mut self, index: T::IdentifierType, val: T) -> Option<T> {
        let idx = &self.index_map[index.to_usize().unwrap()];
        assert!(idx.is_some());
        self.data.push(val);
        Some(self.data.swap_remove(idx.unwrap()))
    }

    /// Remove an item from the storage and return it
    ///
    /// # Arguments
    ///
    /// - `index: T::IdentifierType` -- Cell index.
    ///
    /// # Return
    ///
    /// Return the item associated to the specified index. Note that the method will not panic if
    /// there was not one, it will simply return `None`.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn remove(&mut self, index: T::IdentifierType) -> Option<T> {
        self.index_map.push(None);
        if let Some(tmp) = self.index_map.swap_remove(index.to_usize().unwrap()) {
            self.unused_data_slots.push(tmp);
            return Some(self.data[tmp].clone());
        };
        None
    }
}

#[cfg(feature = "utils")]
impl<T: AttributeBind + AttributeUpdate + Clone> AttrCompactVec<T> {
    /// Return the amount of space allocated for the storage.
    pub fn allocated_size(&self) -> usize {
        self.unused_data_slots.capacity() * std::mem::size_of::<usize>()
            + self.index_map.capacity() * std::mem::size_of::<Option<usize>>()
            + self.data.capacity() * std::mem::size_of::<T>()
    }

    /// Return the total amount of space used by the storage.
    pub fn effective_size(&self) -> usize {
        self.unused_data_slots.len() * std::mem::size_of::<usize>()
            + self.index_map.len() * std::mem::size_of::<Option<usize>>()
            + self.data.len() * std::mem::size_of::<T>()
    }

    /// Return the amount of space used by valid entries of the storage.
    pub fn used_size(&self) -> usize {
        self.unused_data_slots.len() * std::mem::size_of::<usize>()
            + self.index_map.iter().filter(|val| val.is_some()).count()
                * std::mem::size_of::<Option<usize>>()
            + self.data.len() * std::mem::size_of::<T>()
    }
}

// ------ TESTS

#[cfg(test)]
mod tests {
    use super::super::Temperature;
    use super::*;

    macro_rules! generate_sparse {
        ($name: ident) => {
            let mut $name = AttrSparseVec::<Temperature>::new(10);
            $name.insert(0, Temperature::from(273.0));
            $name.insert(1, Temperature::from(275.0));
            $name.insert(2, Temperature::from(277.0));
            $name.insert(3, Temperature::from(279.0));
            $name.insert(4, Temperature::from(281.0));
            $name.insert(5, Temperature::from(283.0));
            $name.insert(6, Temperature::from(285.0));
            $name.insert(7, Temperature::from(287.0));
            $name.insert(8, Temperature::from(289.0));
            $name.insert(9, Temperature::from(291.0));
        };
    }

    #[test]
    fn sparse_vec_n_attributes() {
        generate_sparse!(storage);
        assert_eq!(storage.n_attributes(), 10);
        let _ = storage.remove(3);
        assert_eq!(storage.n_attributes(), 9);
        // extend does not affect the number of attributes
        storage.extend(10);
        assert!(storage.get(15).is_none());
        assert_eq!(storage.n_attributes(), 9);
    }

    #[test]
    fn sparse_vec_get_set_get() {
        generate_sparse!(storage);
        assert_eq!(storage.get(3), &Some(Temperature::from(279.0)));
        storage.set(3, Temperature::from(280.0));
        assert_eq!(storage.get(3), &Some(Temperature::from(280.0)));
    }

    #[test]
    fn sparse_vec_get_replace_get() {
        generate_sparse!(storage);
        assert_eq!(storage.get(3), &Some(Temperature::from(279.0)));
        storage.replace(3, Temperature::from(280.0));
        assert_eq!(storage.get(3), &Some(Temperature::from(280.0)));
    }

    #[test]
    #[should_panic(expected = "assertion failed: tmp.is_none()")]
    fn sparse_vec_insert_already_existing() {
        generate_sparse!(storage);
        assert_eq!(storage.get(3), &Some(Temperature::from(279.0)));
        storage.insert(3, Temperature::from(280.0)); // panic
    }

    #[test]
    fn sparse_vec_remove() {
        generate_sparse!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    }

    #[test]
    fn sparse_vec_remove_remove() {
        generate_sparse!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        assert!(storage.remove(3).is_none());
    }

    #[test]
    fn sparse_vec_remove_get() {
        generate_sparse!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        assert!(storage.get(3).is_none());
    }

    #[test]
    fn sparse_vec_remove_set() {
        generate_sparse!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        storage.set(3, Temperature::from(280.0));
        assert!(storage.get(3).is_some());
    }

    #[test]
    fn sparse_vec_remove_insert() {
        generate_sparse!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        storage.insert(3, Temperature::from(280.0));
        assert!(storage.get(3).is_some());
    }

    #[test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn sparse_vec_replace_already_removed() {
        generate_sparse!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        storage.replace(3, Temperature::from(280.0)).unwrap(); // panic
    }

    macro_rules! generate_compact {
        ($name: ident) => {
            let mut $name = AttrCompactVec::<Temperature>::new(10);
            $name.insert(0, Temperature::from(273.0));
            $name.insert(1, Temperature::from(275.0));
            $name.insert(2, Temperature::from(277.0));
            $name.insert(3, Temperature::from(279.0));
            $name.insert(4, Temperature::from(281.0));
            $name.insert(5, Temperature::from(283.0));
            $name.insert(6, Temperature::from(285.0));
            $name.insert(7, Temperature::from(287.0));
            $name.insert(8, Temperature::from(289.0));
            $name.insert(9, Temperature::from(291.0));
        };
    }

    #[test]
    fn compact_vec_n_attributes() {
        generate_compact!(storage);
        assert_eq!(storage.n_attributes(), 10);
        let _ = storage.remove(3);
        assert_eq!(storage.n_attributes(), 10);
        // extend does not affect the number of attributes
        storage.extend(10);
        assert!(storage.get(15).is_none());
        assert_eq!(storage.n_attributes(), 10);
    }

    #[test]
    fn compact_vec_n_used_attributes() {
        generate_compact!(storage);
        assert_eq!(storage.n_used_attributes(), 10);
        let _ = storage.remove(3);
        assert_eq!(storage.n_used_attributes(), 9);
        // extend does not affect the number of attributes
        storage.extend(10);
        assert!(storage.get(15).is_none());
        assert_eq!(storage.n_used_attributes(), 9);
    }

    #[test]
    fn compact_vec_extend_through_set() {
        generate_compact!(storage);
        assert_eq!(storage.n_attributes(), 10);
        // extend does not affect the number of attributes
        storage.extend(10);
        assert_eq!(storage.n_attributes(), 10);
        storage.set(10, Temperature::from(293.0));
        assert_eq!(storage.n_attributes(), 11);
        storage.set(11, Temperature::from(295.0));
        assert_eq!(storage.n_attributes(), 12);
        storage.set(12, Temperature::from(297.0));
        assert_eq!(storage.n_attributes(), 13);
        let _ = storage.remove(3);
        assert_eq!(storage.n_attributes(), 13);
        assert_eq!(storage.n_used_attributes(), 12);
    }

    #[test]
    fn compact_vec_get_set_get() {
        generate_compact!(storage);
        assert_eq!(storage.get(3), Some(&Temperature::from(279.0)));
        storage.set(3, Temperature::from(280.0));
        assert_eq!(storage.get(3), Some(&Temperature::from(280.0)));
    }

    #[test]
    fn compact_vec_get_replace_get() {
        generate_compact!(storage);
        assert_eq!(storage.get(3), Some(&Temperature::from(279.0)));
        storage.replace(3, Temperature::from(280.0));
        assert_eq!(storage.get(3), Some(&Temperature::from(280.0)));
    }

    #[test]
    #[should_panic(expected = "assertion failed: idx.is_none()")]
    fn compact_vec_insert_already_existing() {
        generate_compact!(storage);
        assert_eq!(storage.get(3), Some(&Temperature::from(279.0)));
        storage.insert(3, Temperature::from(280.0)); // panic
    }

    #[test]
    fn compact_vec_remove() {
        generate_compact!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    }

    #[test]
    fn compact_vec_remove_remove() {
        generate_compact!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        assert!(storage.remove(3).is_none());
    }

    #[test]
    fn compact_vec_remove_get() {
        generate_compact!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        assert!(storage.get(3).is_none());
    }

    #[test]
    fn compact_vec_remove_set() {
        generate_compact!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        storage.set(3, Temperature::from(280.0));
        assert!(storage.get(3).is_some());
    }

    #[test]
    fn compact_vec_remove_insert() {
        generate_compact!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        storage.insert(3, Temperature::from(280.0));
        assert!(storage.get(3).is_some());
    }

    #[test]
    #[should_panic(expected = "assertion failed: idx.is_some()")]
    fn compact_vec_replace_already_removed() {
        generate_compact!(storage);
        assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
        storage.replace(3, Temperature::from(280.0)); // panic
    }
}
