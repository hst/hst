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

//! Defines the sequential composition (`;`) operator.

use std::fmt::Debug;
use std::fmt::Display;
use std::marker::PhantomData;

use crate::primitives::tau;
use crate::primitives::tick;
use crate::primitives::Tau;
use crate::primitives::Tick;
use crate::process::Cursor;
use crate::process::Process;

/// Constructs a new _sequential composition_ process `P ; Q`.  This process behaves like process
/// `P` until it performs a ✔ event, after which is behaves like process `Q`.

pub fn sequential_composition<P: From<SequentialComposition<P>>>(p: P, q: P) -> P {
    SequentialComposition(p, q).into()
}

#[doc(hidden)]
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct SequentialComposition<P>(P, P);

impl<P: Debug + Display> Display for SequentialComposition<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} ; {}", self.0, self.1)
    }
}

impl<P: Debug + Display> Debug for SequentialComposition<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &dyn Display).fmt(f)
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

#[doc(hidden)]
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct SequentialCompositionCursor<E, C> {
    phantom: PhantomData<E>,
    state: SequentialCompositionState,
    p: C,
    // ✔ events are ambiguous; they could represent P performing a τ, or P performing a ✔ that we
    // "hide" as we switch over to behaving like Q.  That means we could start behaving like Q at
    // multiple points, and need to keep track of Q's current state from all of those possible
    // starting points.  The Option lets us "deactive" one of those states if we retroactively
    // discover that it wasn't possible, by not being able to perform some later visible event.
    qs: Vec<Option<C>>,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SequentialCompositionState {
    // We are definitely still in the phase where we've behaving like P.
    BeforeTick,
    // We are definitely in the phase where we've behaving like Q.
    AfterTick,
    // We could be behaving like either P or Q, because we don't have enough information yet to
    // know whether a τ that we performed was a real τ from P, or a ✔ from P that we transformed
    // into a τ.
    Shrug,
}

struct Subcursors<'a, C>(&'a Vec<Option<C>>);

impl<E, C> Debug for SequentialCompositionCursor<E, C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("SequentialCompositionCursor")
            .field("state", &self.state)
            .field("p", &self.p)
            .field("qs", &Subcursors(&self.qs))
            .finish()
    }
}

impl<'a, C> Debug for Subcursors<'a, C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.iter().filter_map(|subcursor| subcursor.as_ref()))
            .finish()
    }
}

impl<E, P> Process<E> for SequentialComposition<P>
where
    E: Eq + From<Tau> + From<Tick> + 'static,
    P: Process<E>,
{
    type Cursor = SequentialCompositionCursor<E, P::Cursor>;

    fn root(&self) -> Self::Cursor {
        SequentialCompositionCursor {
            phantom: PhantomData,
            state: SequentialCompositionState::BeforeTick,
            p: self.0.root(),
            qs: vec![Some(self.1.root())],
        }
    }
}

