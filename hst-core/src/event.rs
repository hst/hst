// -*- coding: utf-8 -*-
// ------------------------------------------------------------------------------------------------
// Copyright © 2019, HST authors.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License.  You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied.  See the License for the specific language governing permissions and
// limitations under the License.
// ------------------------------------------------------------------------------------------------

//! Defines several traits related to events, and alphabets of events.

use std::collections::HashSet;
use std::hash::Hash;
use std::marker::PhantomData;

/// An alphabet is a set of events.
///
/// For some event types, it's not easy (or efficient) to enumerate all of the possibilities, which
/// rules out using something simple like `HashSet` to store them.  For example, you might instead
/// want to define an alphabet of events using a predicate — a function that takes in an event and
/// evaluates to `true` if the event is in the set.
///
/// Of course, if you _can_ easily enumerate all of the events in your alphabet type, that's great!
/// Just implement `IntoIterator<Item = E>` as well.
pub trait Alphabet<E> {
    /// Returns whether this alphabet contains a particular event.
    fn contains(&self, event: &E) -> bool;
}

/// An alphabet that contains no events.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EmptyAlphabet<E>(PhantomData<E>);

impl<E> EmptyAlphabet<E> {
    pub fn new() -> EmptyAlphabet<E> {
        EmptyAlphabet(PhantomData)
    }
}

impl<E> Alphabet<E> for EmptyAlphabet<E> {
    fn contains(&self, _event: &E) -> bool {
        false
    }
}

impl<E> IntoIterator for EmptyAlphabet<E> {
    type Item = E;
    type IntoIter = std::iter::Empty<E>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::empty()
    }
}

impl<E> Alphabet<E> for HashSet<E>
where
    E: Eq + Hash,
{
    fn contains(&self, event: &E) -> bool {
        HashSet::contains(self, event)
    }
}
