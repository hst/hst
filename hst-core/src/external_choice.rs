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
use std::marker::PhantomData;

use smallvec::smallvec;
use smallvec::SmallVec;

use crate::primitives::tau;
use crate::primitives::Tau;
use crate::process::Afters;
use crate::process::Cursor;
use crate::process::Initials;
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

#[doc(hidden)]
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ExternalChoiceCursor<E, C> {
    phantom: PhantomData<E>,
    state: ExternalChoiceState,
    /// The cursors for the choice's subprocesses.  For each one, we keep a vector of cursors, with
    /// each element describing the state after an increasingly large number of τ's have been
    /// performed.  (If a subprocess can't perform any τ's, then there will only ever be a single
    /// subcursor in the corresponding entry of the outer vector.)
    ///
    /// After the choice has been resolved by a visible event, we use the Option part to keep track
    /// of which of the possible choices are able to perform the sequence of events that have
    /// occurred since the choice was resolved.  Subprocesses that aren't able to perform the full
    /// sequence are "eliminated", and retroactively couldn't have been a possible resolution to
    /// the original choice.
    subcursors: Vec<Vec<Option<C>>>,
    /// The number of τ events that have been performed before the choice has been resolved.
    tau_count: usize,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ExternalChoiceState {
    Unresolved,
    Resolved,
}

struct Subcursors<'a, C>(&'a Vec<Vec<Option<C>>>);
struct SubcursorTaus<'a, C>(&'a Vec<Option<C>>);
struct SubcursorTau<'a, C>(&'a Option<C>);

impl<E, C> Debug for ExternalChoiceCursor<E, C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ExternalChoiceCursor")
            .field("state", &self.state)
            .field("tau_count", &self.tau_count)
            .field("subcursors", &Subcursors(&self.subcursors))
            .finish()
    }
}

impl<'a, C> Debug for Subcursors<'a, C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(SubcursorTaus))
            .finish()
    }
}

impl<'a, C> Debug for SubcursorTaus<'a, C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(SubcursorTau).enumerate())
            .finish()
    }
}

impl<'a, C> Debug for SubcursorTau<'a, C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            None => f.write_str("None"),
            Some(subcursor) => subcursor.fmt(f),
        }
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
            phantom: PhantomData,
            state: ExternalChoiceState::Unresolved,
            subcursors: self
                .0
                .iter()
                .map(P::root)
                .map(Some)
                .map(|c| vec![c])
                .collect(),
            tau_count: 0,
        }
    }
}

impl<E, C> ExternalChoiceCursor<E, C> {
    /// Deactivates subprocess states based on the number of τ's that they could have performed
    /// so far, compared to the number of τ's that the choice actually have been performed so far.
    ///
    /// More specifically: Each subprocess will have some number of τ's that it could have
    /// performed, up to a maximum of self.tau_count.  (The subprocess might be willing to perform
    /// more, but we're about to resolve the choice before allowing it to.)  It's only possible for
    /// the subprocess to perform `n` τ's if we can find some way for all of the _other_ τ's (i.e.,
    /// `self.tau_count - n` of them) to be performed by the _other_ τ-eligible subprocesses.  If
    /// we can't, then we deactivate that subprocess state.
    fn deactivate_tau_subprocesses(&mut self) {
        // The total number of τ events performed across all subprocesses at this point.  This will
        // always be ≥ self.tau_count.  It will be larger if there were multiple subprocesses that
        // could perform τ's.
        let total_tau_depth: usize = self.subcursors.iter().map(|taus| taus.len() - 1).sum();

        for taus in &mut self.subcursors {
            // If this subprocess couldn't perform _any_ τ's, then it remains active.  (We might be
            // about to deactivate it based on the visible event that resolves that choice, but
            // it's not being deactivated based on τ's.)
            if taus.len() == 1 {
                continue;
            }

            // The number of τ's this subprocess could have performed.
            let this_tau_count = taus.len() - 1;

            // The total number of τ's that other subprocesses could have performed.
            let remaining_tau_depth = total_tau_depth - this_tau_count;

            if remaining_tau_depth < self.tau_count {
                // The _minimum_ number of τ's this subprocess _must_ perform, to ensure that there
                // are enough other subprocess τ's to cover the total.
                let minimum_tau_count = self.tau_count - remaining_tau_depth;
                for subcursor in taus.iter_mut().take(minimum_tau_count) {
                    subcursor.take();
                }
            }
        }
    }

