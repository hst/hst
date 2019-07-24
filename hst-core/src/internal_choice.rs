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

//! Defines the internal choice (⊓) operator.

use std::fmt::Debug;
use std::fmt::Display;

use crate::primitives::tau;
use crate::primitives::Tau;
use crate::process::Afters;
use crate::process::Initials;

/// Constructs a new _internal choice_ process `P ⊓ Q`.  This process behaves either like `P` _or_
/// `Q`, but the environment has no control of which one is chosen.
pub fn internal_choice<P: From<InternalChoice<P>>>(p: P, q: P) -> P {
    InternalChoice(p, q).into()
}

/// The type of an [`internal_choice`] process.
///
/// [`internal_choice`]: fn.internal_choice.html
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct InternalChoice<P>(P, P);

impl<P: Display> Display for InternalChoice<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} ⊓ {}", self.0, self.1)
    }
}

impl<P: Display> Debug for InternalChoice<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &Display).fmt(f)
    }
}

// Operational semantics for ⊓ Ps
//
// 1) ──────────── P ∈ Ps
//     ⊓ Ps -τ→ P

impl<E, P> Initials<E> for InternalChoice<P>
where
    E: From<Tau>,
{
    type Initials = std::iter::Once<E>;

    fn initials(&self) -> Self::Initials {
        // initials(⊓ Ps) = {τ}
        std::iter::once(tau())
    }
}

impl<E, P> Afters<E, P> for InternalChoice<P>
where
    E: Eq + From<Tau>,
    P: Clone,
{
    type Afters = std::vec::IntoIter<P>;

    fn afters(&self, initial: &E) -> Option<Self::Afters> {
        // afters(⊓ Ps, τ) = Ps
        if *initial == tau() {
            Some(vec![self.0.clone(), self.1.clone()].into_iter())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod internal_choice_tests {
    use super::*;

    use maplit::hashmap;
    use maplit::hashset;
    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::process::transitions;
    use crate::test_support::TestEvent;

    #[proptest]
    fn check_internal_choice_transitions(p: CSP<TestEvent>, q: CSP<TestEvent>) {
        let process = internal_choice(p.clone(), q.clone());
        let transitions = transitions(&process);
        assert_eq!(transitions, hashmap! { tau() => hashset![p, q] });
    }
}
