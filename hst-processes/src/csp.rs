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
use crate::external_choice::ExternalChoice;
use crate::internal_choice::InternalChoice;
use crate::prefix::Prefix;
use crate::primitives::Skip;
use crate::primitives::Stop;
use crate::primitives::Tau;
use crate::primitives::Tick;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct CSP<E, TauProof, TickProof>(Rc<CSPInner<E, TauProof, TickProof>>);

impl<E, TauProof, TickProof> Display for CSP<E, TauProof, TickProof>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (&self.0 as &dyn Display).fmt(f)
    }
}

impl<E, TauProof, TickProof> Debug for CSP<E, TauProof, TickProof>
where
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (&self.0 as &dyn Debug).fmt(f)
    }
}

impl<E, TauProof, TickProof> CSP<E, TauProof, TickProof> {
    /// Constructs a new _external choice_ process `P □ Q`.  This process behaves either like `P`
    /// _or_ `Q`, and the environment gets to choose — the process is willing to do either.
    pub fn external_choice(p: Self, q: Self) -> Self {
        CSP(Rc::new(CSPInner::ExternalChoice(ExternalChoice::new(
            vec![p, q],
        ))))
    }

    /// Constructs a new _external choice_ process `P ⊓ Q`.  This process behaves either like `P`
    /// _or_ `Q`, but the environment has no control over which one is chosen.
    pub fn internal_choice(p: Self, q: Self) -> Self {
        CSP(Rc::new(CSPInner::InternalChoice(InternalChoice::new(
            vec![p, q],
        ))))
    }

    /// Constructs a new _prefix_ process `{a} → P`.  This process performs any event in `a` and
    /// then behaves like process `P`.
    pub fn prefix(initials: E, after: Self) -> Self {
        CSP(Rc::new(CSPInner::Prefix(Prefix::new(initials, after))))
    }

    /// Constructs a new _replicated external choice_ process `□ Ps` over a non-empty collection of
    /// processes.  The process behaves like one of the processes in the set, but the environment
    /// has no control over which one is chosen.
    pub fn replicated_external_choice<I>(ps: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        CSP(Rc::new(CSPInner::ExternalChoice(ExternalChoice::new(
            ps.into_iter().collect(),
        ))))
    }

    /// Constructs a new _replicated internal choice_ process `⊓ Ps` over a non-empty collection of
    /// processes.  The process behaves like one of the processes in the set, but the environment
    /// has no control over which one is chosen.
    ///
    /// Panics if `ps` is empty.
    pub fn replicated_internal_choice<I>(ps: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        CSP(Rc::new(CSPInner::InternalChoice(InternalChoice::new(
            ps.into_iter().collect(),
        ))))
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

impl<E, TauProof, TickProof> CSP<E, TauProof, TickProof>
where
    E: Clone + EventSet + Tau<TauProof> + Tick<TickProof>,
    TauProof: Clone,
    TickProof: Clone,
{
    pub fn initials(&self) -> E {
        self.0.initials()
    }

    pub fn transitions(
        &self,
        events: &E,
    ) -> impl Iterator<Item = (E, CSP<E, TauProof, TickProof>)> + '_ {
        self.0.transitions(events)
    }
}

#[derive(Eq, Hash, PartialEq)]
enum CSPInner<E, TauProof, TickProof> {
    ExternalChoice(ExternalChoice<E, TauProof, TickProof>),
    InternalChoice(InternalChoice<E, TauProof, TickProof>),
    Prefix(Prefix<E, TauProof, TickProof>),
    Skip(Skip<E, TickProof>),
    Stop(Stop<E>),
}

impl<E, TauProof, TickProof> Display for CSPInner<E, TauProof, TickProof>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CSPInner::ExternalChoice(this) => (this as &dyn Display).fmt(f),
            CSPInner::InternalChoice(this) => (this as &dyn Display).fmt(f),
            CSPInner::Prefix(this) => (this as &dyn Display).fmt(f),
            CSPInner::Skip(this) => (this as &dyn Display).fmt(f),
            CSPInner::Stop(this) => (this as &dyn Display).fmt(f),
        }
    }
}

impl<E, TauProof, TickProof> Debug for CSPInner<E, TauProof, TickProof>
where
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CSPInner::ExternalChoice(this) => (this as &dyn Debug).fmt(f),
            CSPInner::InternalChoice(this) => (this as &dyn Debug).fmt(f),
            CSPInner::Prefix(this) => (this as &dyn Debug).fmt(f),
            CSPInner::Skip(this) => (this as &dyn Debug).fmt(f),
            CSPInner::Stop(this) => (this as &dyn Debug).fmt(f),
        }
    }
}

impl<E, TauProof, TickProof> CSPInner<E, TauProof, TickProof>
where
    E: Clone + EventSet + Tau<TauProof> + Tick<TickProof>,
    TauProof: Clone,
    TickProof: Clone,
{
    fn initials(&self) -> E {
        match self {
            CSPInner::ExternalChoice(this) => this.initials(),
            CSPInner::InternalChoice(this) => this.initials(),
            CSPInner::Prefix(this) => this.initials(),
            CSPInner::Skip(this) => this.initials(),
            CSPInner::Stop(this) => this.initials(),
        }
    }

    fn transitions(
        &self,
        events: &E,
    ) -> Box<dyn Iterator<Item = (E, CSP<E, TauProof, TickProof>)> + '_> {
        match self {
            CSPInner::ExternalChoice(this) => Box::new(this.transitions(events)),
            CSPInner::InternalChoice(this) => Box::new(this.transitions(events)),
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

    impl<E, TauProof, TickProof> Arbitrary for CSP<E, TauProof, TickProof>
    where
        E: Clone + Debug + Display + NameableEvents + Tau<TauProof> + Tick<TickProof> + 'static,
        E::Strategy: Strategy<Value = E>,
        TauProof: Clone + 'static,
        TickProof: Clone + 'static,
    {
        type Parameters = ();
        type Strategy = BoxedStrategy<CSP<E, TauProof, TickProof>>;

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
