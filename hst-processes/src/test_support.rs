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

use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Display;

use bit_array::BitArray;
use proptest::arbitrary::any;
use proptest::arbitrary::Arbitrary;
use proptest::collection::hash_set;
use proptest::strategy::BoxedStrategy;
use proptest::strategy::Strategy;

use crate::event::DisjointSum;
use crate::event::EventSet;
use crate::primitives::PrimitiveEvents;

/// An event that is identified by a number.  Makes it easy to construct distinct events in
/// test cases.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct NumberedEvent(pub u16);

impl From<u16> for NumberedEvent {
    fn from(from: u16) -> NumberedEvent {
        NumberedEvent(from)
    }
}

const SUBSCRIPT_DIGITS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];

impl Debug for NumberedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &dyn Display).fmt(f)
    }
}

impl Display for NumberedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let digits: String = self
            .0
            .to_string()
            .chars()
            .map(|ch| SUBSCRIPT_DIGITS[ch.to_digit(10).unwrap() as usize])
            .collect();
        write!(f, "E{}", digits)
    }
}

impl Arbitrary for NumberedEvent {
    type Parameters = ();
    type Strategy = BoxedStrategy<NumberedEvent>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        any::<u16>().prop_map_into().boxed()
    }
}

#[test]
fn can_display_events() {
    assert_eq!(NumberedEvent(0).to_string(), "E₀");
    assert_eq!(NumberedEvent(10).to_string(), "E₁₀");
    assert_eq!(NumberedEvent(01234).to_string(), "E₁₂₃₄");
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct NumberedEvents(BitArray<usize, typenum::U65536>);

impl NumberedEvents {
    pub fn add(&mut self, event: NumberedEvent) {
        let index = event.0 as usize;
        self.0.set(index, true);
    }

    pub fn contains(&self, event: NumberedEvent) -> bool {
        let index = event.0 as usize;
        self.0[index]
    }
}

impl Display for NumberedEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_set()
            .entries(
                self.0
                    .iter()
                    .enumerate()
                    .filter(|(_index, value)| *value)
                    .map(|(index, _value)| index as u16)
                    .map(NumberedEvent::from),
            )
            .finish()
    }
}

impl Debug for NumberedEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "NumberedEvents ")?;
        f.debug_set()
            .entries(
                self.0
                    .iter()
                    .enumerate()
                    .filter(|(_index, value)| *value)
                    .map(|(index, _value)| index as u16)
                    .map(NumberedEvent::from),
            )
            .finish()
    }
}

impl From<NumberedEvent> for NumberedEvents {
    fn from(event: NumberedEvent) -> NumberedEvents {
        let mut events = NumberedEvents::empty();
        events.add(event);
        events
    }
}

impl From<HashSet<NumberedEvent>> for NumberedEvents {
    fn from(set: HashSet<NumberedEvent>) -> NumberedEvents {
        let mut events = NumberedEvents::empty();
        for event in set {
            events.add(event);
        }
        events
    }
}

impl EventSet for NumberedEvents {
    fn empty() -> Self {
        NumberedEvents(BitArray::from_elem(false))
    }

    fn intersect(&mut self, other: &Self) {
        self.0.intersect(&other.0);
    }

    fn is_empty(&self) -> bool {
        self.0.none()
    }

    fn negate(&mut self) {
        self.0.negate();
    }

    fn subtract(&mut self, other: &Self) {
        self.0.difference(&other.0);
    }

    fn union(&mut self, other: &Self) {
        self.0.union(&other.0);
    }

    fn universe() -> Self {
        NumberedEvents(BitArray::from_elem(true))
    }
}

impl IntoIterator for NumberedEvents {
    type Item = NumberedEvents;
    type IntoIter = Box<dyn Iterator<Item = NumberedEvents>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(
            self.0
                .into_iter()
                .enumerate()
                .filter(|(_index, value)| *value)
                .map(|(index, _value)| index as u16)
                .map(NumberedEvent::from)
                .map(NumberedEvents::from),
        )
    }
}

impl Arbitrary for NumberedEvents {
    type Parameters = ();
    type Strategy = BoxedStrategy<NumberedEvents>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        hash_set(any::<NumberedEvent>(), 0..32)
            .prop_map_into()
            .boxed()
    }
}

#[cfg(test)]
mod numbered_events_tests {
    use proptest_attr_macro::proptest;

    use super::*;

    #[proptest]
    fn can_intersect(a: NumberedEvents, b: NumberedEvents, event: NumberedEvent) {
        let mut intersection = a.clone();
        intersection.intersect(&b);
        assert_eq!(
            intersection.contains(event),
            a.contains(event) && b.contains(event)
        );
    }

    #[proptest]
    fn intersection_is_commutative(a: NumberedEvents, b: NumberedEvents) {
        let mut i1 = a.clone();
        i1.intersect(&b);
        let mut i2 = b.clone();
        i2.intersect(&a);
        assert_eq!(i1, i2);
    }

    #[proptest]
    fn can_negate(a: NumberedEvents, event: NumberedEvent) {
        let mut negation = a.clone();
        negation.negate();
        assert_eq!(negation.contains(event), !a.contains(event));
    }

    #[proptest]
    fn negation_is_reversible(a: NumberedEvents) {
        let mut negated_twice = a.clone();
        negated_twice.negate();
        negated_twice.negate();
        assert_eq!(a, negated_twice);
    }

    #[proptest]
    fn can_subtract(a: NumberedEvents, b: NumberedEvents, event: NumberedEvent) {
        let mut difference = a.clone();
        difference.subtract(&b);
        assert_eq!(
            difference.contains(event),
            a.contains(event) && !b.contains(event)
        );
    }

    #[proptest]
    fn can_union(a: NumberedEvents, b: NumberedEvents, event: NumberedEvent) {
        let mut union = a.clone();
        union.union(&b);
        assert_eq!(
            union.contains(event),
            a.contains(event) || b.contains(event)
        );
    }

    #[proptest]
    fn union_is_commutative(a: NumberedEvents, b: NumberedEvents) {
        let mut u1 = a.clone();
        u1.union(&b);
        let mut u2 = b.clone();
        u2.union(&a);
        assert_eq!(u1, u2);
    }
}

/// An event type that is useful in test cases.  It can be a NumberedEvent or any of the
/// built-in event types.
pub type TestEvents = DisjointSum<PrimitiveEvents, NumberedEvents>;

impl From<NumberedEvent> for TestEvents {
    fn from(event: NumberedEvent) -> TestEvents {
        TestEvents::from_b(event.into())
    }
}
