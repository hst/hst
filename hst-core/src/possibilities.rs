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

use std::fmt::Debug;
use std::marker::PhantomData;

use smallbitvec::SmallBitVec;

use crate::process::Cursor;

/// A set of possible current states for a process, where each current state is defined by the
/// current states of some subprocesses.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Possibilities<E, C> {
    phantom: PhantomData<E>,
    subcursors: Vec<C>,
    activated: SmallBitVec,
    possibilities: Vec<Possibility>,
    next_possibilities: Vec<Possibility>,
}

impl<E, C> Possibilities<E, C> {
    pub fn new<I>(subcursors: I) -> Possibilities<E, C>
    where
        I: IntoIterator<Item = C>,
    {
        let subcursors = subcursors.into_iter().collect::<Vec<_>>();
        let subcursor_count = subcursors.len();
        let possibility = (0..subcursor_count).collect();
        Possibilities {
            phantom: PhantomData,
            subcursors,
            activated: SmallBitVec::from_elem(subcursor_count, true),
            possibilities: vec![possibility],
            next_possibilities: Vec::new(),
        }
    }
}

impl<E, C> Debug for Possibilities<E, C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_list().entries(self.activated_subcursors()).finish()
    }
}

#[cfg(test)]
mod test_support {
    use super::*;

    use std::collections::HashSet;

    use maplit::hashset;

    impl<E, C> Possibilities<E, C>
    where
        C: Clone,
    {
        pub fn possibilities<R>(&self) -> R
        where
            R: std::iter::FromIterator<Vec<C>>,
        {
            self.possibilities
                .iter()
                .map(|possibility| {
                    possibility
                        .iter()
                        .filter(|idx| self.activated[**idx])
                        .map(|idx| self.subcursors[*idx].clone())
                        .collect()
                })
                .collect()
        }
    }

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    pub struct Event;

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    pub enum TestCursor {
        Before1,
        After1,
        Before2,
        After2,
    }

    impl Cursor<Event> for TestCursor {
        fn events<'a>(&'a self) -> Box<dyn Iterator<Item = Event> + 'a> {
            match self {
                TestCursor::Before1 | TestCursor::Before2 => Box::new(std::iter::once(Event)),
                TestCursor::After1 | TestCursor::After2 => Box::new(std::iter::empty()),
            }
        }

        fn can_perform(&self, _event: &Event) -> bool {
            match self {
                TestCursor::Before1 | TestCursor::Before2 => true,
                TestCursor::After1 | TestCursor::After2 => false,
            }
        }

        fn perform(&mut self, _event: &Event) {
            match self {
                TestCursor::Before1 => *self = TestCursor::After1,
                TestCursor::Before2 => *self = TestCursor::After2,
                _ => panic!("Cannot perform event"),
            }
        }
    }

    impl Possibilities<Event, TestCursor> {
        pub fn verify_cannot_perform_event(&self) {
            assert_eq!(self.events().collect::<HashSet<_>>(), hashset![]);
            assert!(!self.can_perform(&Event));
        }

        pub fn verify_can_perform_event(&self) {
            assert_eq!(self.events().collect::<HashSet<_>>(), hashset![Event]);
            assert!(self.can_perform(&Event));
        }
    }
}

/// Each possible current state is represented by the indices of one or more `subcursors`.
type Possibility = Vec<usize>;

impl<E, C> Possibilities<E, C> {
    /// Returns an iterator of the subcursors that are still activated.
    pub fn activated_subcursors<'a>(&'a self) -> impl Iterator<Item = &C> + 'a {
        self.activated
            .iter()
            .zip(&self.subcursors)
            .filter(|(activated, _)| *activated)
            .map(|(_, subcursor)| subcursor)
    }
}

impl<E, C> Possibilities<E, C>
where
    C: Cursor<E>,
{
    /// Returns an iterator of all of the events that _any_ subprocess can perform in _any_
    /// possible current state.
    pub fn events<'a>(&'a self) -> impl Iterator<Item = E> + 'a {
        self.activated_subcursors().flat_map(C::events)
    }

    /// Returns whether _any_ subprocess in _any_ possible current state can perform `event`.
    pub fn can_perform(&self, event: &E) -> bool {
        self.activated_subcursors()
            .any(|subcursor| subcursor.can_perform(event))
    }
}

