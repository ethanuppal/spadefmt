// Copyright (C) 2024 Ethan Uppal.
//
// This file is part of spadefmt.
//
// spadefmt is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version. spadefmt is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details. You should have received a copy of the GNU General Public License
// along with spadefmt. If not, see <https://www.gnu.org/licenses/>.

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
