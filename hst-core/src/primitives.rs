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

use std::fmt::Debug;
use std::fmt::Display;
use std::marker::PhantomData;

use crate::process::Afters;
use crate::process::Initials;

//-------------------------------------------------------------------------------------------------
// Stop

/// The process that performs no actions (and prevents any other synchronized processes from
/// performing any, either).
pub struct Stop;

impl Display for Stop {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Stop")
    }
}

impl Debug for Stop {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &Display).fmt(f)
    }
}

pub struct StopInitials<E>(PhantomData<E>);

impl<E> IntoIterator for StopInitials<E> {
    type Item = E;
    type IntoIter = std::iter::Empty<E>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::empty()
    }
}

impl<E> Initials<E> for Stop {
    type Initials = StopInitials<E>;

    fn initials(&self) -> Self::Initials {
        StopInitials(PhantomData)
    }
}

pub struct StopAfters<P>(PhantomData<P>);

impl<P> IntoIterator for StopAfters<P> {
    type Item = P;
    type IntoIter = std::iter::Empty<P>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::empty()
    }
}

impl<E, P> Afters<E, P> for Stop {
    type Afters = StopAfters<P>;

    fn afters(&self, _initial: &E) -> Option<Self::Afters> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use crate::process::transitions;

    #[test]
    fn stop_has_no_transitions() {
        let transitions: HashMap<(), Vec<Stop>> = transitions(&Stop);
        assert!(transitions.is_empty());
    }
}
