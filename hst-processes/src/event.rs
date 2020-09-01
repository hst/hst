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
