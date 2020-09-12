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

use std::marker::PhantomData;

use crate::event::EventSet;

#[derive(Clone, Eq, PartialEq)]
pub struct CSP<E>(PhantomData<E>);

impl<E> CSP<E>
where
    E: EventSet,
{
    pub fn initials(&self) -> E {
        E::empty()
    }

    pub fn transitions(&self, _events: &E) -> impl Iterator<Item = (E, CSP<E>)> + '_ {
        std::iter::empty()
    }
}
