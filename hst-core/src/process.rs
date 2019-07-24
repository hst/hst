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

//! Defines several traits that CSP processes will probably implement.

use std::collections::HashMap;
use std::hash::Hash;
use std::iter::FromIterator;

/// Returns the events that the process is willing to perform.
pub trait Initials<E> {
    type Initials;
    fn initials(&self) -> Self::Initials;
}

/// Returns how the process behaves after one of its initial events is performed.  The result is
/// a _set_ of processes; if there are multiple processes in the set, then there is nondeterminism,
/// and the process will behave like one of the elements arbitrarily.
pub trait Afters<E, P> {
    type Afters;
    fn afters(&self, initial: &E) -> Option<Self::Afters>;
}

/// Returns all of the outgoing transitions for the process.  This is a map where the keys are the
/// initial events of the process, and the values are a collection of the after processes for each
/// of those events.
pub fn transitions<'a, E, P, C>(process: &P) -> HashMap<E, C>
where
    E: Eq + Hash,
    P: Initials<E>,
    P: Afters<E, P>,
    C: FromIterator<P>,
    P::Initials: IntoIterator<Item = E>,
    P::Afters: IntoIterator<Item = P>,
{
    let mut transitions = HashMap::new();
    for initial in process.initials() {
        let afters = process
            .afters(&initial)
            .expect("No afters for initial")
            .into_iter()
            .collect();
        transitions.insert(initial, afters);
    }
    transitions
}
