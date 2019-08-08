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

//! Defines the external choice (`□`) operator.

use std::fmt::Debug;
use std::fmt::Display;

use smallvec::smallvec;
use smallvec::SmallVec;

use crate::possibilities::Possibilities;
use crate::primitives::tau;
use crate::primitives::Tau;
use crate::process::Cursor;
use crate::process::Process;

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

#[doc(hidden)]
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

#[doc(hidden)]
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ExternalChoiceCursor<E, C> {
    state: ExternalChoiceState,
    possibilities: Possibilities<E, C>,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ExternalChoiceState {
    Unresolved,
    Resolved,
}

impl<E, C> Debug for ExternalChoiceCursor<E, C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ExternalChoiceCursor")
            .field("state", &self.state)
            .field("subcursors", &self.possibilities)
            .finish()
    }
}

impl<E, P> Process<E> for ExternalChoice<P>
where
    E: Display + Eq + From<Tau> + 'static,
    P: Process<E>,
    P::Cursor: Clone,
{
    type Cursor = ExternalChoiceCursor<E, P::Cursor>;

    fn root(&self) -> Self::Cursor {
        ExternalChoiceCursor {
            state: ExternalChoiceState::Unresolved,
            possibilities: Possibilities::new(self.0.iter().map(P::root)),
        }
    }
}

impl<E, C> Cursor<E> for ExternalChoiceCursor<E, C>
where
    E: Display + Eq + From<Tau>,
    C: Clone + Cursor<E>,
{
    fn events<'a>(&'a self) -> Box<dyn Iterator<Item = E> + 'a> {
        // An external choice can always perform any event that any of its still-activated
        // subprocesses can perform, regardless of whether the choice has been resolved yet.
        Box::new(self.possibilities.events())
    }

    fn can_perform(&self, event: &E) -> bool {
        self.possibilities.can_perform(event)
    }

    fn perform(&mut self, event: &E) {
        if self.state == ExternalChoiceState::Resolved {
            // If the choice has been resolved, pass the event on to any subprocesses that are
            // still activated.
            self.possibilities.perform_all(event);
            return;
        }

        // If the choice has _not_ been resolved, and it's a τ, allow each subprocess to
        // _independently_ perform the event.
        if *event == tau() {
            self.possibilities.perform_piecewise(event);
            return;
        }

        // Otherwise, the event resolves the choice!  If there is more than one subprocess that can
        // perform the event, allow them all to perform it simultaneously.
        self.possibilities.perform_all(event);
        self.state = ExternalChoiceState::Resolved;
    }
}

#[cfg(test)]
mod external_choice_tests {
    use super::*;

    use maplit::hashset;
    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::process::initials;
    use crate::process::maximal_finite_traces;
    use crate::test_support::TestEvent;

    #[test]
    fn check_empty_external_choice() {
        let process: CSP<TestEvent> = replicated_external_choice(vec![]);
        assert_eq!(maximal_finite_traces(process.root()), hashset! {vec![]});
    }

    #[proptest]
    fn check_singleton_external_choice(p: CSP<TestEvent>) {
        let process = replicated_external_choice(vec![p.clone()]);
        assert_eq!(initials(&process.root()), initials(&p.root()));
        assert_eq!(
            maximal_finite_traces(process.root()),
            maximal_finite_traces(p.root())
        );
    }

    #[proptest]
    fn check_doubleton_external_choice(p: CSP<TestEvent>, q: CSP<TestEvent>) {
        let process = replicated_external_choice(vec![p.clone(), q.clone()]);
        assert_eq!(
            initials(&process.root()),
            &initials(&p.root()) | &initials(&q.root())
        );
        assert_eq!(
            maximal_finite_traces(process.root()),
            maximal_finite_traces(p.root()) + maximal_finite_traces(q.root())
        );
    }

    #[proptest]
    fn check_tripleton_external_choice(p: CSP<TestEvent>, q: CSP<TestEvent>, r: CSP<TestEvent>) {
        let process = replicated_external_choice(vec![p.clone(), q.clone(), r.clone()]);
        assert_eq!(
            initials(&process.root()),
            &(&initials(&p.root()) | &initials(&q.root())) | &initials(&r.root())
        );
        assert_eq!(
            maximal_finite_traces(process.root()),
            maximal_finite_traces(p.root())
                + maximal_finite_traces(q.root())
                + maximal_finite_traces(r.root())
        );
    }
}
