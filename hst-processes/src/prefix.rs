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

//! Defines the prefix (`→`) operator.

use std::fmt::Debug;
use std::fmt::Display;

use itertools::Either;

use crate::csp::CSP;
use crate::event::EventSet;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Prefix<E, TauProof, TickProof>(E, CSP<E, TauProof, TickProof>);

impl<E, TauProof, TickProof> Debug for Prefix<E, TauProof, TickProof>
where
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Prefix({:?}, {:?})", self.0, self.1)
    }
}

impl<E, TauProof, TickProof> Display for Prefix<E, TauProof, TickProof>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} → {}", self.0, self.1)
    }
}

impl<E, TauProof, TickProof> Prefix<E, TauProof, TickProof> {
    pub(crate) fn new(
        initials: E,
        after: CSP<E, TauProof, TickProof>,
    ) -> Prefix<E, TauProof, TickProof> {
        Prefix(initials, after)
    }
}

// Operational semantics for a → P
//
// 1) ─────────────
//     a → P -a→ P

impl<E, TauProof, TickProof> Prefix<E, TauProof, TickProof>
where
    E: Clone + EventSet,
    TauProof: Clone,
    TickProof: Clone,
{
    pub(crate) fn initials(&self) -> E {
        self.0.clone()
    }

    pub(crate) fn transitions(
        &self,
        events: &E,
    ) -> impl Iterator<Item = (E, CSP<E, TauProof, TickProof>)> {
        let mut events = events.clone();
        events.intersect(&self.0);
        if events.is_empty() {
            return Either::Left(std::iter::empty());
        }

        Either::Right(std::iter::once((events, self.1.clone())))
    }
}

#[cfg(test)]
mod prefix_tests {
    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::maximal_traces::maximal_finite_traces;
    use crate::test_support::NonemptyNumberedEvents;
    use crate::test_support::TestEvents;

    #[proptest]
    fn check_prefix_initials(initials: NonemptyNumberedEvents, after: CSP<TestEvents, _, _>) {
        let initials = TestEvents::from_b(initials.into());
        let process = CSP::prefix(initials.clone(), after.clone());
        assert_eq!(process.initials(), initials);
    }

    #[proptest]
    fn check_prefix_traces(initials: NonemptyNumberedEvents, after: CSP<TestEvents, _, _>) {
        let initials = TestEvents::from_b(initials.into());
        let process = CSP::prefix(initials.clone(), after.clone());
        assert_eq!(
            maximal_finite_traces(&process),
            maximal_finite_traces(&after).map(|trace| trace.insert(0, initials.clone()))
        );
    }
}