impl<E, C> Possibilities<E, C>
where
    C: Clone + Cursor<E>,
{
    /// Tries to perform `event` in each possible current state.  Any possible current states that
    /// _can't_ perform the event are deactivated.
    ///
    /// Within each possible current state, the individual subprocesses try to perform the event
    /// independently.  If more than one subprocess can, then that possibility is "split" into
    /// multiple possibilities, one for each subprocess that can perform the event.
    pub fn perform_piecewise(&mut self, event: &E) {
        let subcursor_count = self.subcursors.len();
        let possibility_count = self.possibilities.len();

        // First find all of the still-active subprocesses that can perform the event.
        let mut eligible = SmallBitVec::from_elem(subcursor_count, false);
        for idx in 0..subcursor_count {
            if self.activated[idx] {
                if self.subcursors[idx].can_perform(event) {
                    unsafe { eligible.set_unchecked(idx, true) };
                }
            }
        }

        // For each possible current state, count how many of its subprocesses can perform the
        // event.
        let eligible_per_possibility = self
            .possibilities
            .iter()
            .map(|possibility| {
                possibility
                    .iter()
                    .filter(|subprocess| eligible[**subprocess])
                    .count()
            })
            .collect::<Vec<_>>();

        // If a possibility has more than one subprocess that can perform the event, we call that a
        // "splittable" possibility.  If an eligible subprocess appears in any splittable
        // possibility, then the subprocess is splittable.
        let mut splittable = SmallBitVec::from_elem(subcursor_count, false);
        for idx in 0..possibility_count {
            if eligible_per_possibility[idx] > 1 {
                for subprocess in &self.possibilities[idx] {
                    if eligible[*subprocess] {
                        unsafe { splittable.set_unchecked(*subprocess, true) };
                    }
                }
            }
        }

        // Allow each eligible subprocess to perform the event.  For the splittable ones, we have
        // to clone the corresponding cursor, so that we can keep track of the subprocess before
        // and after the event is performed.
        let mut eligible_afters = self.subcursors.iter().map(|_| 0usize).collect::<Vec<_>>();
        for idx in 0..subcursor_count {
            if !eligible[idx] {
                continue;
            }

            // If the subprocess is splittable, we need to make sure that the _new_ subcursor is
            // the one that performs the event.  Within each splittable possibility, we want to
            // update _exactly one_ of its eligible subprocesses at a time, leaving all of the
            // others in their before state.  That's easier to do if the possibility starts off
            // with its eligible processes in their before state, which means that we need the
            // _existing_ subprocesses to remain in their before state.
            if splittable[idx] {
                let mut after = self.subcursors[idx].clone();
                after.perform(event);
                eligible_afters[idx] = self.subcursors.len();
                self.subcursors.push(after);
                self.activated.push(true);
                continue;
            }

            // If the subprocess is _not_ splittable, then we can go ahead and have it perform the
            // event directly.  It's guaranteed to exist only in non-splittable possibilities, and
            // so we don't need to keep its before state around.  We do need to add an entry in
            // `eligible_afters` for this subprocess, so that we do the right thing down below when
            // we edit the contents of each non-splittable possibility.
            self.subcursors[idx].perform(event);
            eligible_afters[idx] = idx;
        }

        // Jeez, now we can finally go update all of the possibilities.  We accumulate the new set
        // of possibilities into a separate field (yay double buffering).
        for (idx, possibility) in self.possibilities.drain(..).enumerate() {
            if eligible_per_possibility[idx] == 0 {
                // This possibility can't perform the event at all, so it's no longer a valid
                // possibility!
                continue;
            }

            // This logic should work regardless of whether the process is splittable or not.
            //
            // If it's splittable, the existing possibility entry currently contains the before
            // state for each eligible subprocess.  For each of those eligible subprocesses, we
            // create a new copy of the possibility with exactly one of them updated to the
            // corresponding after state.
            //
            // If it's not splittable, then it contains exactly one eligible subprocess.  If that
            // subprocess is splittable, then it also appears in some other splittable possibility.
            // The current possibility contains its before state, and our loop will create a copy
            // where it's updated to the after state.  (Since there's only one eligible subprocess,
            // the meat of the loop will only execute once!)
            //
            // If the possibility and subprocess are both non-splittable, then we've already
            // updated that subcursor in-place to have performed the event.  But because we made
            // sure to still fill in `eligible_afters` for the subprocess, we'll end up creating a
            // copy of the possibility with the subprocess pointing at the same subcursor (which,
            // as mentioned, is now in its after state).  Maybe a bit more copying than we need,
            // but it works!
            for (subprocess_idx, subprocess) in possibility.iter().enumerate() {
                if !eligible[*subprocess] {
                    continue;
                }

                let mut new_possibility = possibility.clone();
                new_possibility[subprocess_idx] = eligible_afters[*subprocess];
                self.next_possibilities.push(new_possibility);
            }
        }

        // We built up the new possibilities into a separate field, so swap them into place.
        std::mem::swap(&mut self.possibilities, &mut self.next_possibilities);
    }
}

#[cfg(test)]
mod perform_piecewise_tests {
    use super::test_support::*;
    use super::*;

    use std::fmt::Debug;
    use std::iter::FromIterator;

    use maplit::hashset;

    impl Possibilities<Event, TestCursor> {
        fn perform_piecewise_and_verify<R>(&mut self, expected: R)
        where
            R: Debug + Eq + FromIterator<Vec<TestCursor>>,
        {
            self.perform_piecewise(&Event);
            assert_eq!(self.possibilities::<R>(), expected);
        }
    }

    #[test]
    fn check_empty() {
        let possibilities = Possibilities::new(vec![]);
        possibilities.verify_cannot_perform_event();
    }

