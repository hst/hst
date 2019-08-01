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

//! Defines the prefix (→) operator.

use std::fmt::Debug;
use std::fmt::Display;

use auto_enums::enum_derive;

use crate::process::Afters;
use crate::process::Cursor;
use crate::process::Initials;
use crate::process::Process;

/// Constructs a new _prefix_ process `a → P`.  This process performs event `a` and then behaves
/// like process `P`.
pub fn prefix<E, P: From<Prefix<E, P>>>(initial: E, after: P) -> P {
    Prefix(initial, after).into()
}

/// The type of a [`prefix`] process.
///
/// [`prefix`]: fn.prefix.html
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

impl<E, P> Cursor<E> for PrefixCursor<E, P>
where
    E: Clone + Display + Eq + 'static,
    P: Cursor<E>,
{
    fn events(&self) -> Box<dyn Iterator<Item = E>> {
        match self.state {
            PrefixState::BeforeInitial => Box::new(std::iter::once(self.initial.clone())),
            PrefixState::AfterInitial => self.after.events(),
        }
    }

    fn can_perform(&self, event: &E) -> bool {
        match self.state {
            PrefixState::BeforeInitial => *event == self.initial,
            PrefixState::AfterInitial => self.after.can_perform(event),
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

impl<'a, E, P> Initials<'a, E> for Prefix<E, P>
where
    E: Clone + 'a,
{
    type Initials = std::iter::Once<E>;

    fn initials(&'a self) -> Self::Initials {
        // initials(a → P) = {a}
        std::iter::once(self.0.clone())
    }
}

#[doc(hidden)]
#[enum_derive(Iterator)]
pub enum PrefixAfters<Initial, NotInitial> {
    Initial(Initial),
    NotInitial(NotInitial),
}

impl<'a, E, P> Afters<'a, E, P> for Prefix<E, P>
where
    E: Eq,
    P: Clone + 'a,
{
    type Afters = PrefixAfters<std::iter::Once<P>, std::iter::Empty<P>>;

    fn afters(&'a self, initial: &E) -> Self::Afters {
        // afters(a → P, a) = P
        if *initial == self.0 {
            PrefixAfters::Initial(std::iter::once(self.1.clone()))
        } else {
            PrefixAfters::NotInitial(std::iter::empty())
        }
    }
}

#[cfg(test)]
mod prefix_tests {
    use super::*;

    use maplit::hashmap;
    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::primitives::stop;
    use crate::primitives::tau;
    use crate::primitives::Stop;
    use crate::process::satisfies_trace;
    use crate::process::transitions;
    use crate::test_support::NumberedEvent;
    use crate::test_support::TestEvent;

    #[proptest]
    fn check_prefix_events(initial: NumberedEvent) {
        let initial = TestEvent::from(initial);
        // TODO: Use CSP<TestEvent> once all operators implement Process
        let process = Prefix::<TestEvent, Stop<TestEvent>>(initial.clone(), stop());
        let mut cursor = process.root();
        assert_eq!(cursor.events().collect::<Vec<_>>(), vec![initial.clone()]);
        cursor.perform(&initial);
        assert!(cursor.events().collect::<Vec<_>>().is_empty());
    }

    #[proptest]
    fn check_prefix_traces(initial: NumberedEvent) {
        let initial = TestEvent::from(initial);
        // TODO: Use CSP<TestEvent> once all operators implement Process
        let process = Prefix::<TestEvent, Stop<TestEvent>>(initial.clone(), stop());
        let cursor = process.root();
        assert!(satisfies_trace(cursor.clone(), vec![initial.clone()]));
        assert!(!satisfies_trace(cursor.clone(), vec![tau()]));
        assert!(!satisfies_trace(
            cursor.clone(),
            vec![initial.clone(), tau()]
        ));
    }

    #[proptest]
    fn check_prefix_transitions(initial: NumberedEvent, after: CSP<TestEvent>) {
        let initial = TestEvent::from(initial);
        let process = prefix(initial.clone(), after.clone());
        let transitions = transitions(&process);
        assert_eq!(transitions, hashmap! { initial => vec![after] });
    }
}
