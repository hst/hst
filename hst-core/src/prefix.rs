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

//! Defines the prefix (`→`) operator.

use std::fmt::Debug;
use std::fmt::Display;

use auto_enums::enum_derive;

use crate::event::Alphabet;
use crate::process::Cursor;
use crate::process::Process;

/// Constructs a new _prefix_ process `a → P`.  This process performs event `a` and then behaves
/// like process `P`.
pub fn prefix<E, P: From<Prefix<E, P>>>(initial: E, after: P) -> P {
    Prefix(initial, after).into()
}

#[doc(hidden)]
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Prefix<E, P>(E, P);

impl<E: Display, P: Display> Display for Prefix<E, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} → {}", self.0, self.1)
    }
}

impl<E: Display, P: Display> Debug for Prefix<E, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &dyn Display).fmt(f)
    }
}

// Operational semantics for a → P
//
// 1) ─────────────
//     a → P -a→ P

#[doc(hidden)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PrefixCursor<E, P> {
    state: PrefixState,
    initial: E,
    after: P,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PrefixState {
    BeforeInitial,
    AfterInitial,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PrefixAlphabet<E, A> {
    BeforeInitial(E),
    AfterInitial(A),
}

impl<E, P> Process<E> for Prefix<E, P>
where
    E: Clone + Display + Eq + 'static,
    P: Process<E>,
{
    type Cursor = PrefixCursor<E, P::Cursor>;

    fn root(&self) -> Self::Cursor {
        PrefixCursor {
            state: PrefixState::BeforeInitial,
            initial: self.0.clone(),
            after: self.1.root(),
        }
    }
}

impl<E, C> Cursor<E> for PrefixCursor<E, C>
where
    E: Clone + Display + Eq + 'static,
    C: Cursor<E>,
{
    type Alphabet = PrefixAlphabet<E, C::Alphabet>;

    fn initials(&self) -> PrefixAlphabet<E, C::Alphabet> {
        match self.state {
            PrefixState::BeforeInitial => PrefixAlphabet::BeforeInitial(self.initial.clone()),
            PrefixState::AfterInitial => PrefixAlphabet::AfterInitial(self.after.initials()),
        }
    }

    fn perform(&mut self, event: &E) {
        if self.state == PrefixState::AfterInitial {
            self.after.perform(event);
            return;
        }
        if *event != self.initial {
            panic!("Prefix cannot perform {}", event);
        }
        self.state = PrefixState::AfterInitial;
    }
}

impl<E, A> Alphabet<E> for PrefixAlphabet<E, A>
where
    E: Eq,
    A: Alphabet<E>,
{
    fn contains(&self, event: &E) -> bool {
        match self {
            PrefixAlphabet::BeforeInitial(initial) => initial == event,
            PrefixAlphabet::AfterInitial(alphabet) => alphabet.contains(event),
        }
    }
}

#[doc(hidden)]
#[enum_derive(Iterator)]
pub enum PrefixAlphabetIterator<E, A> {
    BeforeInitial(std::iter::Once<E>),
    AfterInitial(A),
}

impl<E, A> IntoIterator for PrefixAlphabet<E, A>
where
    E: Clone,
    A: IntoIterator<Item = E>,
{
    type Item = E;
    type IntoIter = PrefixAlphabetIterator<E, A::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            PrefixAlphabet::BeforeInitial(initial) => {
                PrefixAlphabetIterator::BeforeInitial(std::iter::once(initial.clone()))
            }
            PrefixAlphabet::AfterInitial(alphabet) => {
                PrefixAlphabetIterator::AfterInitial(alphabet.into_iter())
            }
        }
    }
}

#[cfg(test)]
mod prefix_tests {
    use super::*;

    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::process::maximal_finite_traces;
    use crate::test_support::NumberedEvent;
    use crate::test_support::TestEvent;

    #[proptest]
    fn check_prefix_initials(event: TestEvent, initial: NumberedEvent, after: CSP<TestEvent>) {
        let initial = TestEvent::from(initial);
        let process = dbg!(prefix(initial.clone(), after.clone()));

        let alphabet = process.root().initials();
        assert!(alphabet.contains(&initial));
        assert_eq!(alphabet.contains(&event), event == initial);

        let alphabet = process.root().after(&initial).initials();
        assert_eq!(
            alphabet.contains(&event),
            after.root().initials().contains(&event)
        );
    }

    #[proptest]
    fn check_prefix_traces(initial: NumberedEvent, after: CSP<TestEvent>) {
        let initial = TestEvent::from(initial);
        let process = dbg!(prefix(initial.clone(), after.clone()));
        assert_eq!(
            maximal_finite_traces(process.root()),
            maximal_finite_traces(after.root()).map(|trace| trace.insert(0, initial.clone()))
        );
    }
}
