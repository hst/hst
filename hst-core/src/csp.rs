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

//! Defines a process type that includes all of the CSP language.

use std::fmt::Debug;
use std::fmt::Display;
use std::rc::Rc;

use auto_enums::enum_derive;
use derive_more::From;

use crate::event::Alphabet;
use crate::external_choice::ExternalChoice;
use crate::internal_choice::InternalChoice;
use crate::prefix::Prefix;
use crate::primitives::Skip;
use crate::primitives::Stop;
use crate::primitives::Tau;
use crate::primitives::Tick;
use crate::process::Cursor;
use crate::process::Process;
use crate::sequential_composition::SequentialComposition;

/// A process type that includes all of the primitive processes and operators in the CSP language.
/// Note that you should never need to construct instances of this type directly; use the helper
/// constructor for each process (e.g. [`stop`]) or operator (e.g. [`prefix`]) instead.
///
/// [`stop`]: ../primitives/fn.stop.html
/// [`prefix`]: ../prefix/fn.prefix.html
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct CSP<E>(Rc<CSPSig<E, CSP<E>>>);

impl<E: Display> Display for CSP<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self.0.as_ref() as &dyn Display).fmt(f)
    }
}

impl<E: Display> Debug for CSP<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self.0.as_ref() as &dyn Debug).fmt(f)
    }
}

impl<E, T> From<T> for CSP<E>
where
    CSPSig<E, CSP<E>>: From<T>,
{
    fn from(t: T) -> CSP<E> {
        CSP(Rc::new(CSPSig::from(t)))
    }
}

impl<E> Process<E> for CSP<E>
where
    E: Clone + Display + Eq + From<Tau> + From<Tick> + 'static,
{
    type Cursor = CSPCursor<E>;

    fn root(&self) -> Self::Cursor {
        CSPCursor(Box::new(self.0.root()))
    }
}

#[doc(hidden)]
#[derive(Clone, Eq, PartialEq)]
pub struct CSPCursor<E>(
    Box<
        CSPSigCursor<
            <ExternalChoice<CSP<E>> as Process<E>>::Cursor,
            <InternalChoice<CSP<E>> as Process<E>>::Cursor,
            <Prefix<E, CSP<E>> as Process<E>>::Cursor,
            <SequentialComposition<CSP<E>> as Process<E>>::Cursor,
            <Skip<E> as Process<E>>::Cursor,
            <Stop<E> as Process<E>>::Cursor,
        >,
    >,
)
where
    E: Clone + Display + Eq + From<Tau> + From<Tick> + 'static;

impl<E> Debug for CSPCursor<E>
where
    E: Clone + Debug + Display + Eq + From<Tau> + From<Tick> + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self.0.as_ref() as &dyn Debug).fmt(f)
    }
}

