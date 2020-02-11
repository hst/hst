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

use auto_enums::enum_derive;
use smallbitvec::SmallBitVec;
use smallvec::smallvec;
use smallvec::SmallVec;

use crate::event::Alphabet;
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
    /// Indicates which child processes are still a possible result of the original internal
    /// choice.  (Visible events after a τ can rule out child processes that aren't able to perform
    /// that event.)
    activated: SmallBitVec,
    /// The cursors that track the current state of each child process.  If the corresponding entry
    /// in `activated` is unset, then that child process is no longer a possible result, and its
    /// cursor is ignored.
    subcursors: SmallVec<[C; 2]>,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum InternalChoiceState {
    BeforeTau,
    AfterTau,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum InternalChoiceAlphabet<E, A> {
    BeforeTau(PhantomData<E>),
    AfterTau(SmallVec<[A; 2]>),
}

struct ActivatedSubcursors<'a, E, C>(&'a InternalChoiceCursor<E, C>);

impl<E, C> Debug for InternalChoiceCursor<E, C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("InternalChoiceCursor")
            .field("state", &self.state)
            .field("subcursors", &ActivatedSubcursors(self))
            .finish()
    }
}

impl<'a, E, C> Debug for ActivatedSubcursors<'a, E, C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.activated_subcursors())
            .finish()
    }
}

impl<E, C> InternalChoiceCursor<E, C> {
    /// Returns an iterator of the subcursors that are still activated.
    fn activated_subcursors<'a>(&'a self) -> impl Iterator<Item = &C> + 'a {
        self.activated
            .iter()
            .zip(&self.subcursors)
            .filter(|(activated, _)| *activated)
            .map(|(_, subcursor)| subcursor)
    }
}

impl<E, P> Process<E> for InternalChoice<P>
where
    E: Display + Eq + From<Tau> + 'static,
    P: Process<E>,
{
    type Cursor = InternalChoiceCursor<E, P::Cursor>;

    fn root(&self) -> Self::Cursor {
        InternalChoiceCursor {
            phantom: PhantomData,
            state: InternalChoiceState::BeforeTau,
            activated: SmallBitVec::from_elem(self.0.len(), true),
            subcursors: self.0.iter().map(P::root).collect(),
        }
    }
}

impl<E, C> Cursor<E> for InternalChoiceCursor<E, C>
where
    E: Display + Eq + From<Tau>,
    C: Cursor<E>,
{
    type Alphabet = InternalChoiceAlphabet<E, C::Alphabet>;

    fn initials(&self) -> InternalChoiceAlphabet<E, C::Alphabet> {
        match self.state {
            InternalChoiceState::BeforeTau => InternalChoiceAlphabet::BeforeTau(PhantomData),
            InternalChoiceState::AfterTau => InternalChoiceAlphabet::AfterTau(
                self.activated_subcursors().map(C::initials).collect(),
            ),
        }
    }

    fn perform(&mut self, event: &E) {
        if self.state == InternalChoiceState::AfterTau {
            for (idx, subcursor) in self.subcursors.iter_mut().enumerate() {
                // Is this subprocess already deactivated?
                if !self.activated[idx] {
                    continue;
                }
                // If it can't perform this event, then it _becomes_ deactivated.
                if !subcursor.can_perform(event) {
                    unsafe { self.activated.set_unchecked(idx, false) };
                    continue;
                }
                // Otherwise allow the subprocess to perform the event.
                subcursor.perform(event);
            }
            return;
        }

        if *event != tau() {
            panic!("Internal choice cannot perform {}", event);
        }
        self.state = InternalChoiceState::AfterTau;
    }
}

impl<E, A> Alphabet<E> for InternalChoiceAlphabet<E, A>
where
    E: Eq + From<Tau>,
    A: Alphabet<E>,
{
    fn contains(&self, event: &E) -> bool {
        match self {
            InternalChoiceAlphabet::BeforeTau(_) => *event == tau(),
            InternalChoiceAlphabet::AfterTau(alphabets) => {
                alphabets.iter().any(|alphabet| alphabet.contains(event))
            }
        }
    }
}