impl<E, C> SequentialCompositionCursor<E, C>
where
    E: Eq + From<Tau> + From<Tick>,
    C: Cursor<E>,
{
    fn p_events(&self) -> impl Iterator<Item = E> + '_ {
        self.p.events().map(|e| if e == tick() { tau() } else { e })
    }

    fn p_can_perform(&self, event: &E) -> bool {
        if *event == tick() {
            false
        } else if *event == tau() {
            self.p.can_perform(event) || self.p.can_perform(&tick())
        } else {
            self.p.can_perform(event)
        }
    }

    fn p_perform(&mut self, event: &E) {
        if *event == tick() {
            return;
        }

        if *event == tau() {
            // If P can perform a τ (not a ✔ transformed into a τ), then we might still be
            // in the P process after performing this event.
            let could_be_before = self.p.can_perform(event);

            // If P can perform a ✔, then we might become Q after performing this event.
            let could_be_after = self.p.can_perform(&tick());

            if !could_be_before && !could_be_after {
                // We were in a state where we thought that we could be behaving like P, but P
                // can't perform this event.  That means we can't be behaving like P after all!
                // We'd better already be in a state where we could be performing like Q, since
                // that's now the only remaining possibility.
                assert!(self.state != SequentialCompositionState::BeforeTick);
                self.state = SequentialCompositionState::AfterTick;
                return;
            }

            if could_be_before {
                self.p.perform(event);
            }
            if could_be_after {
                if could_be_before {
                    self.state = SequentialCompositionState::Shrug;
                } else {
                    self.state = SequentialCompositionState::AfterTick;
                }
            }

            return;
        }

        if self.p.can_perform(event) {
            self.p.perform(event);
            return;
        }

        // We were in a state where we thought that we could be behaving like P, but P can't
        // perform this event.  That means we can't be behaving like P after all!  We'd better
        // already be in a state where we could be performing like Q, since that's now the only
        // remaining possibility.
        assert!(self.state != SequentialCompositionState::BeforeTick);
        self.state = SequentialCompositionState::AfterTick;
    }

    fn q_events(&self) -> impl Iterator<Item = E> + '_ {
        self.qs.iter().flatten().flat_map(C::events)
    }

    fn q_can_perform(&self, event: &E) -> bool {
        self.qs.iter().flatten().any(|q| q.can_perform(event))
    }

    fn q_perform(&mut self, event: &E) {
        for q in &mut self.qs {
            match q {
                Some(q) if q.can_perform(event) => q.perform(event),
                Some(_) => {
                    q.take();
                }
                _ => (),
            }
        }
    }
}

impl<E, C> Cursor<E> for SequentialCompositionCursor<E, C>
where
    E: Eq + From<Tau> + From<Tick>,
    C: Cursor<E>,
{
    fn events<'a>(&'a self) -> Box<dyn Iterator<Item = E> + 'a> {
        match self.state {
            SequentialCompositionState::BeforeTick => Box::new(self.p_events()),
            SequentialCompositionState::AfterTick => Box::new(self.q_events()),
            SequentialCompositionState::Shrug => {
                // If we don't know if we're in P or Q, we have to combine the possible events from
                // both of them!
                Box::new(self.p_events().chain(self.q_events()))
            }
        }
    }

    fn can_perform(&self, event: &E) -> bool {
        match self.state {
            SequentialCompositionState::BeforeTick => self.p_can_perform(event),
            SequentialCompositionState::AfterTick => self.q_can_perform(event),
            SequentialCompositionState::Shrug => {
                self.p_can_perform(event) || self.q_can_perform(event)
            }
        }
    }

    fn perform(&mut self, event: &E) {
        match self.state {
            SequentialCompositionState::BeforeTick => self.p_perform(event),
            SequentialCompositionState::AfterTick => self.q_perform(event),
            SequentialCompositionState::Shrug => {
                self.p_perform(event);
                self.q_perform(event);
            }
        }
    }
}

#[cfg(test)]
mod sequential_composition_tests {
    use super::*;

    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::primitives::tick;
    use crate::process::initials;
    use crate::process::maximal_finite_traces;
    use crate::process::MaximalTraces;
    use crate::test_support::TestEvent;

    #[proptest]
    fn check_sequential_composition(p: CSP<TestEvent>, q: CSP<TestEvent>) {
        let process = dbg!(sequential_composition(p.clone(), q.clone()));

        // We need to replace ✔ with τ in the initials of P.
        let mut expected = initials(&p.root());
        if expected.remove(&tick()) {
            expected.insert(tau());
        }
        assert_eq!(initials(&process.root()), expected);

        // For any trace of P, we need to replace a ✔ at the end with all possible traces of Q.
        let mut expected = MaximalTraces::new();
        for mut trace in maximal_finite_traces(p.root()) {
            if trace.ends_with(&vec![tick()]) {
                trace.pop();
                expected.insert(trace.clone());
                for suffix in maximal_finite_traces(q.root()) {
                    let mut combined = trace.clone();
                    combined.extend(suffix);
                    expected.insert(combined);
                }
            } else {
                expected.insert(trace);
            }
        }
        assert_eq!(maximal_finite_traces(process.root()), expected);
    }
}
