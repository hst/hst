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

use smallvec::smallvec;
use smallvec::SmallVec;

use crate::primitives::tau;
use crate::primitives::Tau;
use crate::process::Afters;
use crate::process::Initials;

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

/// The type of an [`internal_choice`] process.
///
/// [`internal_choice`]: fn.internal_choice.html
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
        (self as &Display).fmt(f)
    }
}

// Operational semantics for ⊓ Ps
//
// 1) ──────────── P ∈ Ps
//     ⊓ Ps -τ→ P

impl<'a, E, P> Initials<'a, E> for InternalChoice<P>
where
    E: From<Tau> + 'a,
{
    type Initials = std::iter::Once<E>;

    fn initials(&'a self) -> Self::Initials {
        // initials(⊓ Ps) = {τ}
        std::iter::once(tau())
    }
}

impl<'a, E, P> Afters<'a, E, P> for InternalChoice<P>
where
    E: Eq + From<Tau>,
    P: Clone + 'a,
{
    type Afters = std::iter::Cloned<std::slice::Iter<'a, P>>;

    fn afters(&'a self, initial: &E) -> Option<Self::Afters> {
        // afters(⊓ Ps, τ) = Ps
        if *initial == tau() {
            Some(self.0.iter().cloned())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod internal_choice_tests {
    use super::*;

    use maplit::hashmap;
    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::process::transitions;
    use crate::test_support::NonemptyVec;
    use crate::test_support::TestEvent;

    #[proptest]
    fn check_internal_choice_transitions(p: CSP<TestEvent>, q: CSP<TestEvent>) {
        let process = internal_choice(p.clone(), q.clone());
        let transitions = transitions(&process);
        assert_eq!(transitions, hashmap! { tau() => vec![p, q] });
    }

    #[proptest]
    fn check_replicated_internal_choice_transitions(ps: NonemptyVec<CSP<TestEvent>>) {
        let process = replicated_internal_choice(ps.vec.clone());
        let transitions = transitions(&process);
        assert_eq!(transitions, hashmap! { tau() => ps.vec });
    }
}
