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

//! Defines a process type that includes all of the CSP language.

use std::fmt::Debug;
use std::fmt::Display;
use std::rc::Rc;

use crate::event::EventSet;
use crate::prefix::Prefix;
use crate::primitives::Skip;
use crate::primitives::Stop;
use crate::primitives::Tick;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct CSP<E, TickProof>(Rc<CSPInner<E, TickProof>>);

impl<E, TickProof> Display for CSP<E, TickProof>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (&self.0 as &dyn Display).fmt(f)
    }
}

impl<E, TickProof> Debug for CSP<E, TickProof>
where
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (&self.0 as &dyn Debug).fmt(f)
    }
}

impl<E, TickProof> CSP<E, TickProof> {
    /// Constructs a new _prefix_ process `{a} → P`.  This process performs any event in `a` and
    /// then behaves like process `P`.
    pub fn prefix(initials: E, after: Self) -> Self {
        CSP(Rc::new(CSPInner::Prefix(Prefix::new(initials, after))))
    }

    /// Constructs a new _Skip_ process.  The process that performs ✔ and then becomes _Stop_.
    /// Used to indicate the end of a process that can be sequentially composed with something
    /// else.
    pub fn skip() -> Self {
        CSP(Rc::new(CSPInner::Skip(Skip::new())))
    }

    /// Constructs a new _Stop_ process.  This is the process that performs no actions (and
    /// prevents any other synchronized processes from performing any, either).
    pub fn stop() -> Self {
        CSP(Rc::new(CSPInner::Stop(Stop::new())))
    }
}

impl<E, TickProof> CSP<E, TickProof>
where
    E: Clone + EventSet + Tick<TickProof>,
    TickProof: Clone,
{
    pub fn initials(&self) -> E {
        self.0.initials()
    }

    pub fn transitions(&self, events: &E) -> impl Iterator<Item = (E, CSP<E, TickProof>)> + '_ {
        self.0.transitions(events)
    }
}

#[derive(Eq, Hash, PartialEq)]
enum CSPInner<E, TickProof> {
    Prefix(Prefix<E, TickProof>),
    Skip(Skip<E, TickProof>),
    Stop(Stop<E>),
}

impl<E, TickProof> Display for CSPInner<E, TickProof>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CSPInner::Prefix(this) => (this as &dyn Display).fmt(f),
            CSPInner::Skip(this) => (this as &dyn Display).fmt(f),
            CSPInner::Stop(this) => (this as &dyn Display).fmt(f),
        }
    }
}

impl<E, TickProof> Debug for CSPInner<E, TickProof>
where
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CSPInner::Prefix(this) => (this as &dyn Debug).fmt(f),
            CSPInner::Skip(this) => (this as &dyn Debug).fmt(f),
            CSPInner::Stop(this) => (this as &dyn Debug).fmt(f),
        }
    }
}

impl<E, TickProof> CSPInner<E, TickProof>
where
    E: Clone + EventSet + Tick<TickProof>,
    TickProof: Clone,
{
    fn initials(&self) -> E {
        match self {
            CSPInner::Prefix(this) => this.initials(),
            CSPInner::Skip(this) => this.initials(),
            CSPInner::Stop(this) => this.initials(),
        }
    }

    fn transitions(&self, events: &E) -> Box<dyn Iterator<Item = (E, CSP<E, TickProof>)> + '_> {
        match self {
            CSPInner::Prefix(this) => Box::new(this.transitions(events)),
            CSPInner::Skip(this) => Box::new(this.transitions(events)),
            CSPInner::Stop(this) => Box::new(this.transitions(events)),
        }
    }
}

#[cfg(test)]
mod proptest_support {
    use super::*;

    use proptest::arbitrary::any;
    use proptest::arbitrary::Arbitrary;
    use proptest::prop_oneof;
    use proptest::strategy::BoxedStrategy;
    use proptest::strategy::Just;
    use proptest::strategy::MapInto;
    use proptest::strategy::Strategy;

    use crate::test_support::NumberedEvent;
    use crate::test_support::TestEvents;

    pub trait NameableEvents {
        type Strategy: Strategy;
        fn nameable_events() -> Self::Strategy;
    }

    impl NameableEvents for TestEvents {
        type Strategy = MapInto<<NumberedEvent as Arbitrary>::Strategy, TestEvents>;
        fn nameable_events() -> Self::Strategy {
            any::<NumberedEvent>().prop_map_into()
        }
    }

    impl<E, TickProof> Arbitrary for CSP<E, TickProof>
    where
        E: Clone + Debug + Display + NameableEvents + Tick<TickProof> + 'static,
        E::Strategy: Strategy<Value = E>,
        TickProof: Clone + 'static,
    {
        type Parameters = ();
        type Strategy = BoxedStrategy<CSP<E, TickProof>>;

        fn arbitrary_with(_args: ()) -> Self::Strategy {
            let leaf = prop_oneof![Just(CSP::stop()), Just(CSP::skip())];
            let basic = leaf.prop_recursive(8, 16, 2, move |inner| {
                // We use NumberedEvent here because you shouldn't really create processes that
                // explicitly refer to Tau and Tick; those should only be created as part of
                // the CSP operators.
                (E::nameable_events(), inner.clone())
                    .prop_map(|(initials, after)| CSP::prefix(initials.into(), after))
            });
            basic.boxed()
        }
    }
}
