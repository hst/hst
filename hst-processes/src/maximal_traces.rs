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

//! Defines several traits that CSP processes will probably implement.

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::FromIterator;
use std::iter::Sum;
use std::ops::Add;

use crate::csp::CSP;
use crate::event::EventSet;
use crate::primitives::Tau;

/// A set of traces that is maximal — where we ensure that no element of the set is a prefix of any
/// other element.
#[derive(Clone, Eq, PartialEq)]
pub struct MaximalTraces<E: Eq + Hash>(HashSet<Vec<E>>);

impl<E> MaximalTraces<E>
where
    E: Eq + Hash,
{
    pub fn new() -> MaximalTraces<E> {
        let mut traces = HashSet::new();
        traces.insert(Vec::new());
        MaximalTraces(traces)
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

#[cfg(test)]
mod maximal_traces_tests {
    use super::*;

    use proptest_attr_macro::proptest;

    use crate::test_support::NumberedEvent;

    #[proptest]
    fn maximal_traces_are_maximal(traces: Vec<Vec<NumberedEvent>>) {
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

/// Returns the maximal finite traces of a process.  Note that traces only contain visible events,
/// and never contain τ!
pub fn maximal_finite_traces<E, TauProof>(process: &CSP<E>) -> MaximalTraces<E>
where
    E: Clone + Eq + EventSet + Tau<TauProof> + Hash,
{
    fn subprocess<E, TauProof>(
        result: &mut MaximalTraces<E>,
        process: &CSP<E>,
        previous_processes: &mut Vec<CSP<E>>,
        current_trace: &mut Vec<E>,
    ) where
        E: Clone + Eq + EventSet + Tau<TauProof> + Hash,
    {
        // If `process` already appears earlier in the current trace, then we've found a cycle.
        if previous_processes.contains(&process) {
            result.insert(current_trace.clone());
            return;
        }

        // If the current subprocess doesn't have any outgoing transitions, we've found the end of
        // a finite trace.
        let initials = process.initials();
        if initials.is_empty() {
            result.insert(current_trace.clone());
            return;
        }

        // Otherwise recurse into the subprocesses we get by following each possible event from the
        // current state.
        previous_processes.push(process.clone());
        for (mut initials, after) in process.transitions(&initials) {
            if initials.can_perform_tau() {
                subprocess(result, &after, previous_processes, current_trace);
                initials.subtract(&E::tau());
            }
            if !initials.is_empty() {
                current_trace.push(initials);
                subprocess(result, &after, previous_processes, current_trace);
                current_trace.pop();
            }
        }
        previous_processes.pop();
    }

    let mut result = MaximalTraces::new();
    let mut previous_processes = Vec::new();
    let mut current_trace = Vec::new();
    subprocess(
        &mut result,
        process,
        &mut previous_processes,
        &mut current_trace,
    );
    result
}
