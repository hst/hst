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

//! Defines the internal choice (`⊓`) operator.

use std::fmt::Debug;
use std::fmt::Display;
use std::marker::PhantomData;

use smallvec::smallvec;
use smallvec::SmallVec;

use crate::possibilities::Possibilities;
use crate::primitives::tau;
use crate::primitives::Tau;
use crate::process::Cursor;
use crate::process::Process;

/// Constructs a new _internal choice_ process `P ⊓ Q`.  This process behaves either like `P` _or_
/// `Q`, but the environment has no control over which one is chosen.
pub fn internal_choice<P: From<InternalChoice<P>>>(p: P, q: P) -> P {
    InternalChoice(smallvec![p, q]).into()
}

/// Constructs a new _replicated internal choice_ process `⊓ Ps` over a non-empty collection of
/// processes.  The process behaves like one of the processes in the set, but the environment has
/// no control over which one is chosen.
///
/// Panics if `ps` is empty.
pub fn replicated_internal_choice<P: From<InternalChoice<P>>, I: IntoIterator<Item = P>>(
    ps: I,
) -> P {
    let ps: SmallVec<[P; 2]> = ps.into_iter().collect();
    assert!(
        !ps.is_empty(),
        "Cannot perform internal choice over no processes"
    );
    InternalChoice(ps).into()
}

#[doc(hidden)]
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct InternalChoice<P>(SmallVec<[P; 2]>);

impl<P: Debug + Display> Display for InternalChoice<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.0.len() == 2 {
            write!(f, "{} ⊓ {}", self.0[0], self.0[1])
        } else {
            f.write_str("⊓ ")?;
            f.debug_set().entries(&self.0).finish()
        }
    }
}

impl<P: Debug + Display> Debug for InternalChoice<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &dyn Display).fmt(f)
    }
}

// Operational semantics for ⊓ Ps
//
// 1) ──────────── P ∈ Ps
//     ⊓ Ps -τ→ P

#[doc(hidden)]
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct InternalChoiceCursor<E, C> {
    phantom: PhantomData<E>,
    state: InternalChoiceState,
    possibilities: Possibilities<E, C>,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum InternalChoiceState {
    BeforeTau,
    AfterTau,
}

impl<E, C> Debug for InternalChoiceCursor<E, C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("InternalChoiceCursor")
            .field("state", &self.state)
            .field("subcursors", &self.possibilities)
            .finish()
    }
}

impl<E, P> Process<E> for InternalChoice<P>
where
    E: Display + Eq + From<Tau> + 'static,
    P: Process<E>,
    P::Cursor: Clone,
{
    type Cursor = InternalChoiceCursor<E, P::Cursor>;

    fn root(&self) -> Self::Cursor {
        InternalChoiceCursor {
            phantom: PhantomData,
            state: InternalChoiceState::BeforeTau,
            possibilities: Possibilities::new(self.0.iter().map(P::root)),
        }
    }
}

impl<E, C> Cursor<E> for InternalChoiceCursor<E, C>
where
    E: Display + Eq + From<Tau>,
    C: Clone + Cursor<E>,
{
    fn events<'a>(&'a self) -> Box<dyn Iterator<Item = E> + 'a> {
        match self.state {
            InternalChoiceState::BeforeTau => Box::new(std::iter::once(tau())),
            InternalChoiceState::AfterTau => Box::new(self.possibilities.events()),
        }
    }

    fn can_perform(&self, event: &E) -> bool {
        match self.state {
            InternalChoiceState::BeforeTau => *event == tau(),
            InternalChoiceState::AfterTau => self.possibilities.can_perform(event),
        }
    }

    fn perform(&mut self, event: &E) {
        if self.state == InternalChoiceState::AfterTau {
            self.possibilities.perform_all(event);
            return;
        }

        if *event != tau() {
            panic!("Internal choice cannot perform {}", event);
        }
        self.state = InternalChoiceState::AfterTau;
    }
}

#[cfg(test)]
mod internal_choice_tests {
    use super::*;

    use maplit::hashset;
    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::primitives::tau;
    use crate::process::initials;
    use crate::process::maximal_finite_traces;
    use crate::process::MaximalTraces;
    use crate::test_support::NonemptyVec;
    use crate::test_support::TestEvent;

    #[proptest]
    fn check_singleton_internal_choice(p: CSP<TestEvent>) {
        let process = replicated_internal_choice(vec![p.clone()]);
        assert_eq!(initials(&process.root()), hashset! {tau()});
        assert_eq!(
            maximal_finite_traces(process.root()),
            maximal_finite_traces(p.root())
        );
    }

    #[proptest]
    fn check_doubleton_internal_choice(p: CSP<TestEvent>, q: CSP<TestEvent>) {
        let process = internal_choice(p.clone(), q.clone());
        assert_eq!(initials(&process.root()), hashset! {tau()});
        assert_eq!(
            maximal_finite_traces(process.root()),
            maximal_finite_traces(p.root()) + maximal_finite_traces(q.root())
        );
    }

    #[proptest]
    fn check_replicated_internal_choice_transitions(ps: NonemptyVec<CSP<TestEvent>>) {
        let process = replicated_internal_choice(ps.vec.clone());
        assert_eq!(initials(&process.root()), hashset! {tau()});
        assert_eq!(
            maximal_finite_traces(process.root()),
            ps.vec
                .iter()
                .map(Process::root)
                .map(maximal_finite_traces)
                .sum::<MaximalTraces<_>>()
        );
    }
}
