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
use std::fmt::Display;

use auto_enums::enum_derive;
use auto_from::From;
use proptest::arbitrary::any;
use proptest::arbitrary::Arbitrary;
use proptest::strategy::BoxedStrategy;
use proptest::strategy::Strategy;

use crate::primitives::Tau;
use crate::primitives::Tick;

/// An event that is identified by a number.  Makes it easy to construct distinct events in
/// test cases.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct NumberedEvent(pub u32);

impl From<u32> for NumberedEvent {
    fn from(from: u32) -> NumberedEvent {
        NumberedEvent(from)
    }
}

const SUBSCRIPT_DIGITS: [char; 10] = [
    '₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉',
];

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

impl Debug for NumberedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &Display).fmt(f)
    }
}

impl Arbitrary for NumberedEvent {
    type Parameters = ();
    type Strategy = BoxedStrategy<NumberedEvent>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        any::<u32>().prop_map_into().boxed()
    }
}

#[test]
fn can_display_events() {
    assert_eq!(NumberedEvent(0).to_string(), "E₀");
    assert_eq!(NumberedEvent(10).to_string(), "E₁₀");
    assert_eq!(
        NumberedEvent(0123456789).to_string(),
        "E₁₂₃₄₅₆₇₈₉"
    );
}

/// An event type that is useful in test cases.  It can be a NumberedEvent or any of the
/// built-in event types.
#[enum_derive(Debug, Display)]
#[derive(Clone, Eq, From, Hash, PartialEq)]
pub enum TestEvent {
    Tau(Tau),
    Tick(Tick),
    NumberedEvent(NumberedEvent),
}

impl Arbitrary for TestEvent {
    type Parameters = ();
    type Strategy = BoxedStrategy<TestEvent>;

    fn arbitrary_with(_args: ()) -> Self::Strategy {
        // Note that we don't generate Tau or Tick as values for TestEvent; those built-in events
        // should never be mentioned by name in processes, and should only come about because of
        // the operational semantics of the CSP operators.
        any::<NumberedEvent>().prop_map_into().boxed()
    }
}