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

//! Defines the internal choice (`⊓`) operator.

use std::fmt::Debug;
use std::fmt::Display;
use std::marker::PhantomData;

use itertools::Either;

use crate::csp::CSP;
use crate::event::EventSet;
use crate::primitives::Tau;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct InternalChoice<E, TauProof, TickProof>(
    Vec<CSP<E, TauProof, TickProof>>,
    PhantomData<TauProof>,
);

impl<E, TauProof, TickProof> Debug for InternalChoice<E, TauProof, TickProof>
where
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut tuple = f.debug_tuple("InternalChoice");
        for child in &self.0 {
            tuple.field(child);
        }
        tuple.finish()
    }
}

impl<E, TauProof, TickProof> Display for InternalChoice<E, TauProof, TickProof>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.0.len() == 2 {
            write!(f, "{} ⊓ {}", self.0[0], self.0[1])
        } else {
            f.write_str("⊓ ")?;
            let mut set = f.debug_set();
            for child in &self.0 {
                set.entry(&format_args!("{}", child));
            }
            set.finish()
        }
    }
}

impl<E, TauProof, TickProof> InternalChoice<E, TauProof, TickProof> {
    pub(crate) fn new(
        ps: Vec<CSP<E, TauProof, TickProof>>,
    ) -> InternalChoice<E, TauProof, TickProof> {
        assert!(
            !ps.is_empty(),
            "Cannot perform internal choice over no processes"
        );
        InternalChoice(ps, PhantomData)
    }
}

// Operational semantics for ⊓ Ps
//
// 1) ──────────── P ∈ Ps
//     ⊓ Ps -τ→ P

impl<E, TauProof, TickProof> InternalChoice<E, TauProof, TickProof>
where
    E: Clone + EventSet + Tau<TauProof>,
    TauProof: Clone,
    TickProof: Clone,
{
    pub(crate) fn initials(&self) -> E {
        E::tau()
    }

    pub(crate) fn transitions(
        &self,
        events: &E,
    ) -> impl Iterator<Item = (E, CSP<E, TauProof, TickProof>)> + '_ {
        if !events.can_perform_tau() {
            return Either::Left(std::iter::empty());
        }

        Either::Right(self.0.iter().map(|child| (E::tau(), child.clone())))
    }
}

#[cfg(test)]
mod internal_choice_tests {
    use super::*;

    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::maximal_traces::maximal_finite_traces;
    use crate::maximal_traces::MaximalTraces;
    use crate::test_support::NonemptyVec;
    use crate::test_support::TestEvents;

    #[proptest]
    fn check_singleton_internal_choice_initials(p: CSP<TestEvents, _, _>) {
        let process = CSP::replicated_internal_choice(vec![p]);
        assert_eq!(process.initials(), TestEvents::tau());
    }

    #[proptest]
    fn check_singleton_internal_choice_traces(p: CSP<TestEvents, _, _>) {
        let process = CSP::replicated_internal_choice(vec![p.clone()]);
        assert_eq!(maximal_finite_traces(&process), maximal_finite_traces(&p));
    }

    #[proptest]
    fn check_doubleton_internal_choice_initials(
        p: CSP<TestEvents, _, _>,
        q: CSP<TestEvents, _, _>,
    ) {
        let process = CSP::internal_choice(p, q);
        assert_eq!(process.initials(), TestEvents::tau());
    }

    #[proptest]
    fn check_doubleton_internal_choice_traces(p: CSP<TestEvents, _, _>, q: CSP<TestEvents, _, _>) {
        let process = CSP::internal_choice(p.clone(), q.clone());
        assert_eq!(
            maximal_finite_traces(&process),
            maximal_finite_traces(&p) + maximal_finite_traces(&q)
        );
    }

    #[proptest]
    fn check_replicated_internal_choice_initials(ps: NonemptyVec<CSP<TestEvents, _, _>>) {
        let process = CSP::replicated_internal_choice(ps.vec);
        assert_eq!(process.initials(), TestEvents::tau());
    }

    #[proptest]
    fn check_replicated_internal_choice_traces(ps: NonemptyVec<CSP<TestEvents, _, _>>) {
        let process = CSP::replicated_internal_choice(ps.vec.clone());
        assert_eq!(
            maximal_finite_traces(&process),
            ps.vec
                .iter()
                .map(maximal_finite_traces)
                .sum::<MaximalTraces<_>>()
        );
    }
}
