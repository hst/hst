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
use auto_from::From;

use crate::external_choice::ExternalChoice;
use crate::internal_choice::InternalChoice;
use crate::prefix::Prefix;
use crate::primitives::Skip;
use crate::primitives::Stop;
use crate::primitives::Tau;
use crate::primitives::Tick;
use crate::process::Afters;
use crate::process::Initials;

/// A process type that includes all of the primitive processes and operators in the CSP language.
/// Note that you should never need to construct instances of this type directly; use the helper
/// constructor for each process (e.g. [`stop`]) or operator (TBD) instead.
///
/// [`stop`]: ../primitives/fn.stop.html
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

impl<'a, E> Initials<'a, E> for CSP<E>
where
    E: Clone + From<Tau> + From<Tick> + 'a,
{
    type Initials = Box<dyn Iterator<Item = E> + 'a>;

    fn initials(&'a self) -> Self::Initials {
        Box::new(self.0.initials())
    }
}

impl<'a, E> Afters<'a, E, CSP<E>> for CSP<E>
where
    E: Clone + Eq + From<Tau> + From<Tick> + 'a,
{
    type Afters = Box<dyn Iterator<Item = CSP<E>> + 'a>;

    fn afters(&'a self, initial: &E) -> Self::Afters {
        Box::new(self.0.afters(initial))
    }
}

#[doc(hidden)]
#[enum_derive(Debug, Display)]
#[derive(Clone, Eq, From, Hash, PartialEq)]
pub enum CSPSig<E, P> {
    #[doc(hidden)]
    Stop(Stop<E>),
    #[doc(hidden)]
    Skip(Skip),
    #[doc(hidden)]
    Prefix(Prefix<E, P>),
    #[doc(hidden)]
    ExternalChoice(ExternalChoice<P>),
    #[doc(hidden)]
    InternalChoice(InternalChoice<P>),
}

#[doc(hidden)]
#[enum_derive(Iterator)]
pub enum CSPIter<Stop, Skip, Prefix, ExternalChoice, InternalChoice> {
    Stop(Stop),
    Skip(Skip),
    Prefix(Prefix),
    ExternalChoice(ExternalChoice),
    InternalChoice(InternalChoice),
}

impl<'a, E, P> Initials<'a, E> for CSPSig<E, P>
where
    Stop<E>: Initials<'a, E>,
    Skip: Initials<'a, E>,
    Prefix<E, P>: Initials<'a, E>,
    ExternalChoice<P>: Initials<'a, E>,
    InternalChoice<P>: Initials<'a, E>,
{
    type Initials = CSPIter<
        <Stop<E> as Initials<'a, E>>::Initials,
        <Skip as Initials<'a, E>>::Initials,
        <Prefix<E, P> as Initials<'a, E>>::Initials,
        <ExternalChoice<P> as Initials<'a, E>>::Initials,
        <InternalChoice<P> as Initials<'a, E>>::Initials,
    >;

    fn initials(&'a self) -> Self::Initials {
        match self {
            CSPSig::Stop(this) => CSPIter::Stop(this.initials()),
            CSPSig::Skip(this) => CSPIter::Skip(this.initials()),
            CSPSig::Prefix(this) => CSPIter::Prefix(this.initials()),
            CSPSig::ExternalChoice(this) => CSPIter::ExternalChoice(this.initials()),
            CSPSig::InternalChoice(this) => CSPIter::InternalChoice(this.initials()),
        }
    }
}

impl<'a, E, P> Afters<'a, E, P> for CSPSig<E, P>
where
    Stop<E>: Afters<'a, E, P>,
    Skip: Afters<'a, E, P>,
    Prefix<E, P>: Afters<'a, E, P>,
    ExternalChoice<P>: Afters<'a, E, P>,
    InternalChoice<P>: Afters<'a, E, P>,
{
    type Afters = CSPIter<
        <Stop<E> as Afters<'a, E, P>>::Afters,
        <Skip as Afters<'a, E, P>>::Afters,
        <Prefix<E, P> as Afters<'a, E, P>>::Afters,
        <ExternalChoice<P> as Afters<'a, E, P>>::Afters,
        <InternalChoice<P> as Afters<'a, E, P>>::Afters,
    >;

    fn afters(&'a self, initial: &E) -> Self::Afters {
        match self {
            CSPSig::Stop(this) => CSPIter::Stop(this.afters(initial)),
            CSPSig::Skip(this) => CSPIter::Skip(this.afters(initial)),
            CSPSig::Prefix(this) => CSPIter::Prefix(this.afters(initial)),
            CSPSig::ExternalChoice(this) => CSPIter::ExternalChoice(this.afters(initial)),
            CSPSig::InternalChoice(this) => CSPIter::InternalChoice(this.afters(initial)),
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
    use crate::internal_choice::internal_choice;
    use crate::internal_choice::replicated_internal_choice;
    use crate::prefix::prefix;
    use crate::primitives::skip;
    use crate::primitives::stop;
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
            let leaf = prop_oneof![Just(stop()), Just(skip())];
            leaf.prop_recursive(8, 256, 100, move |inner| {
                prop_oneof![
                    // We use NumberedEvent here because you shouldn't really create processes that
                    // explicitly refer to Tau and Tick; those should only be created as part of
                    // the CSP operators.
                    (E::nameable_events(), inner.clone())
                        .prop_map(|(initial, after)| prefix(initial.into(), after)),
                    (inner.clone(), inner.clone()).prop_map(|(p, q)| external_choice(p, q)),
                    (inner.clone(), inner.clone()).prop_map(|(p, q)| internal_choice(p, q)),
                    vec(inner.clone(), 1..100).prop_map(replicated_internal_choice),
                ]
            })
            .boxed()
        }
    }

}
