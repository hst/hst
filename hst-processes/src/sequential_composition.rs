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

//! Defines the sequential composition (`;`) operator.

use std::fmt::Debug;
use std::fmt::Display;

use itertools::Either;

use crate::csp::CSP;
use crate::event::EventSet;
use crate::primitives::Tau;
use crate::primitives::Tick;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct SequentialComposition<E, TauProof, TickProof> {
    p: CSP<E, TauProof, TickProof>,
    q: CSP<E, TauProof, TickProof>,
}

impl<E, TauProof, TickProof> Debug for SequentialComposition<E, TauProof, TickProof>
where
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("SequentialComposition")
            .field("p", &self.p)
            .field("q", &self.q)
            .finish()
    }
}

impl<E, TauProof, TickProof> Display for SequentialComposition<E, TauProof, TickProof>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} ; {}", self.p, self.q)
    }
}

impl<E, TauProof, TickProof> SequentialComposition<E, TauProof, TickProof> {
    pub(crate) fn new(
        p: CSP<E, TauProof, TickProof>,
        q: CSP<E, TauProof, TickProof>,
    ) -> SequentialComposition<E, TauProof, TickProof> {
        SequentialComposition { p, q }
    }
}

// Operational semantics for P ; Q
//
//        P -a→ P'
// 1)  ────────────── a ≠ ✔
//      P;Q -a→ P';Q
//
//     ∃ P' • P -✔→ P'
// 2) ─────────────────
//       P;Q -τ→ Q

impl<E, TauProof, TickProof> SequentialComposition<E, TauProof, TickProof>
where
    E: Clone + EventSet + Tau<TauProof> + Tick<TickProof>,
    TauProof: Clone,
    TickProof: Clone,
{
    pub(crate) fn initials(&self) -> E {
        let mut initials = self.p.initials();
        if initials.can_perform_tick() {
            initials.subtract(&E::tick());
            initials.union(&E::tau());
        }
        initials
    }

    pub(crate) fn transitions(
        &self,
        events: &E,
    ) -> impl Iterator<Item = (E, CSP<E, TauProof, TickProof>)> + '_ {
        let mut events = events.clone();

        // The composition can never perform a ✔; that's always translated into a τ that activates
        // process Q.
        events.subtract(&E::tick());

        // If P can perform a non-✔ event (including τ) leading to P', then P;Q can also perform
        // that event, leading to P';Q.
        let other_transitions = self.p.transitions(&events).map(move |(initials, after)| {
            (initials, CSP::sequential_composition(after, self.q.clone()))
        });

        // If P can perform a ✔ leading to P', then P;Q can perform a τ leading to Q.  Note that we
        // don't care what P' is; we just care that it exists.
        let tau_transitions = if events.can_perform_tau() {
            let tau = E::tau();
            if !self.p.transitions(&tau).next().is_some() {
                Either::Left(std::iter::once((tau, self.q.clone())))
            } else {
                Either::Right(std::iter::empty())
            }
        } else {
            Either::Right(std::iter::empty())
        };

        other_transitions.chain(tau_transitions)
    }
}

#[cfg(test)]
mod sequential_composition_tests {
    use super::*;

    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::maximal_traces::maximal_finite_traces;
    use crate::maximal_traces::MaximalTraces;
    use crate::test_support::TestEvents;

    #[proptest]
    fn check_sequential_composition_initials(p: CSP<TestEvents, _, _>, q: CSP<TestEvents, _, _>) {
        let process = CSP::sequential_composition(p.clone(), q.clone());
        let mut expected = p.initials();
        if expected.can_perform_tick() {
            expected.subtract(&TestEvents::tick());
            expected.union(&TestEvents::tau());
        }
        assert_eq!(process.initials(), expected);
    }

    #[proptest]
    fn check_sequential_composition_traces(p: CSP<TestEvents, _, _>, q: CSP<TestEvents, _, _>) {
        let process = CSP::sequential_composition(p.clone(), q.clone());

        // For any trace of P, we need to replace a ✔ at the end with all possible traces of Q.
        let mut expected = MaximalTraces::new();
        for mut trace in maximal_finite_traces(&p) {
            if trace.ends_with(&vec![TestEvents::tick()]) {
                trace.pop();
                expected.insert(trace.clone());
                for suffix in maximal_finite_traces(&q) {
                    let mut combined = trace.clone();
                    combined.extend(suffix);
                    expected.insert(combined);
                }
            } else {
                expected.insert(trace);
            }
        }
        assert_eq!(maximal_finite_traces(&process), expected);
    }
}
