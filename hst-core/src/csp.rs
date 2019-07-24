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

use auto_enums::enum_derive;
use auto_from::From;

use crate::primitives::Skip;
use crate::primitives::Stop;
use crate::process::Afters;
use crate::process::Initials;

/// A process type that includes all of the primitive processes and operators in the CSP language.
#[enum_derive(Debug, Display)]
#[derive(Clone, Eq, From, Hash, PartialEq)]
pub enum CSP {
    #[doc(hidden)]
    Stop(Stop),
    #[doc(hidden)]
    Skip(Skip),
}

#[doc(hidden)]
#[enum_derive(Iterator)]
pub enum CSPIter<STOP, SKIP> {
    Stop(STOP),
    Skip(SKIP),
}

#[doc(hidden)]
pub enum CSPIntoIter<STOP, SKIP> {
    Stop(STOP),
    Skip(SKIP),
}

impl<E, STOP, SKIP> IntoIterator for CSPIntoIter<STOP, SKIP>
where
    STOP: IntoIterator<Item = E>,
    SKIP: IntoIterator<Item = E>,
{
    type Item = E;
    type IntoIter = CSPIter<STOP::IntoIter, SKIP::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            CSPIntoIter::Stop(this) => CSPIter::Stop(this.into_iter()),
            CSPIntoIter::Skip(this) => CSPIter::Skip(this.into_iter()),
        }
    }
}

impl<E> Initials<E> for CSP
where
    Stop: Initials<E>,
    Skip: Initials<E>,
{
    type Initials = CSPIntoIter<<Stop as Initials<E>>::Initials, <Skip as Initials<E>>::Initials>;

    fn initials(&self) -> Self::Initials {
        match self {
            CSP::Stop(this) => CSPIntoIter::Stop(this.initials()),
            CSP::Skip(this) => CSPIntoIter::Skip(this.initials()),
        }
    }
}

impl<E, P> Afters<E, P> for CSP
where
    Stop: Afters<E, P>,
    Skip: Afters<E, P>,
{
    type Afters = CSPIntoIter<<Stop as Afters<E, P>>::Afters, <Skip as Afters<E, P>>::Afters>;

    fn afters(&self, initial: &E) -> Option<Self::Afters> {
        match self {
            CSP::Stop(this) => this.afters(initial).map(CSPIntoIter::Stop),
            CSP::Skip(this) => this.afters(initial).map(CSPIntoIter::Skip),
        }
    }
}
