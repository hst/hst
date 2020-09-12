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
use crate::primitives::Stop;

#[derive(Clone, Eq, PartialEq)]
pub struct CSP<E>(Rc<CSPInner<E>>);

impl<E> CSP<E> {
    /// Constructs a new _Stop_ process.  This is the process that performs no actions (and
    /// prevents any other synchronized processes from performing any, either).
    pub fn stop() -> Self {
        CSP(Rc::new(CSPInner::Stop(Stop::new())))
    }
}

impl<E> CSP<E>
where
    E: EventSet,
{
    pub fn initials(&self) -> E {
        self.0.initials()
    }

    pub fn transitions(&self, events: &E) -> impl Iterator<Item = (E, CSP<E>)> + '_ {
        self.0.transitions(events)
    }
}

#[derive(Eq, PartialEq)]
enum CSPInner<E> {
    Stop(Stop<E, CSP<E>>),
}

impl<E> CSPInner<E>
where
    E: EventSet,
{
    fn initials(&self) -> E {
        match self {
            CSPInner::Stop(this) => this.initials(),
        }
    }

    fn transitions(&self, events: &E) -> Box<dyn Iterator<Item = (E, CSP<E>)> + '_> {
        match self {
            CSPInner::Stop(this) => Box::new(this.transitions(events)),
        }
    }
}