impl<E> Cursor<E> for CSPCursor<E>
where
    E: Clone + Display + Eq + From<Tau> + From<Tick> + 'static,
{
    type Alphabet = CSPAlphabet<E>;

    fn initials(&self) -> Self::Alphabet {
        CSPAlphabet(Box::new(self.0.initials()))
    }

    fn perform(&mut self, event: &E) {
        self.0.perform(event);
    }
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CSPAlphabet<E>(
    Box<
        CSPSigAlphabet<
            <<ExternalChoice<CSP<E>> as Process<E>>::Cursor as Cursor<E>>::Alphabet,
            <<InternalChoice<CSP<E>> as Process<E>>::Cursor as Cursor<E>>::Alphabet,
            <<Prefix<E, CSP<E>> as Process<E>>::Cursor as Cursor<E>>::Alphabet,
            <<SequentialComposition<CSP<E>> as Process<E>>::Cursor as Cursor<E>>::Alphabet,
            <<Skip<E> as Process<E>>::Cursor as Cursor<E>>::Alphabet,
            <<Stop<E> as Process<E>>::Cursor as Cursor<E>>::Alphabet,
        >,
    >,
)
where
    E: Clone + Display + Eq + From<Tau> + From<Tick> + 'static;

impl<E> Alphabet<E> for CSPAlphabet<E>
where
    E: Clone + Display + Eq + From<Tau> + From<Tick> + 'static,
{
    fn contains(&self, event: &E) -> bool {
        self.0.contains(event)
    }
}

#[doc(hidden)]
pub struct CSPAlphabetIterator<E>(
    Box<CSPSigAlphabetIterator<
        <<<ExternalChoice<CSP<E>> as Process<E>>::Cursor as Cursor<E>>::Alphabet as IntoIterator>::IntoIter,
        <<<InternalChoice<CSP<E>> as Process<E>>::Cursor as Cursor<E>>::Alphabet as IntoIterator>::IntoIter,
        <<<Prefix<E, CSP<E>> as Process<E>>::Cursor as Cursor<E>>::Alphabet as IntoIterator>::IntoIter,
        <<<SequentialComposition<CSP<E>> as Process<E>>::Cursor as Cursor<E>>::Alphabet as IntoIterator>::IntoIter,
        <<<Skip<E> as Process<E>>::Cursor as Cursor<E>>::Alphabet as IntoIterator>::IntoIter,
        <<<Stop<E> as Process<E>>::Cursor as Cursor<E>>::Alphabet as IntoIterator>::IntoIter,
    >>
)
where
    E: Clone + Display + Eq + From<Tau> + From<Tick> + 'static;

impl<E> Iterator for CSPAlphabetIterator<E>
where
    E: Clone + Display + Eq + From<Tau> + From<Tick> + 'static,
{
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<E> IntoIterator for CSPAlphabet<E>
where
    E: Clone + Display + Eq + From<Tau> + From<Tick> + 'static,
{
    type Item = E;
    type IntoIter = CSPAlphabetIterator<E>;

    fn into_iter(self) -> Self::IntoIter {
        CSPAlphabetIterator(Box::new(self.0.into_iter()))
    }
}

#[doc(hidden)]
#[enum_derive(Debug, Display)]
#[derive(Clone, Eq, From, Hash, PartialEq)]
pub enum CSPSig<E, P> {
    #[doc(hidden)]
    ExternalChoice(ExternalChoice<P>),
    #[doc(hidden)]
    InternalChoice(InternalChoice<P>),
    #[doc(hidden)]
    Prefix(Prefix<E, P>),
    #[doc(hidden)]
    SequentialComposition(SequentialComposition<P>),
    #[doc(hidden)]
    Skip(Skip<E>),
    #[doc(hidden)]
    Stop(Stop<E>),
}

#[doc(hidden)]
#[enum_derive(Debug, Display)]
#[derive(Clone, Eq, PartialEq)]
pub enum CSPSigCursor<ExternalChoice, InternalChoice, Prefix, SequentialComposition, Skip, Stop> {
    ExternalChoice(ExternalChoice),
    InternalChoice(InternalChoice),
    Prefix(Prefix),
    SequentialComposition(SequentialComposition),
    Skip(Skip),
    Stop(Stop),
}

impl<E, P> Process<E> for CSPSig<E, P>
where
    E: Clone + Display + Eq + From<Tau> + From<Tick> + 'static,
    P: Clone + Process<E>,
    P::Cursor: Clone,
{
    type Cursor = CSPSigCursor<
        <ExternalChoice<P> as Process<E>>::Cursor,
        <InternalChoice<P> as Process<E>>::Cursor,
        <Prefix<E, P> as Process<E>>::Cursor,
        <SequentialComposition<P> as Process<E>>::Cursor,
        <Skip<E> as Process<E>>::Cursor,
        <Stop<E> as Process<E>>::Cursor,
    >;

    fn root(&self) -> Self::Cursor {
        match self {
            CSPSig::ExternalChoice(this) => CSPSigCursor::ExternalChoice(this.root()),
            CSPSig::InternalChoice(this) => CSPSigCursor::InternalChoice(this.root()),
            CSPSig::Prefix(this) => CSPSigCursor::Prefix(this.root()),
            CSPSig::SequentialComposition(this) => CSPSigCursor::SequentialComposition(this.root()),
            CSPSig::Skip(this) => CSPSigCursor::Skip(this.root()),
            CSPSig::Stop(this) => CSPSigCursor::Stop(this.root()),
        }
    }
}

impl<E, ExternalChoice, InternalChoice, Prefix, SequentialComposition, Skip, Stop> Cursor<E>
    for CSPSigCursor<ExternalChoice, InternalChoice, Prefix, SequentialComposition, Skip, Stop>
where
    ExternalChoice: Cursor<E>,
    InternalChoice: Cursor<E>,
    Prefix: Cursor<E>,
    SequentialComposition: Cursor<E>,
    Skip: Cursor<E>,
    Stop: Cursor<E>,
{
    type Alphabet = CSPSigAlphabet<
        ExternalChoice::Alphabet,
        InternalChoice::Alphabet,
        Prefix::Alphabet,
        SequentialComposition::Alphabet,
        Skip::Alphabet,
        Stop::Alphabet,
    >;

    fn initials(&self) -> Self::Alphabet {
        match self {
            CSPSigCursor::ExternalChoice(this) => CSPSigAlphabet::ExternalChoice(this.initials()),
            CSPSigCursor::InternalChoice(this) => CSPSigAlphabet::InternalChoice(this.initials()),
            CSPSigCursor::Prefix(this) => CSPSigAlphabet::Prefix(this.initials()),
            CSPSigCursor::SequentialComposition(this) => {
                CSPSigAlphabet::SequentialComposition(this.initials())
            }
            CSPSigCursor::Skip(this) => CSPSigAlphabet::Skip(this.initials()),
            CSPSigCursor::Stop(this) => CSPSigAlphabet::Stop(this.initials()),
        }
    }

    fn perform(&mut self, event: &E) {
        match self {
            CSPSigCursor::ExternalChoice(this) => this.perform(event),
            CSPSigCursor::InternalChoice(this) => this.perform(event),
            CSPSigCursor::Prefix(this) => this.perform(event),
            CSPSigCursor::SequentialComposition(this) => this.perform(event),
            CSPSigCursor::Skip(this) => this.perform(event),
            CSPSigCursor::Stop(this) => this.perform(event),
        }
    }
}

#[doc(hidden)]
#[enum_derive(Debug, Display)]
#[derive(Clone, Eq, PartialEq)]
pub enum CSPSigAlphabet<ExternalChoice, InternalChoice, Prefix, SequentialComposition, Skip, Stop> {
    ExternalChoice(ExternalChoice),
    InternalChoice(InternalChoice),
    Prefix(Prefix),
    SequentialComposition(SequentialComposition),
    Skip(Skip),
    Stop(Stop),
}

impl<E, ExternalChoice, InternalChoice, Prefix, SequentialComposition, Skip, Stop> Alphabet<E>
    for CSPSigAlphabet<ExternalChoice, InternalChoice, Prefix, SequentialComposition, Skip, Stop>
where
    ExternalChoice: Alphabet<E>,
    InternalChoice: Alphabet<E>,
    Prefix: Alphabet<E>,
    SequentialComposition: Alphabet<E>,
    Skip: Alphabet<E>,
    Stop: Alphabet<E>,
{
    fn contains(&self, event: &E) -> bool {
        match self {
            CSPSigAlphabet::ExternalChoice(this) => this.contains(event),
            CSPSigAlphabet::InternalChoice(this) => this.contains(event),
            CSPSigAlphabet::Prefix(this) => this.contains(event),
            CSPSigAlphabet::SequentialComposition(this) => this.contains(event),
            CSPSigAlphabet::Skip(this) => this.contains(event),
            CSPSigAlphabet::Stop(this) => this.contains(event),
        }
    }
}

#[doc(hidden)]
#[enum_derive(Iterator)]
#[derive(Clone, Eq, PartialEq)]
pub enum CSPSigAlphabetIterator<
    ExternalChoice,
    InternalChoice,
    Prefix,
    SequentialComposition,
    Skip,
    Stop,
> {
    ExternalChoice(ExternalChoice),
    InternalChoice(InternalChoice),
    Prefix(Prefix),
    SequentialComposition(SequentialComposition),
    Skip(Skip),
    Stop(Stop),
}

impl<ExternalChoice, InternalChoice, Prefix, SequentialComposition, Skip, Stop> IntoIterator
    for CSPSigAlphabet<ExternalChoice, InternalChoice, Prefix, SequentialComposition, Skip, Stop>
where
    ExternalChoice: IntoIterator,
    InternalChoice: IntoIterator<Item = ExternalChoice::Item>,
    Prefix: IntoIterator<Item = ExternalChoice::Item>,
    SequentialComposition: IntoIterator<Item = ExternalChoice::Item>,
    Skip: IntoIterator<Item = ExternalChoice::Item>,
    Stop: IntoIterator<Item = ExternalChoice::Item>,
{
    type Item = ExternalChoice::Item;
    type IntoIter = CSPSigAlphabetIterator<
        ExternalChoice::IntoIter,
        InternalChoice::IntoIter,
        Prefix::IntoIter,
        SequentialComposition::IntoIter,
        Skip::IntoIter,
        Stop::IntoIter,
    >;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            CSPSigAlphabet::ExternalChoice(this) => {
                CSPSigAlphabetIterator::ExternalChoice(this.into_iter())
            }
            CSPSigAlphabet::InternalChoice(this) => {
                CSPSigAlphabetIterator::InternalChoice(this.into_iter())
            }
            CSPSigAlphabet::Prefix(this) => CSPSigAlphabetIterator::Prefix(this.into_iter()),
            CSPSigAlphabet::SequentialComposition(this) => {
                CSPSigAlphabetIterator::SequentialComposition(this.into_iter())
            }
            CSPSigAlphabet::Skip(this) => CSPSigAlphabetIterator::Skip(this.into_iter()),
            CSPSigAlphabet::Stop(this) => CSPSigAlphabetIterator::Stop(this.into_iter()),
        }
    }
}