    #[test]
    fn check_one_before() {
        let mut possibilities = Possibilities::new(vec![TestCursor::Before1]);
        possibilities.verify_can_perform_event();
        possibilities.perform_piecewise_and_verify(hashset![vec![TestCursor::After1]]);
        // After performing the event, we shouldn't be able to perform it anymore.
        possibilities.verify_cannot_perform_event();
    }

    #[test]
    fn check_one_after() {
        let possibilities = Possibilities::new(vec![TestCursor::After1]);
        possibilities.verify_cannot_perform_event();
    }

    #[test]
    fn check_two_befores() {
        let mut possibilities = Possibilities::new(vec![TestCursor::Before1, TestCursor::Before2]);
        possibilities.verify_can_perform_event();
        possibilities.perform_piecewise_and_verify(hashset![
            vec![TestCursor::After1, TestCursor::Before2],
            vec![TestCursor::Before1, TestCursor::After2]
        ]);

        // We can still perform the event!  One of the subprocesses went first; now the other one
        // can go.
        possibilities.verify_can_perform_event();
        possibilities.perform_piecewise_and_verify(vec![
            // The after possibility appears twice, once for each ordering of subprocesses.
            // We're not clever enough to detect that they're the same and de-dup them.
            vec![TestCursor::After1, TestCursor::After2],
            vec![TestCursor::After1, TestCursor::After2],
        ]);

        // After performing the event twice, we shouldn't be able to perform it anymore.
        possibilities.verify_cannot_perform_event();
    }

    #[test]
    fn check_two_afters() {
        let possibilities = Possibilities::new(vec![TestCursor::After1, TestCursor::After2]);
        possibilities.verify_cannot_perform_event();
    }

    #[test]
    fn check_one_of_each() {
        let mut possibilities = Possibilities::new(vec![TestCursor::After1, TestCursor::Before2]);
        possibilities.verify_can_perform_event();
        possibilities
            .perform_piecewise_and_verify(hashset![vec![TestCursor::After1, TestCursor::After2]]);
        // After performing the event, we shouldn't be able to perform it anymore.
        possibilities.verify_cannot_perform_event();
    }
}

impl<E, C> Possibilities<E, C>
where
    C: Clone + Cursor<E>,
{
    /// Tries to have each subprocess perform `event`.  Any subprocesses that can't perform the
    /// event are deactivated.
    pub fn perform_all(&mut self, event: &E) {
        let subcursor_count = self.subcursors.len();
        for idx in 0..subcursor_count {
            if self.activated[idx] {
                if self.subcursors[idx].can_perform(event) {
                    self.subcursors[idx].perform(event);
                } else {
                    unsafe { self.activated.set_unchecked(idx, false) };
                }
            }
        }
    }
}

#[cfg(test)]
mod perform_all_tests {
    use super::test_support::*;
    use super::*;

    use std::fmt::Debug;
    use std::iter::FromIterator;

    use maplit::hashset;

    impl Possibilities<Event, TestCursor> {
        fn perform_all_and_verify<R>(&mut self, expected: R)
        where
            R: Debug + Eq + FromIterator<Vec<TestCursor>>,
        {
            self.perform_all(&Event);
            assert_eq!(self.possibilities::<R>(), expected);
        }
    }

    #[test]
    fn check_empty() {
        let possibilities = Possibilities::new(vec![]);
        possibilities.verify_cannot_perform_event();
    }

    #[test]
    fn check_one_before() {
        let mut possibilities = Possibilities::new(vec![TestCursor::Before1]);
        possibilities.verify_can_perform_event();
        possibilities.perform_all_and_verify(hashset![vec![TestCursor::After1]]);
        // After performing the event, we shouldn't be able to perform it anymore.
        possibilities.verify_cannot_perform_event();
    }

    #[test]
    fn check_one_after() {
        let possibilities = Possibilities::new(vec![TestCursor::After1]);
        possibilities.verify_cannot_perform_event();
    }

    #[test]
    fn check_two_befores() {
        let mut possibilities = Possibilities::new(vec![TestCursor::Before1, TestCursor::Before2]);
        possibilities.verify_can_perform_event();
        possibilities
            .perform_all_and_verify(hashset![vec![TestCursor::After1, TestCursor::After2]]);

        // After performing the event twice, we shouldn't be able to perform it anymore.
        possibilities.verify_cannot_perform_event();
    }

    #[test]
    fn check_two_afters() {
        let possibilities = Possibilities::new(vec![TestCursor::After1, TestCursor::After2]);
        possibilities.verify_cannot_perform_event();
    }

    #[test]
    fn check_one_of_each() {
        let mut possibilities = Possibilities::new(vec![TestCursor::After1, TestCursor::Before2]);
        possibilities.verify_can_perform_event();
        possibilities.perform_all_and_verify(hashset![vec![TestCursor::After2]]);
        // After performing the event, we shouldn't be able to perform it anymore.
        possibilities.verify_cannot_perform_event();
    }
}