    fn perform_tau_when_unresolved(&mut self, tau: &E)
    where
        C: Clone + Cursor<E>,
    {
        self.tau_count += 1;

        // For any subprocess that has been able to perform however many τ's occurred _before_
        // this one, try to have it perform another!  If it can, stash away a copy of the
        // subcursor describing its state after this τ.
        for taus in &mut self.subcursors {
            // This subprocess already stopped being able to perform τ's.  No point checking it
            // any further!
            if taus.len() != self.tau_count {
                continue;
            }

            // Check whether this subprocess can perform a τ in the current state.
            let before = taus
                .last()
                .expect("Vector should never be empty")
                .as_ref()
                .expect("All subprocesses should be active in unresolved external choice");
            if !before.can_perform(tau) {
                continue;
            }

            // If so, grab the subprocess's new state after this τ.
            let mut after = before.clone();
            after.perform(tau);
            taus.push(Some(after));
        }

        // Deactivate any τ subprocess states that are no longer eligible (because that state has
        // performed too few τ's, and there aren't enough other subprocesses to make up the
        // difference).
        self.deactivate_tau_subprocesses();
    }

    /// Has each subprocess state that's still activated perform an event, if it can.  If it can't,
    /// deactivates that state.  Returns whether there were any subprocesses that could perform the
    /// event.
    fn perform_and_deactivate(&mut self, event: &E) -> bool
    where
        C: Cursor<E>,
    {
        let mut any_performed = false;
        for subcursor in self.subcursors.iter_mut().flatten() {
            match subcursor {
                Some(subcursor) if subcursor.can_perform(event) => {
                    subcursor.perform(event);
                    any_performed = true;
                }
                Some(_) => {
                    subcursor.take();
                }
                _ => {}
            }
        }
        any_performed
    }

    fn perform_visible_when_unresolved(&mut self, event: &E)
    where
        C: Cursor<E>,
    {
        // Perform the visible event to actually resolve the choice.  This might deactivate
        // additional subprocess states, if any of them can't perform this event.
        self.perform_and_deactivate(event);
        self.state = ExternalChoiceState::Resolved;
    }

    fn perform_when_resolved(&mut self, event: &E)
    where
        C: Cursor<E>,
        E: Display,
    {
        if !self.perform_and_deactivate(event) {
            panic!("Resolved external choice cannot perform {}", event);
        }
    }
}

impl<E, C> Cursor<E> for ExternalChoiceCursor<E, C>
where
    E: Display + Eq + From<Tau>,
    C: Clone + Cursor<E>,
{
    fn events<'a>(&'a self) -> Box<dyn Iterator<Item = E> + 'a> {
        // Regardless of whether the choice has been resolved yet, we're able to perform any
        // event allowed by any of the still-eligible possible subprocesses.
        Box::new(
            self.subcursors
                .iter()
                // One flatten merges the Vecs, the other takes care of the Options.
                .flatten()
                .flatten()
                .flat_map(C::events),
        )
    }

    fn can_perform(&self, event: &E) -> bool {
        self.subcursors
            .iter()
            // One flatten merges the Vecs, the other takes care of the Options.
            .flatten()
            .flatten()
            .any(|subcursor| subcursor.can_perform(event))
    }

    fn perform(&mut self, event: &E) {
        match self.state {
            ExternalChoiceState::Unresolved => {
                if *event == tau() {
                    self.perform_tau_when_unresolved(event);
                } else {
                    self.perform_visible_when_unresolved(event);
                }
            }
            ExternalChoiceState::Resolved => {
                self.perform_when_resolved(event);
            }
        }
    }
}

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
