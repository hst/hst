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

//! Defines the external choice (`□`) operator.

use std::fmt::Debug;
use std::fmt::Display;
use std::marker::PhantomData;

use itertools::Either;

use crate::csp::CSP;
use crate::event::EventSet;
use crate::primitives::Tau;
use crate::primitives::Tick;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ExternalChoice<E, TauProof, TickProof>(
    Vec<CSP<E, TauProof, TickProof>>,
    PhantomData<TauProof>,
);

impl<E, TauProof, TickProof> Debug for ExternalChoice<E, TauProof, TickProof>
where
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut tuple = f.debug_tuple("ExternalChoice");
        for child in &self.0 {
            tuple.field(child);
        }
        tuple.finish()
    }
}

impl<E, TauProof, TickProof> Display for ExternalChoice<E, TauProof, TickProof>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.0.len() == 2 {
            write!(f, "{} □ {}", self.0[0], self.0[1])
        } else {
            f.write_str("□ ")?;
            let mut set = f.debug_set();
            for child in &self.0 {
                set.entry(&format_args!("{}", child));
            }
            set.finish()
        }
    }
}

impl<E, TauProof, TickProof> ExternalChoice<E, TauProof, TickProof> {
    pub(crate) fn new(
        ps: Vec<CSP<E, TauProof, TickProof>>,
    ) -> ExternalChoice<E, TauProof, TickProof> {
        ExternalChoice(ps, PhantomData)
    }
}

// Operational semantics for □ Ps
//
//                  P -τ→ P'
//  1)  ────────────────────────────── P ∈ Ps
//       □ Ps -τ→ □ (Ps ∖ {P} ∪ {P'})
//
//         P -a→ P'
//  2)  ───────────── P ∈ Ps, a ≠ τ
//       □ Ps -a→ P'

impl<E, TauProof, TickProof> ExternalChoice<E, TauProof, TickProof>
where
    E: Clone + EventSet + Tau<TauProof> + Tick<TickProof>,
    TauProof: Clone,
    TickProof: Clone,
{
    pub(crate) fn initials(&self) -> E {
        let mut initials = E::empty();
        for child in &self.0 {
            initials.union(&child.initials());
        }
        initials
    }

    pub(crate) fn transitions(
        &self,
        events: &E,
    ) -> impl Iterator<Item = (E, CSP<E, TauProof, TickProof>)> + '_ {
        let events = events.clone();
        self.0.iter().enumerate().flat_map(move |(index, child)| {
            child
                .transitions(&events)
                .flat_map(move |(mut initials, after)| {
                    // If any child process can perform τ, then the external choice can as well,
                    // but it _does not_ resolve the choice.
                    let tau_transitions = if initials.can_perform_tau() {
                        initials.subtract(&E::tau());
                        // All other processes in the choice remain the same.  The process that
                        // performed τ advances forward to its next state.
                        let mut tau_children = self.0.clone();
                        tau_children[index] = after.clone();
                        Either::Left(std::iter::once((
                            E::tau(),
                            CSP::replicated_external_choice(tau_children),
                        )))
                    } else {
                        Either::Right(std::iter::empty())
                    };

                    // If any child process can perform a non-τ event, then the external choice can
                    // as well, and it _does_ resolve the choice.  Note that we've already removed
                    // τ from `initials` at this point.
                    let other_transitions = if !initials.is_empty() {
                        Either::Left(std::iter::once((initials, after)))
                    } else {
                        Either::Right(std::iter::empty())
                    };

                    tau_transitions.chain(other_transitions)
                })
        })
    }
}

#[cfg(test)]
mod external_choice_tests {
    use super::*;

    use maplit::hashset;
    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::maximal_traces::maximal_finite_traces;
    use crate::maximal_traces::MaximalTraces;
    use crate::test_support::NonemptyVec;
    use crate::test_support::TestEvents;

    #[test]
    fn check_empty_external_choice_initials() {
        let process = CSP::<TestEvents, _, _>::replicated_external_choice(vec![]);
        assert_eq!(process.initials(), TestEvents::empty());
    }

    #[test]
    fn check_empty_external_choice_traces() {
        let process = CSP::<TestEvents, _, _>::replicated_external_choice(vec![]);
        assert_eq!(maximal_finite_traces(&process), hashset! {vec![]});
    }

    #[proptest]
    fn check_singleton_external_choice_initials(p: CSP<TestEvents, _, _>) {
        let process = CSP::replicated_external_choice(vec![p.clone()]);
        assert_eq!(process.initials(), p.initials());
    }

    #[proptest]
    fn check_singleton_external_choice_traces(p: CSP<TestEvents, _, _>) {
        let process = CSP::replicated_external_choice(vec![p.clone()]);
        assert_eq!(maximal_finite_traces(&process), maximal_finite_traces(&p));
    }

    #[proptest]
    fn check_doubleton_external_choice_initials(
        p: CSP<TestEvents, _, _>,
        q: CSP<TestEvents, _, _>,
    ) {
        let process = CSP::external_choice(p.clone(), q.clone());
        let mut expected = p.initials();
        expected.union(&q.initials());
        assert_eq!(process.initials(), expected);
    }

    #[proptest]
    fn check_doubleton_external_choice_traces(p: CSP<TestEvents, _, _>, q: CSP<TestEvents, _, _>) {
        let process = CSP::external_choice(p.clone(), q.clone());
        assert_eq!(
            maximal_finite_traces(&process),
            maximal_finite_traces(&p) + maximal_finite_traces(&q)
        );
    }

    #[proptest]
    fn check_replicated_external_choice_initials(ps: NonemptyVec<CSP<TestEvents, _, _>>) {
        let process = CSP::replicated_external_choice(ps.vec.clone());
        let expected = ps.vec.iter().fold(TestEvents::empty(), |mut expected, p| {
            expected.union(&p.initials());
            expected
        });
        assert_eq!(process.initials(), expected);
    }

    #[proptest]
    fn check_replicated_external_choice_traces(ps: NonemptyVec<CSP<TestEvents, _, _>>) {
        let process = CSP::replicated_external_choice(ps.vec.clone());
        assert_eq!(
            maximal_finite_traces(&process),
            ps.vec
                .iter()
                .map(maximal_finite_traces)
                .sum::<MaximalTraces<_>>()
        );
    }
}
