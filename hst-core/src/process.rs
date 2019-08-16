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

//! Defines several traits that CSP processes will probably implement.

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::FromIterator;
use std::iter::Sum;
use std::ops::Add;

use maplit::hashset;

use crate::event::Alphabet;
use crate::primitives::tau;
use crate::primitives::Tau;

/// A CSP process is defined by what events it's willing and able to communicate, and when.
pub trait Process<E> {
    /// A process's _cursor_ keeps track of a _current state_, which describes which events the
    /// process is willing and able to communicate _now_.  After communicating one of those
    /// available events, the cursor will move into a different state.
    type Cursor: Cursor<E>;

    /// Returns the _root cursor_ for this process.
    fn root(&self) -> Self::Cursor;
}

/// Tracks the current state of a CSP process, which defines which events it's willing to perform
/// now.
pub trait Cursor<E> {
    /// The type that describes the set of events that the process is willing to perform in its
    /// current state.
    type Alphabet: Alphabet<E>;

    /// Returns the set of events that the process is willing to perform in its current state.
    fn initials(&self) -> Self::Alphabet;

    /// Returns the set of events that the process is willing to perform in its current state.
    ///
    /// (The result represents a _set_ of events, but to make it easier to implement this method,
    /// the result is allowed to contain the same event multiple times.  If you need to have an
    /// actual set, with events appearing once, it's your responsibility to dedup them.)
    fn events<'a>(&'a self) -> Box<dyn Iterator<Item = E> + 'a>;

    /// Returns whether the process is willing to perform a particular event in its current state.
    ///
    /// This is equivalent to the following, but can be more efficient for some process types:
    ///
    /// ``` ignore
    /// self.events().any(|e| *e == event)
    /// ```
    fn can_perform(&self, event: &E) -> bool;

    /// Updates the current state of the cursor to describe what the process would do after
    /// performing a particular event.  Panics if the process is not willing to perform `event` in
    /// its current state.
    fn perform(&mut self, event: &E);

    /// Returns a new cursor representing the process's state after performing a particular event.
    /// Panics if the process is not willing to perform `event` in its current state.
    fn after(&self, event: &E) -> Self
    where
        Self: Clone,
    {
        let mut after = self.clone();
        after.perform(event);
        after
    }
}

/// Returns the initial events of a process.  This includes invisible events like τ.
pub fn initials<C, E>(cursor: &C) -> HashSet<E>
where
    C: Cursor<E>,
    C::Alphabet: IntoIterator<Item = E>,
    E: Eq + From<Tau> + Hash,
{
    cursor.initials().into_iter().collect()
}

/// Returns whether a process satisfies a trace.
pub fn satisfies_trace<C, E, I>(mut cursor: C, trace: I) -> bool
where
    C: Cursor<E>,
    I: IntoIterator<Item = E>,
{
    for event in trace {
        if !cursor.can_perform(&event) {
            return false;
        }
        cursor.perform(&event);
    }
    true
}

/// A set of traces that is maximal — where we ensure that no element of the set is a prefix of any
/// other element.
#[derive(Clone, Eq, PartialEq)]
pub struct MaximalTraces<E: Eq + Hash>(HashSet<Vec<E>>);

impl<E> MaximalTraces<E>
where
    E: Eq + Hash,
{
    pub fn new() -> MaximalTraces<E> {
        MaximalTraces(hashset! {vec![]})
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a Vec<E>> {
        self.0.iter()
    }
}

impl<E> MaximalTraces<E>
where
    E: Clone + Eq + Hash,
{
    pub fn insert(&mut self, trace: Vec<E>) {
        // If the new trace is a prefix of any existing trace, do nothing.
        if self.0.iter().any(|existing| existing.starts_with(&trace)) {
            return;
        }

        // Remove any existing traces that are a prefix of the new one.
        let mut prefix = trace.clone();
        while !prefix.is_empty() {
            prefix.pop();
            self.0.remove(&prefix);
        }

        self.0.insert(trace);
    }

    pub fn map<F>(self, mut f: F) -> MaximalTraces<E>
    where
        F: FnMut(&mut Vec<E>),
    {
        self.into_iter()
            .map(|mut trace| {
                f(&mut trace);
                trace
            })
            .collect()
    }
}

impl<E> Debug for MaximalTraces<E>
where
    E: Debug + Eq + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> Add for MaximalTraces<E>
where
    E: Clone + Eq + Hash,
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        for trace in rhs.0 {
            self.insert(trace);
        }
        self
    }
}

