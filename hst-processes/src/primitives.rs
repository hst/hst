// -*- coding: utf-8 -*-
// ------------------------------------------------------------------------------------------------
// Copyright © 2020, HST authors.
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

//! Defines primitive CSP events and processes.

use std::fmt::Debug;
use std::fmt::Display;
use std::marker::PhantomData;

use crate::csp::CSP;
use crate::event::DisjointSum;
use crate::event::EventSet;
use crate::event::Here;
use crate::event::There;

//-------------------------------------------------------------------------------------------------
// Built-in CSP events

/// The _tau_ event (τ).  This is the hidden event that expresses nondeterminism in a CSP process.
/// You should rarely have to construct it directly, unless you're digging through the transitions
/// of a process.
pub trait Tau<Proof> {
    fn tau() -> Self;
    fn can_perform_tau(&self) -> bool;
}

/// The _tick_ event (✔).  This is the hidden event that represents the end of a process that can
/// be sequentially composed with another process.  You should rarely have to construct it
/// directly, unless you're digging through the transitions of a process.
pub trait Tick<Proof> {
    fn tick() -> Self;
    fn can_perform_tick(&self) -> bool;
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct PrimitiveEvents {
    contains_tau: bool,
    contains_tick: bool,
}

impl Debug for PrimitiveEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match (self.contains_tau, self.contains_tick) {
            (true, true) => write!(f, "PrimitiveEvents {{τ,✔}}"),
            (true, false) => write!(f, "PrimitiveEvents {{τ}}"),
            (false, true) => write!(f, "PrimitiveEvents {{✔}}"),
            (false, false) => write!(f, "PrimitiveEvents {{}}"),
        }
    }
}

impl Display for PrimitiveEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match (self.contains_tau, self.contains_tick) {
            (true, true) => write!(f, "{{τ,✔}}"),
            (true, false) => write!(f, "{{τ}}"),
            (false, true) => write!(f, "{{✔}}"),
            (false, false) => write!(f, "{{}}"),
        }
    }
}

impl EventSet for PrimitiveEvents {
    fn empty() -> Self {
        PrimitiveEvents {
            contains_tau: false,
            contains_tick: false,
        }
    }

    fn intersect(&mut self, other: &Self) {
        self.contains_tau &= other.contains_tau;
        self.contains_tick &= other.contains_tick;
    }

    fn is_empty(&self) -> bool {
        !self.contains_tau && !self.contains_tick
    }

    fn negate(&mut self) {
        self.contains_tau = !self.contains_tau;
        self.contains_tick = !self.contains_tick;
    }

    fn subtract(&mut self, other: &Self) {
        self.contains_tau &= !other.contains_tau;
        self.contains_tick &= !other.contains_tick;
    }

    fn union(&mut self, other: &Self) {
        self.contains_tau |= other.contains_tau;
        self.contains_tick |= other.contains_tick;
    }

    fn universe() -> Self {
        PrimitiveEvents {
            contains_tau: true,
            contains_tick: true,
        }
    }
}

#[derive(Eq, PartialEq)]
enum PrimitiveEventsIteratorState {
    Tau,
    Tick,
    Done,
}

pub struct PrimitiveEventsIterator {
    state: PrimitiveEventsIteratorState,
    source: PrimitiveEvents,
}

impl PrimitiveEventsIterator {
    fn get_tau(&mut self) -> Option<PrimitiveEvents> {
        self.state = PrimitiveEventsIteratorState::Tick;
        if !self.source.contains_tau {
            return None;
        }
        Some(PrimitiveEvents {
            contains_tau: true,
            contains_tick: false,
        })
    }

    fn get_tick(&mut self) -> Option<PrimitiveEvents> {
        self.state = PrimitiveEventsIteratorState::Done;
        if !self.source.contains_tick {
            return None;
        }
        Some(PrimitiveEvents {
            contains_tau: false,
            contains_tick: true,
        })
    }
}

impl Iterator for PrimitiveEventsIterator {
    type Item = PrimitiveEvents;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == PrimitiveEventsIteratorState::Tau {
            if let Some(result) = self.get_tau() {
                return Some(result);
            }
        }

        if self.state == PrimitiveEventsIteratorState::Tick {
            if let Some(result) = self.get_tick() {
                return Some(result);
            }
        }

        None
    }
}

impl IntoIterator for PrimitiveEvents {
    type Item = PrimitiveEvents;
    type IntoIter = PrimitiveEventsIterator;

    fn into_iter(self) -> Self::IntoIter {
        PrimitiveEventsIterator {
            state: PrimitiveEventsIteratorState::Tau,
            source: self,
        }
    }
}

impl Tau<()> for PrimitiveEvents {
    fn tau() -> Self {
        PrimitiveEvents {
            contains_tau: true,
            contains_tick: false,
        }
    }

    fn can_perform_tau(&self) -> bool {
        self.contains_tau
    }
}

impl<E, Tail> Tau<Here> for DisjointSum<E, Tail>
where
    E: Tau<()>,
    Tail: EventSet,
{
    fn tau() -> Self {
        DisjointSum::from_a(E::tau())
    }

    fn can_perform_tau(&self) -> bool {
        self.0.can_perform_tau()
    }
}

impl<Head, Tail, TailIndex> Tau<There<TailIndex>> for DisjointSum<Head, Tail>
where
    Head: EventSet,
    Tail: Tau<TailIndex>,
{
    fn tau() -> Self {
        DisjointSum::from_b(Tail::tau())
    }

    fn can_perform_tau(&self) -> bool {
        self.1.can_perform_tau()
    }
}

