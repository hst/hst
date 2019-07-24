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

#[doc(hidden)]
pub struct PrefixInitials<E>(E);

impl<E> IntoIterator for PrefixInitials<E> {
    type Item = E;
    type IntoIter = std::iter::Once<E>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self.0)
    }
}

impl<E, P> Initials<E> for Prefix<E, P>
where
    E: Clone,
{
    type Initials = PrefixInitials<E>;

    fn initials(&self) -> Self::Initials {
        PrefixInitials(self.0.clone())
    }
}

#[doc(hidden)]
pub struct PrefixAfters<P>(P);

impl<P> IntoIterator for PrefixAfters<P> {
    type Item = P;
    type IntoIter = std::iter::Once<P>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self.0)
    }
}

impl<E, P> Afters<E, P> for Prefix<E, P>
where
    E: Eq,
    P: Clone,
{
    type Afters = PrefixAfters<P>;

    fn afters(&self, initial: &E) -> Option<Self::Afters> {
        if *initial == self.0 {
            Some(PrefixAfters(self.1.clone()))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod prefix_tests {
    use super::*;

    use std::collections::HashMap;

    use maplit::hashmap;

    use crate::csp::CSP;
    use crate::primitives::stop;
    use crate::process::transitions;
    use crate::test_support::numbered_event;
    use crate::test_support::TestEvent;

    #[test]
    fn check_prefix_transitions() {
        let process = prefix(numbered_event(0), stop());
        let transitions: HashMap<TestEvent, Vec<CSP<TestEvent>>> = transitions(&process);
        assert_eq!(transitions, hashmap! { numbered_event(0) => vec![stop()] });
    }
}