#[cfg(test)]
mod proptest_support {
    use super::*;

    use proptest::arbitrary::any;
    use proptest::arbitrary::Arbitrary;
    use proptest::collection::vec;
    use proptest::prop_oneof;
    use proptest::strategy::BoxedStrategy;
    use proptest::strategy::Just;
    use proptest::strategy::MapInto;
    use proptest::strategy::Strategy;

    use crate::external_choice::external_choice;
    use crate::external_choice::replicated_external_choice;
    use crate::internal_choice::internal_choice;
    use crate::internal_choice::replicated_internal_choice;
    use crate::prefix::prefix;
    use crate::primitives::skip;
    use crate::primitives::stop;
    use crate::sequential_composition::sequential_composition;
    use crate::test_support::NumberedEvent;
    use crate::test_support::TestEvent;

    pub trait NameableEvents {
        type Strategy: Strategy;
        fn nameable_events() -> Self::Strategy;
    }

    impl NameableEvents for TestEvent {
        type Strategy = MapInto<<NumberedEvent as Arbitrary>::Strategy, TestEvent>;
        fn nameable_events() -> Self::Strategy {
            any::<NumberedEvent>().prop_map_into()
        }
    }

    impl<E> Arbitrary for CSP<E>
    where
        E: Clone + Debug + Display + NameableEvents + 'static,
        E::Strategy: Strategy<Value = E>,
    {
        type Parameters = ();
        type Strategy = BoxedStrategy<CSP<E>>;

        fn arbitrary_with(_args: ()) -> Self::Strategy {
            let leaf = prop_oneof![Just(stop()), Just(skip()),];
            let basic = leaf.prop_recursive(8, 16, 2, move |inner| {
                // We use NumberedEvent here because you shouldn't really create processes that
                // explicitly refer to Tau and Tick; those should only be created as part of
                // the CSP operators.
                (E::nameable_events(), inner.clone())
                    .prop_map(|(initial, after)| prefix(initial.into(), after))
            });
            basic
                .prop_recursive(4, 64, 16, move |inner| {
                    prop_oneof![
                        (inner.clone(), inner.clone()).prop_map(|(p, q)| external_choice(p, q)),
                        (inner.clone(), inner.clone()).prop_map(|(p, q)| internal_choice(p, q)),
                        (inner.clone(), inner.clone())
                            .prop_map(|(p, q)| sequential_composition(p, q)),
                        vec(inner.clone(), 3..8).prop_map(replicated_external_choice),
                        vec(inner.clone(), 3..8).prop_map(replicated_internal_choice),
                    ]
                })
                .boxed()
        }
    }
}
