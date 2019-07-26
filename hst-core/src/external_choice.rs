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

//! Defines the external choice (□) operator.

use std::fmt::Debug;
use std::fmt::Display;

use smallvec::smallvec;
use smallvec::SmallVec;

use crate::primitives::tau;
use crate::primitives::Tau;
use crate::process::Afters;
use crate::process::Initials;

/// Constructs a new _external choice_ process `P □ Q`.  This process behaves either like `P` _or_
/// `Q`, and the environment gets to choose — the process is willing to do either.
pub fn external_choice<P: From<ExternalChoice<P>>>(p: P, q: P) -> P {
    ExternalChoice(smallvec![p, q]).into()
}

/// Constructs a new _replicated external choice_ process `□ Ps` over a non-empty collection of
/// processes.  The process behaves like one of the processes in the set, but the environment has
/// no control over which one is chosen.
///
/// Panics if `ps` is empty.
pub fn replicated_external_choice<P: From<ExternalChoice<P>>, I: IntoIterator<Item = P>>(
    ps: I,
) -> P {
    let ps: SmallVec<[P; 2]> = ps.into_iter().collect();
    ExternalChoice(ps).into()
}

/// The type of an [`external_choice`] process.
///
/// [`external_choice`]: fn.external_choice.html
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ExternalChoice<P>(SmallVec<[P; 2]>);

impl<P: Debug + Display> Display for ExternalChoice<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.0.len() == 2 {
            write!(f, "{} □ {}", self.0[0], self.0[1])
        } else {
            f.write_str("□ ")?;
            f.debug_set().entries(&self.0).finish()
        }
    }
}

impl<P: Debug + Display> Debug for ExternalChoice<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &dyn Display).fmt(f)
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

impl<'a, E, P> Initials<'a, E> for ExternalChoice<P>
where
    E: 'a,
    P: Initials<'a, E>,
{
    // Need the box since we can't name the type that it contains :-(
    type Initials = Box<dyn Iterator<Item = E> + 'a>;

    fn initials(&'a self) -> Self::Initials {
        // 1) If P ∈ Ps can perform τ, then □ Ps can perform τ.
        // 2) If P ∈ Ps can perform a ≠ τ, then □ Ps can perform a ≠ τ.
        //
        // initials(□ Ps) = ⋃ { initials(P) ∩ {τ} | P ∈ Ps }                [rule 1]
        //                ∪ ⋃ { initials(P) ∖ {τ} | P ∈ Ps }                [rule 2]
        //
        //                = ⋃ { initials(P) | P ∈ Ps }
        Box::new(self.0.iter().flat_map(P::initials))
    }
}

impl<'a, E, P> Afters<'a, E, P> for ExternalChoice<P>
where
    E: Clone + Eq + From<Tau> + 'a,
    P: Clone + From<ExternalChoice<P>> + 'a,
    P: Afters<'a, E, P>,
{
    type Afters = Box<dyn Iterator<Item = P> + 'a>;

    fn afters(&'a self, initial: &E) -> Self::Afters {
        // afters(□ Ps, τ) = ⋃ { □ Ps ∖ {P} ∪ {P'} | P ∈ Ps, P' ∈ afters(P, τ) }
        //                                                                  [rule 1]
        // afters(□ Ps, a ≠ τ) = ⋃ { P' | P ∈ Ps, P' ∈ afters(P, a) }       [rule 2]
        if *initial == tau() {
            // An iterator of (idx, P) pairs: each process that we're □-ing over, along with its
            // index in our Ps collection.
            let enumerated = self.0.iter().enumerate();
            // Expands each P into its after processes, but still paired with the corresponding P's
            // original index in Ps.
            let afters = enumerated.flat_map({
                let initial = initial.clone();
                move |(idx, p)| p.afters(&initial).map(move |p1| (idx, p1))
            });
            // For each (idx, P') element, use the idx to replace P in Ps with P'.
            let mut ps = self.0.clone();
            let replaced = afters.map(move |(idx, mut p1)| {
                std::mem::swap(&mut ps[idx], &mut p1);
                let result = ExternalChoice(ps.clone()).into();
                std::mem::swap(&mut ps[idx], &mut p1);
                result
            });

            Box::new(replaced)
        } else {
            let initial = initial.clone();
            Box::new(self.0.iter().flat_map(move |p| p.afters(&initial)))
        }
    }
}

#[cfg(test)]
mod external_choice_tests {
    use super::*;

    use std::collections::HashMap;

    use maplit::hashmap;
    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::internal_choice::internal_choice;
    use crate::prefix::prefix;
    use crate::process::transitions;
    use crate::test_support::NumberedEvent;
    use crate::test_support::TestEvent;

    #[test]
    fn check_empty_external_choice() {
        let process = replicated_external_choice(vec![]);
        let transitions: HashMap<TestEvent, Vec<CSP<TestEvent>>> = transitions(&process);
        assert!(transitions.is_empty());
    }

    #[proptest]
    fn check_singleton_external_choice(a: NumberedEvent, p: CSP<TestEvent>) {
        let a = TestEvent::from(a);
        let process = replicated_external_choice(vec![prefix(a.clone(), p.clone())]);
        let transitions = transitions(&process);
        assert_eq!(transitions, hashmap! { a => vec![p] });
    }

    #[proptest]
    fn check_doubleton_external_choice(a: NumberedEvent, p: CSP<TestEvent>, q: CSP<TestEvent>) {
        let a = TestEvent::from(a);
        let process = replicated_external_choice(vec![
            prefix(a.clone(), p.clone()),
            prefix(a.clone(), q.clone()),
        ]);
        let transitions = transitions(&process);
        assert_eq!(transitions, hashmap! { a => vec![p, q] });
    }

    #[proptest]
    fn check_external_internal(
        a: NumberedEvent,
        b: NumberedEvent,
        c: NumberedEvent,
        p: CSP<TestEvent>,
        q: CSP<TestEvent>,
        r: CSP<TestEvent>,
    ) {
        // process = a → P □ (b → Q ⊓ c → R)
        let a = TestEvent::from(a);
        let b = TestEvent::from(b);
        let c = TestEvent::from(c);
        let prefix_p = prefix(a.clone(), p.clone());
        let prefix_q = prefix(b.clone(), q.clone());
        let prefix_r = prefix(c.clone(), r.clone());
        let process = external_choice(
            prefix_p.clone(),
            internal_choice(prefix_q.clone(), prefix_r.clone()),
        );
        let transitions = transitions(&process);
        assert_eq!(
            transitions,
            hashmap! {
                // A tau resolves the internal choice
                tau() => vec![
                    external_choice(prefix_p.clone(), prefix_q.clone()),
                    external_choice(prefix_p.clone(), prefix_r.clone()),
                ],
                // If the environment chooses `a`, then the internal choice doesn't matter.
                a => vec![p],
            }
        );
    }
}
