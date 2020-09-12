// -*- coding: utf-8 -*-
// ------------------------------------------------------------------------------------------------
// Copyright © 2020, HST authors.
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

use std::fmt::Display;
use std::marker::PhantomData;

/// A set of events.
///
/// For some event types, it's not easy (or efficient) to enumerate all of the possibilities, which
/// rules out using something simple like `HashSet` to store them.  For example, you might instead
/// want to define an alphabet of events using a predicate — a function that takes in an event and
/// evaluates to `true` if the event is in the set.
pub trait EventSet {
    /// Returns an instance of this type that contains no events.
    fn empty() -> Self;

    /// Returns whether this set contains any events.
    fn is_empty(&self) -> bool;

    /// Updates this set to contain any event that's in both `self` and `other`.
    fn intersect(&mut self, other: &Self);

    /// Updates this set to contain exactly the opposite set of events as `self`.
    fn negate(&mut self);

    /// Updates this set to contain any event that's in `self` but not `other`.
    fn subtract(&mut self, other: &Self);

    /// Updates this set to contain any event that's in either `self` or `other`.
    fn union(&mut self, other: &Self);

    /// Returns an instance of this type that contains every possible event.
    fn universe() -> Self;
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DisjointSum<A, B>(pub A, pub B);

impl<A, B> DisjointSum<A, B>
where
    B: EventSet,
{
    pub fn from_a(a: A) -> DisjointSum<A, B> {
        DisjointSum(a, B::empty())
    }
}

impl<A, B> DisjointSum<A, B>
where
    A: EventSet,
{
    pub fn from_b(b: B) -> DisjointSum<A, B> {
        DisjointSum(A::empty(), b)
    }
}

impl<A, B> Display for DisjointSum<A, B>
where
    A: Display + EventSet,
    B: Display + EventSet,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match (self.0.is_empty(), self.1.is_empty()) {
            (false, false) => write!(f, "{} ∪ {}", &self.0, &self.1),
            (false, true) => self.0.fmt(f),
            (true, false) => self.1.fmt(f),
            (true, true) => write!(f, "{{}}"),
        }
    }
}

impl<A, B> EventSet for DisjointSum<A, B>
where
    A: EventSet,
    B: EventSet,
{
    fn empty() -> Self {
        DisjointSum(A::empty(), B::empty())
    }

    fn intersect(&mut self, other: &Self) {
        // Intersection (doubly) distributes through union:
        //   (Sa ∪ Sb) ∩ (Oa ∪ Ob) = (Sa ∩ Oa) ∪ (Sa ∩ Ob) ∪ (Sb ∩ Oa) ∪ (Sb ∩ Ob)
        // but A and be are disjoint:
        //   Sa ∩ Ob = ∅
        //   Sb ∩ Oa = ∅
        // resulting in:
        //   (Sa ∪ Sb) ∩ (Oa ∪ Ob) = (Sa ∩ Oa) ∪ (Sb ∩ Ob)
        self.0.intersect(&other.0);
        self.1.intersect(&other.1);
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty() && self.1.is_empty()
    }

    fn negate(&mut self) {
        // DeMorgan ftw:
        //   ¬(Sa ∪ Sb) = <¬Sa> ∩ <¬Sb>
        // but the negations are actually a bit more complex:
        //   <¬Sa> = ¬Sa ∪ ∞b
        //   <¬Sb> = ∞a ∪ ¬Sb
        // resulting in:
        //   ¬(Sa ∪ Sb) = (¬Sa ∪ ∞b) ∩ (∞a ∪ ¬Sb)
        // and now we doubly distribute:
        //   ¬(Sa ∪ Sb) = (¬Sa ∩ ∞a) ∪ (¬Sa ∩ ¬Sb) ∪ (∞a ∩ ∞b) ∪ (¬Sb ∩ ∞b)
        // but A and be are disjoint:
        //   ¬Sa ∩ ¬Sb = ∅
        //   ∞a ∩ ∞b = ∅
        // resulting in:
        //   ¬(Sa ∪ Sb) = (¬Sa ∩ ∞a) ∪ (¬Sb ∩ ∞b)
        // Intersecting with universe is a no-op:
        //   ¬Sa ∩ ∞a = ¬Sa
        //   ¬Sb ∩ ∞b = ¬Sb
        // resulting in:
        //   ¬(Sa ∪ Sb) = ¬Sa ∪ ¬Sb
        self.0.negate();
        self.1.negate();
    }

    fn subtract(&mut self, other: &Self) {
        self.0.subtract(&other.0);
        self.1.subtract(&other.1);
    }

    fn union(&mut self, other: &Self) {
        // Union is associative and commutative:
        //   (Sa ∪ Sb) ∪ (Oa ∪ Ob) = (Sa ∪ Oa) ∪ (Sb ∪ Ob)
        self.0.union(&other.0);
        self.1.union(&other.1);
    }

    fn universe() -> Self {
        DisjointSum(A::universe(), B::universe())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Here;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct There<T>(PhantomData<T>);

impl<A, B> IntoIterator for DisjointSum<A, B>
where
    A: EventSet + IntoIterator<Item = A>,
    B: EventSet + IntoIterator<Item = B>,
{
    type Item = DisjointSum<A, B>;
    type IntoIter = std::iter::Chain<
        std::iter::Map<A::IntoIter, fn(A) -> DisjointSum<A, B>>,
        std::iter::Map<B::IntoIter, fn(B) -> DisjointSum<A, B>>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        let a = self
            .0
            .into_iter()
            .map(DisjointSum::from_a as fn(A) -> DisjointSum<A, B>);
        let b = self
            .1
            .into_iter()
            .map(DisjointSum::from_b as fn(B) -> DisjointSum<A, B>);
        a.chain(b)
    }
}
