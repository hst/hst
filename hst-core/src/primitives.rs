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

//! Defines primitive CSP events and processes.

use std::fmt::Debug;
use std::fmt::Display;
use std::marker::PhantomData;

use auto_enums::enum_derive;

use crate::event::Alphabet;
use crate::event::EmptyAlphabet;
use crate::process::Cursor;
use crate::process::Process;

//-------------------------------------------------------------------------------------------------
// Built-in CSP events

/// Constructs a new _tau_ event (τ).  This is the hidden event that expresses nondeterminism in a
/// CSP process.  You should rarely have to construct it directly, unless you're digging through
/// the transitions of a process.
pub fn tau<E: From<Tau>>() -> E {
    Tau.into()
}

#[doc(hidden)]
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Tau;

impl Display for Tau {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("τ")
    }
}

impl Debug for Tau {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &dyn Display).fmt(f)
    }
}

/// Constructs a new _tick_ event (✔).  This is the hidden event that represents the end of a
/// process that can be sequentially composed with another process.  You should rarely have to
/// construct it directly, unless you're digging through the transitions of a process.
pub fn tick<E: From<Tick>>() -> E {
    Tick.into()
}

#[doc(hidden)]
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Tick;

impl Display for Tick {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("✔")
    }
}

impl Debug for Tick {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &dyn Display).fmt(f)
    }
}

//-------------------------------------------------------------------------------------------------
// Stop

/// Constructs a new _Stop_ process.  This is the process that performs no actions (and prevents
/// any other synchronized processes from performing any, either).
pub fn stop<E, P: From<Stop<E>>>() -> P {
    Stop(PhantomData).into()
}

#[doc(hidden)]
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Stop<E>(PhantomData<E>);

impl<E> Display for Stop<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Stop")
    }
}

impl<E> Debug for Stop<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &dyn Display).fmt(f)
    }
}

#[doc(hidden)]
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct StopCursor<E>(PhantomData<E>);

impl<E> Debug for StopCursor<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("StopCursor")
    }
}

impl<E> Process<E> for Stop<E>
where
    E: Display + 'static,
{
    type Cursor = StopCursor<E>;

    fn root(&self) -> Self::Cursor {
        StopCursor(PhantomData)
    }
}

impl<E> Cursor<E> for StopCursor<E>
where
    E: Display + 'static,
{
    type Alphabet = EmptyAlphabet<E>;

    fn initials(&self) -> EmptyAlphabet<E> {
        EmptyAlphabet::new()
    }

    fn perform(&mut self, event: &E) {
        panic!("Stop cannot perform {}", event);
    }
}

#[cfg(test)]
mod stop_tests {
    use super::*;

    use maplit::hashset;
    use proptest_attr_macro::proptest;

    use crate::process::maximal_finite_traces;
    use crate::test_support::TestEvent;

    #[proptest]
    fn check_stop_initials(event: TestEvent) {
        let process: Stop<TestEvent> = dbg!(stop());
        let alphabet = process.root().initials();
        assert!(!alphabet.contains(&event));
    }

    #[test]
    fn check_stop_traces() {
        let process: Stop<Tau> = dbg!(stop());
        assert_eq!(maximal_finite_traces(process.root()), hashset! {vec![]});
    }
}

//-------------------------------------------------------------------------------------------------
// Skip

/// Constructs a new _Skip_ process.  The process that performs Tick and then becomes Stop.  Used
/// to indicate the end of a process that can be sequentially composed with something else.
pub fn skip<E, P: From<Skip<E>>>() -> P {
    Skip(PhantomData).into()
}

#[doc(hidden)]
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Skip<E>(PhantomData<E>);

impl<E> Display for Skip<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Skip")
    }
}

impl<E> Debug for Skip<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &dyn Display).fmt(f)
    }
}

#[doc(hidden)]
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct SkipCursor<E> {
    state: SkipState,
    phantom: PhantomData<E>,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SkipState {
    BeforeTick,
    AfterTick,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SkipAlphabet<E> {
    BeforeTick(PhantomData<E>),
    AfterTick,
}

impl<E> Debug for SkipCursor<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SkipCursor({:?})", self.state)
    }
}

impl<E> Process<E> for Skip<E>
where
    E: Display + Eq + From<Tick> + 'static,
{
    type Cursor = SkipCursor<E>;

    fn root(&self) -> Self::Cursor {
        SkipCursor {
            state: SkipState::BeforeTick,
            phantom: PhantomData,
        }
    }
}

impl<E> Cursor<E> for SkipCursor<E>
where
    E: Display + Eq + From<Tick> + 'static,
{
    type Alphabet = SkipAlphabet<E>;

    fn initials(&self) -> SkipAlphabet<E> {
        match self.state {
            SkipState::BeforeTick => SkipAlphabet::BeforeTick(PhantomData),
            SkipState::AfterTick => SkipAlphabet::AfterTick,
        }
    }

    fn perform(&mut self, event: &E) {
        if self.state == SkipState::AfterTick {
            panic!("Skip cannot perform {} after ✔", event);
        }
        if *event != tick() {
            panic!("Skip cannot perform {}", event);
        }
        self.state = SkipState::AfterTick;
    }
}

impl<E> Alphabet<E> for SkipAlphabet<E>
where
    E: Eq + From<Tick>,
{
    fn contains(&self, event: &E) -> bool {
        match self {
            SkipAlphabet::BeforeTick(_) => *event == tick(),
            SkipAlphabet::AfterTick => false,
        }
    }
}

#[doc(hidden)]
#[enum_derive(Iterator)]
pub enum SkipAlphabetIterator<E> {
    BeforeTick(std::iter::Once<E>),
    AfterTick(std::iter::Empty<E>),
}

impl<E> IntoIterator for SkipAlphabet<E>
where
    E: From<Tick>,
{
    type Item = E;
    type IntoIter = SkipAlphabetIterator<E>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            SkipAlphabet::BeforeTick(_) => {
                SkipAlphabetIterator::BeforeTick(std::iter::once(tick()))
            }
            SkipAlphabet::AfterTick => SkipAlphabetIterator::AfterTick(std::iter::empty()),
        }
    }
}

#[cfg(test)]
mod skip_tests {
    use super::*;

    use maplit::hashset;
    use proptest_attr_macro::proptest;

    use crate::process::maximal_finite_traces;
    use crate::test_support::TestEvent;

    #[proptest]
    fn check_skip_initials(event: TestEvent) {
        let process: Skip<TestEvent> = dbg!(skip());

        let alphabet = process.root().initials();
        assert_eq!(alphabet.contains(&event), event == tick());

        let alphabet = process.root().after(&tick()).initials();
        assert!(!alphabet.contains(&event));
    }

    #[test]
    fn check_skip_traces() {
        let process: Skip<TestEvent> = dbg!(skip());
        assert_eq!(
            maximal_finite_traces(process.root()),
            hashset! { vec![tick()] }
        );
    }
}
