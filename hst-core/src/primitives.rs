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

use crate::process::Afters;
use crate::process::Initials;

//-------------------------------------------------------------------------------------------------
// Built-in CSP events

/// Constructs a new _tau_ event (τ).  This is the hidden event that expresses nondeterminism in a
/// CSP process.  You should rarely have to construct it directly, unless you're digging through
/// the transitions of a process.
pub fn tau<E: From<Tau>>() -> E {
    Tau.into()
}

/// The type of the [`tau`] event.
///
/// [`tau`]: fn.tau.html
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Tau;

impl Display for Tau {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("τ")
    }
}

impl Debug for Tau {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &Display).fmt(f)
    }
}

/// Constructs a new _tick_ event (✔).  This is the hidden event that represents the end of a
/// process that can be sequentially composed with another process.  You should rarely have to
/// construct it directly, unless you're digging through the transitions of a process.
pub fn tick<E: From<Tick>>() -> E {
    Tick.into()
}

/// The type of the [`tick`] event.
///
/// [`tick`]: fn.tau.html
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Tick;

impl Display for Tick {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("✔")
    }
}

impl Debug for Tick {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &Display).fmt(f)
    }
}

//-------------------------------------------------------------------------------------------------
// Stop

/// Constructs a new _Stop_ process.  This is the process that performs no actions (and prevents
/// any other synchronized processes from performing any, either).
pub fn stop<P: From<Stop>>() -> P {
    Stop.into()
}

/// The type of the [`Stop`] process.
///
/// [`stop`]: fn.stop.html
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Stop;

impl Display for Stop {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Stop")
    }
}

impl Debug for Stop {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &Display).fmt(f)
    }
}

#[doc(hidden)]
pub struct StopInitials<E>(PhantomData<E>);

impl<E> IntoIterator for StopInitials<E> {
    type Item = E;
    type IntoIter = std::iter::Empty<E>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::empty()
    }
}

impl<E> Initials<E> for Stop {
    type Initials = StopInitials<E>;

    fn initials(&self) -> Self::Initials {
        StopInitials(PhantomData)
    }
}

#[doc(hidden)]
pub struct StopAfters<P>(PhantomData<P>);

impl<P> IntoIterator for StopAfters<P> {
    type Item = P;
    type IntoIter = std::iter::Empty<P>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::empty()
    }
}

impl<E, P> Afters<E, P> for Stop {
    type Afters = StopAfters<P>;

    fn afters(&self, _initial: &E) -> Option<Self::Afters> {
        None
    }
}

#[cfg(test)]
mod stop_tests {
    use super::*;

    use std::collections::HashMap;

    use crate::csp::CSP;
    use crate::process::transitions;
    use crate::test_support::TestEvent;

    #[test]
    fn check_stop_transitions() {
        let transitions: HashMap<TestEvent, Vec<CSP<TestEvent>>> = transitions(&stop());
        assert!(transitions.is_empty());
    }
}

//-------------------------------------------------------------------------------------------------
// Skip

/// Constructs a new _Skip_ process.  The process that performs Tick and then becomes Stop.  Used
/// to indicate the end of a process that can be sequentially composed with something else.
pub fn skip<P: From<Skip>>() -> P {
    Skip.into()
}

/// The type of the [`Skip`] process.
///
/// [`skip`]: fn.stop.html
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Skip;

impl Display for Skip {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Skip")
    }
}

impl Debug for Skip {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &Display).fmt(f)
    }
}

#[doc(hidden)]
pub struct SkipInitials<E>(PhantomData<E>);

impl<E> IntoIterator for SkipInitials<E>
where
    E: From<Tick>,
{
    type Item = E;
    type IntoIter = std::iter::Once<E>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(tick())
    }
}

impl<E> Initials<E> for Skip
where
    E: From<Tick>,
{
    type Initials = SkipInitials<E>;

    fn initials(&self) -> Self::Initials {
        SkipInitials(PhantomData)
    }
}

#[doc(hidden)]
pub struct SkipAfters<P>(PhantomData<P>);

impl<P> IntoIterator for SkipAfters<P>
where
    P: From<Stop>,
{
    type Item = P;
    type IntoIter = std::iter::Once<P>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(stop())
    }
}

impl<E, P> Afters<E, P> for Skip
where
    E: Eq + From<Tick>,
    P: From<Stop>,
{
    type Afters = SkipAfters<P>;

    fn afters(&self, initial: &E) -> Option<Self::Afters> {
        if *initial == Tick.into() {
            Some(SkipAfters(PhantomData))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod skip_tests {
    use super::*;

    use std::collections::HashMap;

    use maplit::hashmap;

    use crate::csp::CSP;
    use crate::process::transitions;
    use crate::test_support::TestEvent;

    #[test]
    fn check_skip_transitions() {
        let transitions: HashMap<TestEvent, Vec<CSP<TestEvent>>> = transitions(&skip());
        assert_eq!(transitions, hashmap! { tick() => vec![stop()] });
    }
}