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

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::Add;

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
        MaximalTraces(HashSet::new())
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

/// Returns the maximal finite traces of a process.
pub fn maximal_finite_traces<C, E>(cursor: C) -> MaximalTraces<E>
where
    C: Clone + Eq + Cursor<E>,
    E: Clone + Eq + Hash,
{
    fn subprocess<C, E>(
        result: &mut MaximalTraces<E>,
        cursor: C,
        previous_cursors: &mut Vec<C>,
        current_trace: &mut Vec<E>,
    ) where
        C: Clone + Eq + Cursor<E>,
        E: Clone + Eq + Hash,
    {
        // If `cursor` already appears earlier in the current trace, then we've found a cycle.
        if previous_cursors.contains(&cursor) {
            result.insert(current_trace.clone());
            return;
        }

        // If the current subprocess doesn't have any outgoing transitions, we've found the end of
        // a finite trace.
        let initials = cursor.events().collect::<HashSet<_>>();
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
            current_trace.push(initial);
            subprocess(result, next_cursor, previous_cursors, current_trace);
            current_trace.pop();
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

/// Returns the events that the process is willing to perform.
pub trait Initials<'a, E> {
    type Initials: Iterator<Item = E> + 'a;
    fn initials(&'a self) -> Self::Initials;
}

/// Returns how the process behaves after one of its initial events is performed.  The result is
/// a _set_ of processes; if there are multiple processes in the set, then there is nondeterminism,
/// and the process will behave like one of the elements arbitrarily.
pub trait Afters<'a, E, P> {
    type Afters: Iterator<Item = P> + 'a;
    fn afters(&'a self, initial: &E) -> Self::Afters;
}

/// Returns all of the outgoing transitions for the process.  This is a map where the keys are the
/// initial events of the process, and the values are a collection of the after processes for each
/// of those events.
pub fn transitions<'a, E, P, C>(process: &'a P) -> HashMap<E, C>
where
    E: Eq + Hash,
    P: Initials<'a, E>,
    P: Afters<'a, E, P>,
    C: FromIterator<P>,
{
    let mut transitions = HashMap::new();
    for initial in process.initials() {
        let afters = process.afters(&initial).collect();
        transitions.insert(initial, afters);
    }
    transitions
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::test_support::TestEvent;

    #[proptest]
    /// The `initials` and `afters` methods for a process must be consistent with each other.  If
    /// an event is in the `initials` set, the `afters` iterator must contain at least one element.
    /// If an event is not in the `initials` set, the `afters` iterator must be empty.
    fn initials_consistent_with_afters(process: CSP<TestEvent>, initial: TestEvent) {
        let in_initials = process.initials().any(|e| e == initial);
        let mut afters = process.afters(&initial);
        if in_initials {
            assert!(
                afters.next().is_some(),
                "Afters can't be empty for initial event {}",
                initial
            );
        } else {
            assert!(
                afters.next().is_none(),
                "Afters must be empty for non-initial event {}",
                initial
            );
        }
    }
}