impl<E> FromIterator<Vec<E>> for MaximalTraces<E>
where
    E: Clone + Eq + Hash,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Vec<E>>,
    {
        let mut result = MaximalTraces::new();
        for trace in iter {
            result.insert(trace);
        }
        result
    }
}

impl<E> IntoIterator for MaximalTraces<E>
where
    E: Eq + Hash,
{
    type Item = Vec<E>;
    type IntoIter = std::collections::hash_set::IntoIter<Vec<E>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<E> PartialEq<HashSet<Vec<E>>> for MaximalTraces<E>
where
    E: Clone + Eq + Hash,
{
    fn eq(&self, other: &HashSet<Vec<E>>) -> bool {
        self.0 == *other
    }
}

impl<E> Sum<MaximalTraces<E>> for MaximalTraces<E>
where
    E: Clone + Eq + Hash,
{
    fn sum<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = MaximalTraces<E>>,
    {
        let mut result = MaximalTraces::new();
        for other in iter {
            for trace in other {
                result.insert(trace);
            }
        }
        result
    }
}

/// Returns the maximal finite traces of a process.  Note that traces only contain visible events,
/// and never contain τ!
pub fn maximal_finite_traces<C, E>(cursor: C) -> MaximalTraces<E>
where
    C: Clone + Eq + Cursor<E>,
    C::Alphabet: IntoIterator<Item = E>,
    E: Clone + Eq + From<Tau> + Hash,
{
    fn subprocess<C, E>(
        result: &mut MaximalTraces<E>,
        cursor: C,
        previous_cursors: &mut Vec<C>,
        current_trace: &mut Vec<E>,
    ) where
        C: Clone + Eq + Cursor<E>,
        C::Alphabet: IntoIterator<Item = E>,
        E: Clone + Eq + From<Tau> + Hash,
    {
        // If `cursor` already appears earlier in the current trace, then we've found a cycle.
        if previous_cursors.contains(&cursor) {
            result.insert(current_trace.clone());
            return;
        }

        // If the current subprocess doesn't have any outgoing transitions, we've found the end of
        // a finite trace.
        let initials = cursor.initials().into_iter().collect::<HashSet<_>>();
        if initials.is_empty() {
            result.insert(current_trace.clone());
            return;
        }

        // Otherwise recurse into the subprocesses we get by following each possible event from the
        // current state.
        previous_cursors.push(cursor.clone());
        for initial in initials {
            let mut next_cursor = cursor.clone();
            next_cursor.perform(&initial);
            if initial == tau() {
                subprocess(result, next_cursor, previous_cursors, current_trace);
            } else {
                current_trace.push(initial);
                subprocess(result, next_cursor, previous_cursors, current_trace);
                current_trace.pop();
            }
        }
        previous_cursors.pop();
    }

    let mut result = MaximalTraces::new();
    let mut previous_cursors = Vec::new();
    let mut current_trace = Vec::new();
    subprocess(
        &mut result,
        cursor,
        &mut previous_cursors,
        &mut current_trace,
    );
    result
}

#[cfg(test)]
mod maximal_traces_tests {
    use super::*;

    use proptest_attr_macro::proptest;

    use crate::test_support::TestEvent;

    #[proptest]
    fn maximal_traces_are_maximal(traces: Vec<Vec<TestEvent>>) {
        // Add a bunch of random traces to the set
        let mut maximal_traces = MaximalTraces::new();
        for trace in traces {
            maximal_traces.insert(trace);
        }

        // And make sure that we've removed any traces that are a prefix of any other trace!
        assert!(!maximal_traces
            .iter()
            .any(|a| maximal_traces.iter().any(|b| *a != *b && a.starts_with(b))));
    }
}
