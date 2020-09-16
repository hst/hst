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

use generational_arena::Arena;
use generational_arena::Index;

use crate::event::EventSet;
use crate::external_choice::ExternalChoice;
use crate::internal_choice::InternalChoice;
use crate::prefix::Prefix;
use crate::primitives::Skip;
use crate::primitives::Stop;
use crate::primitives::Tau;
use crate::primitives::Tick;
use crate::sequential_composition::SequentialComposition;

pub struct CSP<E, TauProof, TickProof> {
    arena: Arena<CSPInner<E, TauProof, TickProof>>,
}

#[derive(Clone)]
pub struct Process<'p, E, TauProof, TickProof> {
    parent: &'p CSP<E, TauProof, TickProof>,
    index: Index,
}

impl<'p, E, TauProof, TickProof> Process<'p, E, TauProof, TickProof> {
    fn inner(&self) -> &CSPInner<E, TauProof, TickProof> {
        self.parent.arena[self.index]
    }
}

impl<'p, E, TauProof, TickProof> Display for Process<'p, E, TauProof, TickProof>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self.inner() as &dyn Display).fmt(f)
    }
}

impl<'p, E, TauProof, TickProof> Debug for Process<'p, E, TauProof, TickProof>
where
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self.inner() as &dyn Debug).fmt(f)
    }
}

impl<E, TauProof, TickProof> CSP<E, TauProof, TickProof> {
    fn new_process<'p>(
        &'p self,
        inner: CSPInner<E, TauProof, TickProof>,
    ) -> Process<'p, E, TauProof, TickProof> {
        let index = self.arena.add(inner);
        Process {
            inner: Rc::new(ProcessInner {
                parent: self,
                index,
            }),
        }
    }

    /// Constructs a new _external choice_ process `P □ Q`.  This process behaves either like `P`
    /// _or_ `Q`, and the environment gets to choose — the process is willing to do either.
    pub fn external_choice<'p>(
        &'p self,
        p: Process<'p, E, TauProof, TickProof>,
        q: Process<'p, E, TauProof, TickProof>,
    ) -> Process<'p, E, TauProof, TickProof> {
        self.new_process(CSPInner::ExternalChoice(ExternalChoice::new(vec![p, q])))
    }

    /// Constructs a new _external choice_ process `P ⊓ Q`.  This process behaves either like `P`
    /// _or_ `Q`, but the environment has no control over which one is chosen.
    pub fn internal_choice<'p>(
        &'p self,
        p: Process<'p, E, TauProof, TickProof>,
        q: Process<'p, E, TauProof, TickProof>,
    ) -> Process<'p, E, TauProof, TickProof> {
        self.new_process(CSPInner::InternalChoice(InternalChoice::new(vec![p, q])))
    }

    /// Constructs a new _prefix_ process `{a} → P`.  This process performs any event in `a` and
    /// then behaves like process `P`.
    pub fn prefix<'p>(
        &'p self,
        initials: E,
        after: Process<'p, E, TauProof, TickProof>,
    ) -> Process<'p, E, TauProof, TickProof> {
        self.new_process(CSPInner::Prefix(Prefix::new(initials, after)))
    }

    /// Constructs a new _replicated external choice_ process `□ Ps` over a non-empty collection of
    /// processes.  The process behaves like one of the processes in the set, but the environment
    /// has no control over which one is chosen.
    pub fn replicated_external_choice<'p, I>(&'p self, ps: I) -> Process<'p, E, TauProof, TickProof>
    where
        I: IntoIterator<Item = Process<'p, E, TauProof, TickProof>>,
    {
        self.new_process(CSPInner::ExternalChoice(ExternalChoice::new(
            ps.into_iter().collect(),
        )))
    }

    /// Constructs a new _replicated internal choice_ process `⊓ Ps` over a non-empty collection of
    /// processes.  The process behaves like one of the processes in the set, but the environment
    /// has no control over which one is chosen.
    ///
    /// Panics if `ps` is empty.
    pub fn replicated_internal_choice<'p, I>(&'p self, ps: I) -> Process<'p, E, TauProof, TickProof>
    where
        I: IntoIterator<Item = Process<'p, E, TauProof, TickProof>>,
    {
        self.new_process(CSPInner::InternalChoice(InternalChoice::new(
            ps.into_iter().collect(),
        )))
    }

    /// Constructs a new _sequential composition_ process `P ; Q`.  This process behaves like
    /// process `P` until it performs a ✔ event, after which is behaves like process `Q`.
    pub fn sequential_composition<'p>(
        &'p self,
        p: Process<'p, E, TauProof, TickProof>,
        q: Process<'p, E, TauProof, TickProof>,
    ) -> Process<'p, E, TauProof, TickProof> {
        self.new_process(CSPInner::SequentialComposition(SequentialComposition::new(
            p, q,
        )))
    }

    /// Constructs a new _Skip_ process.  The process that performs ✔ and then becomes _Stop_.
    /// Used to indicate the end of a process that can be sequentially composed with something
    /// else.
    pub fn skip<'p>(&'p self) -> Process<'p, E, TauProof, TickProof> {
        self.new_process(CSPInner::Skip(Skip::new()))
    }

    /// Constructs a new _Stop_ process.  This is the process that performs no actions (and
    /// prevents any other synchronized processes from performing any, either).
    pub fn stop<'p>(&'p self) -> Process<'p, E, TauProof, TickProof> {
        self.new_process(CSPInner::Stop(Stop::new()))
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
    SequentialComposition(SequentialComposition<E, TauProof, TickProof>),
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
            CSPInner::SequentialComposition(this) => (this as &dyn Display).fmt(f),
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
            CSPInner::SequentialComposition(this) => (this as &dyn Debug).fmt(f),
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
            CSPInner::SequentialComposition(this) => this.initials(),
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
            CSPInner::SequentialComposition(this) => Box::new(this.transitions(events)),
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
