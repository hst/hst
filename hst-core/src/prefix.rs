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

use crate::process::Afters;
use crate::process::Initials;

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
        (self as &Display).fmt(f)
    }
}

// Operational semantics for a → P
//
// 1) ─────────────
//     a → P -a→ P

impl<E, P> Initials<E> for Prefix<E, P>
where
    E: Clone,
{
    type Initials = std::iter::Once<E>;

    fn initials(&self) -> Self::Initials {
        // initials(a → P) = {a}
        std::iter::once(self.0.clone())
    }
}

impl<E, P> Afters<E, P> for Prefix<E, P>
where
    E: Eq,
    P: Clone,
{
    type Afters = std::iter::Once<P>;

    fn afters(&self, initial: &E) -> Option<Self::Afters> {
        // afters(a → P, a) = P
        if *initial == self.0 {
            Some(std::iter::once(self.1.clone()))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod prefix_tests {
    use super::*;

    use maplit::hashmap;
    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::process::transitions;
    use crate::test_support::NumberedEvent;
    use crate::test_support::TestEvent;

    #[proptest]
    fn check_prefix_transitions(initial: NumberedEvent, after: CSP<TestEvent>) {
        let initial = TestEvent::from(initial);
        let process = prefix(initial.clone(), after.clone());
        let transitions = transitions(&process);
        assert_eq!(transitions, hashmap! { initial => vec![after] });
    }
}
