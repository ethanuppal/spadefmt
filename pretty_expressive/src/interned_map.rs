// Copyright (C) 2024 Ethan Uppal.
//
// This file is part of pretty-expressive.
//
// pretty-expressive is free software: you can redistribute it and/or modify it
// under the terms of the GNU Lesser General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at your
// option) any later version. pretty-expressive is distributed in the hope that
// it will be useful, but WITHOUT ANY WARRANTY; without even the implied
// warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Lesser General Public License for more details. You should have received a
// copy of the GNU Lesser General Public License along with pretty-expressive.
// If not, see <https://www.gnu.org/licenses/>.

use std::{
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
#[derivative(PartialEq(bound = ""))]
#[derivative(Eq(bound = ""))]
#[derivative(Hash(bound = ""))]
#[derivative(Clone(bound = ""))]
#[derivative(Copy(bound = ""))]
pub struct InternedKey<T> {
    index: usize,
    generic: PhantomData<T>,
}

impl<T> InternedKey<T> {
    fn at(index: usize) -> Self {
        Self {
            index,
            generic: PhantomData,
        }
    }
}

pub struct InternedMap<T> {
    store: Vec<T>,
    unique: HashMap<T, usize>,
}

impl<T: Eq + Hash + Clone> InternedMap<T> {
    pub fn add(&mut self, value: T) -> InternedKey<T> {
        let index = *self.unique.entry(value).or_insert_with_key(|value| {
            self.store.push(value.clone());
            self.store.len() - 1
        });
        InternedKey::at(index)
    }
}

pub struct InternedMapIter<'a, T> {
    store: &'a [T],
    next: usize,
}

impl<'a, T> Iterator for InternedMapIter<'a, T> {
    type Item = (InternedKey<T>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.store.len() {
            None
        } else {
            let index = self.next;
            let value = &self.store[index];
            self.next += 1;
            Some((InternedKey::at(index), value))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.next, Some(self.next))
    }
}

// impl<'a, T> DoubleEndedIterator for InternedMapIter<'a, T> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         if self.next == self.store.len() {
//             None
//         } else {
//             let (value, rest) = self.store.split_last().unwrap();
//             self.store = rest;
//             Some((InternedKey::at(self.store.len()), value))
//         }
//     }
// }

impl<'a, T> IntoIterator for &'a InternedMap<T> {
    type Item = (InternedKey<T>, &'a T);

    type IntoIter = InternedMapIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            store: &self.store,
            next: 0,
        }
    }
}

impl<T> Index<InternedKey<T>> for InternedMap<T> {
    type Output = T;

    fn index(&self, key: InternedKey<T>) -> &Self::Output {
        &self.store[key.index]
    }
}

impl<T> IndexMut<InternedKey<T>> for InternedMap<T> {
    fn index_mut(&mut self, key: InternedKey<T>) -> &mut Self::Output {
        &mut self.store[key.index]
    }
}

pub struct InternedInfoMap<T, V> {
    store: Box<[V]>,
    present: Box<[bool]>,
    _generic: PhantomData<T>,
}

impl<T, V> InternedInfoMap<InternedKey<T>, V> {
    pub fn aligned_with(backing_context: &InternedMap<T>) -> Self {
        let length = backing_context.store.len();

        let mut store = Vec::with_capacity(length);
        #[allow(clippy::uninit_vec)]
        unsafe {
            store.set_len(length);
        }

        Self {
            store: store.into_boxed_slice(),
            present: vec![false; length].into_boxed_slice(),
            _generic: PhantomData,
        }
    }

    pub fn init(&mut self, key: InternedKey<T>, value: V)
    where
        V: Clone, {
        self.store[key.index] = value;
        self.present[key.index] = true;
    }

    pub fn contains_key(&self, key: InternedKey<T>) -> bool {
        self.present[key.index]
    }
}

impl<T, V> Index<InternedKey<T>> for InternedInfoMap<InternedKey<T>, V> {
    type Output = V;

    fn index(&self, key: InternedKey<T>) -> &Self::Output {
        assert!(self.present[key.index]);
        &self.store[key.index]
    }
}

// impl<T, V> IndexMut<InternedKey<T>> for InternedInfoMap<InternedKey<T>, V> {
//     fn index_mut(&mut self, key: InternedKey<T>) -> &mut Self::Output {
//         assert!(self.present[key.index]);
//         &mut self.store[key.index]
//     }
// }