impl Tick<()> for PrimitiveEvents {
    fn tick() -> Self {
        PrimitiveEvents {
            contains_tau: false,
            contains_tick: true,
        }
    }

    fn can_perform_tick(&self) -> bool {
        self.contains_tick
    }
}

impl<E, Tail> Tick<Here> for DisjointSum<E, Tail>
where
    E: Tick<()>,
    Tail: EventSet,
{
    fn tick() -> Self {
        DisjointSum::from_a(E::tick())
    }

    fn can_perform_tick(&self) -> bool {
        self.0.can_perform_tick()
    }
}

impl<Head, Tail, TailIndex> Tick<There<TailIndex>> for DisjointSum<Head, Tail>
where
    Head: EventSet,
    Tail: Tick<TailIndex>,
{
    fn tick() -> Self {
        DisjointSum::from_b(Tail::tick())
    }

    fn can_perform_tick(&self) -> bool {
        self.1.can_perform_tick()
    }
}

#[cfg(test)]
mod primitive_events_tests {
    use super::*;

    use proptest_attr_macro::proptest;

    use crate::test_support::NumberedEvents;
    use crate::test_support::TestEvents;

    #[test]
    fn can_check_for_tau() {
        assert!(!PrimitiveEvents::empty().can_perform_tau());
        assert!(PrimitiveEvents::tau().can_perform_tau());
        assert!(PrimitiveEvents::universe().can_perform_tau());
    }

    #[test]
    fn can_check_sum_for_tau() {
        assert!(!TestEvents::empty().can_perform_tau());
        assert!(TestEvents::tau().can_perform_tau());
        assert!(TestEvents::universe().can_perform_tau());
    }

    #[proptest]
    fn rest_of_sum_does_not_affect_tau(rest: NumberedEvents) {
        let mut whole = TestEvents::tau();
        whole.union(&TestEvents::from_b(rest));
        assert!(whole.can_perform_tau());
    }

    #[test]
    fn can_check_for_tick() {
        assert!(!PrimitiveEvents::empty().can_perform_tick());
        assert!(PrimitiveEvents::tick().can_perform_tick());
        assert!(PrimitiveEvents::universe().can_perform_tick());
    }

    #[test]
    fn can_check_sum_for_tick() {
        assert!(!TestEvents::empty().can_perform_tick());
        assert!(TestEvents::tick().can_perform_tick());
        assert!(TestEvents::universe().can_perform_tick());
    }

    #[proptest]
    fn rest_of_sum_does_not_affect_tick(rest: NumberedEvents) {
        let mut whole = TestEvents::tick();
        whole.union(&TestEvents::from_b(rest));
        assert!(whole.can_perform_tick());
    }

    #[test]
    fn can_enumerate() {
        let collect = |events: PrimitiveEvents| events.into_iter().collect::<Vec<_>>();
        assert_eq!(collect(PrimitiveEvents::empty()), vec![]);
        assert_eq!(
            collect(PrimitiveEvents::tau()),
            vec![PrimitiveEvents::tau()]
        );
        assert_eq!(
            collect(PrimitiveEvents::tick()),
            vec![PrimitiveEvents::tick()]
        );
        assert_eq!(
            collect(PrimitiveEvents::universe()),
            vec![PrimitiveEvents::tau(), PrimitiveEvents::tick()]
        );
    }

    #[test]
    fn can_enumerate_sum() {
        let collect = |events: TestEvents| events.into_iter().collect::<Vec<_>>();
        assert_eq!(collect(TestEvents::empty()), vec![]);
        assert_eq!(collect(TestEvents::tau()), vec![TestEvents::tau()]);
        assert_eq!(collect(TestEvents::tick()), vec![TestEvents::tick()]);
        assert_eq!(
            // Can't use TestEvents::universe here, because that would include all 2^32 possible
            // NumberedEvents, too.
            collect(TestEvents::from_a(PrimitiveEvents::universe())),
            vec![TestEvents::tau(), TestEvents::tick()]
        );
    }
}

//-------------------------------------------------------------------------------------------------
// Stop

#[derive(Clone, Eq, Hash, PartialEq)]
pub(crate) struct Stop<E, P>(PhantomData<E>, PhantomData<P>);

impl<E, P> Stop<E, P> {
    pub(crate) fn new() -> Stop<E, P> {
        Stop(PhantomData, PhantomData)
    }
}

impl<E, P> Display for Stop<E, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Stop")
    }
}

impl<E, P> Debug for Stop<E, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &dyn Display).fmt(f)
    }
}

impl<E, P> Stop<E, P>
where
    E: EventSet,
{
    pub(crate) fn initials(&self) -> E {
        E::empty()
    }

    pub(crate) fn transitions(&self, _events: &E) -> impl Iterator<Item = (E, CSP<E>)> {
        std::iter::empty()
    }
}

#[cfg(test)]
mod stop_tests {
    use super::*;

    use maplit::hashset;

    use crate::maximal_traces::maximal_finite_traces;
    use crate::test_support::TestEvents;

    #[test]
    fn check_stop_initials() {
        let process = CSP::<TestEvents>::stop();
        assert_eq!(process.initials(), TestEvents::empty());
    }

    #[test]
    fn check_stop_traces() {
        let process = CSP::<TestEvents>::stop();
        assert_eq!(maximal_finite_traces(&process), hashset! {vec![]});
    }
}
