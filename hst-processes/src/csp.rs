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

use std::rc::Rc;

use crate::event::EventSet;
use crate::primitives::Skip;
use crate::primitives::Stop;
use crate::primitives::Tick;

#[derive(Clone, Eq, PartialEq)]
pub struct CSP<E, TickProof>(Rc<CSPInner<E, TickProof>>);

impl<E, TickProof> CSP<E, TickProof> {
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
    E: EventSet + Tick<TickProof>,
{
    pub fn initials(&self) -> E {
        self.0.initials()
    }

    pub fn transitions(&self, events: &E) -> impl Iterator<Item = (E, CSP<E, TickProof>)> + '_ {
        self.0.transitions(events)
    }
}

#[derive(Eq, PartialEq)]
enum CSPInner<E, TickProof> {
    Skip(Skip<E, TickProof>),
    Stop(Stop<E>),
}

impl<E, TickProof> CSPInner<E, TickProof>
where
    E: EventSet + Tick<TickProof>,
{
    fn initials(&self) -> E {
        match self {
            CSPInner::Skip(this) => this.initials(),
            CSPInner::Stop(this) => this.initials(),
        }
    }

    fn transitions(&self, events: &E) -> Box<dyn Iterator<Item = (E, CSP<E, TickProof>)> + '_> {
        match self {
            CSPInner::Skip(this) => Box::new(this.transitions(events)),
            CSPInner::Stop(this) => Box::new(this.transitions(events)),
        }
    }
}