#[doc(hidden)]
#[enum_derive(Iterator)]
pub enum InternalChoiceAlphabetIterator<E, A>
where
    A: IntoIterator<Item = E>,
{
    BeforeTau(std::iter::Once<E>),
    AfterTau(std::iter::FlatMap<smallvec::IntoIter<[A; 2]>, A::IntoIter, fn(A) -> A::IntoIter>),
}

impl<E, A> IntoIterator for InternalChoiceAlphabet<E, A>
where
    E: From<Tau>,
    A: IntoIterator<Item = E>,
{
    type Item = E;
    type IntoIter = InternalChoiceAlphabetIterator<E, A>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            InternalChoiceAlphabet::BeforeTau(_) => {
                InternalChoiceAlphabetIterator::BeforeTau(std::iter::once(tau()))
            }
            InternalChoiceAlphabet::AfterTau(alphabets) => {
                InternalChoiceAlphabetIterator::AfterTau(
                    alphabets.into_iter().flat_map(A::into_iter),
                )
            }
        }
    }
}

#[cfg(test)]
mod internal_choice_tests {
    use super::*;

    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::primitives::tau;
    use crate::process::maximal_finite_traces;
    use crate::process::MaximalTraces;
    use crate::test_support::NonemptyVec;
    use crate::test_support::TestEvent;

    #[proptest]
    fn check_singleton_internal_choice_initials(event: TestEvent, p: CSP<TestEvent>) {
        let process = dbg!(replicated_internal_choice(vec![p.clone()]));

        let alphabet = dbg!(process.root().initials());
        assert_eq!(alphabet.contains(&event), event == tau());

        let alphabet = process.root().after(&tau()).initials();
        assert_eq!(
            alphabet.contains(&event),
            p.root().initials().contains(&event)
        );
    }

    #[proptest]
    fn check_singleton_internal_choice_traces(p: CSP<TestEvent>) {
        let process = dbg!(replicated_internal_choice(vec![p.clone()]));
        assert_eq!(
            maximal_finite_traces(process.root()),
            maximal_finite_traces(p.root())
        );
    }

    #[proptest]
    fn check_doubleton_internal_choice_initials(
        event: TestEvent,
        p: CSP<TestEvent>,
        q: CSP<TestEvent>,
    ) {
        let process = dbg!(internal_choice(p.clone(), q.clone()));

        let alphabet = dbg!(process.root().initials());
        assert_eq!(alphabet.contains(&event), event == tau());

        let alphabet = process.root().after(&tau()).initials();
        assert_eq!(
            alphabet.contains(&event),
            p.root().initials().contains(&event) || q.root().initials().contains(&event)
        );
    }

    #[proptest]
    fn check_doubleton_internal_choice_traces(p: CSP<TestEvent>, q: CSP<TestEvent>) {
        let process = dbg!(internal_choice(p.clone(), q.clone()));
        assert_eq!(
            maximal_finite_traces(process.root()),
            maximal_finite_traces(p.root()) + maximal_finite_traces(q.root())
        );
    }

    #[proptest]
    fn check_replicated_internal_choice_initials(
        event: TestEvent,
        ps: NonemptyVec<CSP<TestEvent>>,
    ) {
        let process = dbg!(replicated_internal_choice(ps.vec.clone()));

        let alphabet = dbg!(process.root().initials());
        assert_eq!(alphabet.contains(&event), event == tau());

        let alphabet = process.root().after(&tau()).initials();
        assert_eq!(
            alphabet.contains(&event),
            ps.vec.iter().any(|p| p.root().initials().contains(&event))
        );
    }

    #[proptest]
    fn check_replicated_internal_choice_traces(ps: NonemptyVec<CSP<TestEvent>>) {
        let process = dbg!(replicated_internal_choice(ps.vec.clone()));
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
