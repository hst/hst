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

//! Defines normalized processes — those in which we go through increasing lengths to collapse
//! identically behaving subprocesses together.

use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;

use itertools::Itertools;

use crate::primitives::tau;
use crate::primitives::Tau;
use crate::process::Cursor;
use crate::process::Process;

/// _Prenormalizes_ a process.  Our representation of process cursors already keeps track of the
/// _set_ of states that a process might be in, so the only thing we have to do is compute a τ
/// closure of each state.
pub fn prenormalize<P>(p: P) -> Prenormalization<P> {
    Prenormalization(p)
}

#[doc(hidden)]
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Prenormalization<P>(P);

impl<P: Debug + Display> Display for Prenormalization<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "prenormalize {}", self.0)
    }
}

impl<P: Debug + Display> Debug for Prenormalization<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (self as &dyn Display).fmt(f)
    }
}

#[doc(hidden)]
#[derive(Clone, Eq, PartialEq)]
pub struct PrenormalizationCursor<E, C>
where
    C: Eq + Hash,
{
    phantom: PhantomData<E>,
    tau_closed: HashSet<C>,
}

impl<E, C> Debug for PrenormalizationCursor<E, C>
where
    C: Debug + Eq + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("PrenormalizationCursor ")?;
        f.debug_set().entries(&self.tau_closed).finish()
    }
}

impl<E, P> Process<E> for Prenormalization<P>
where
    E: From<Tau>,
    P: Process<E>,
    P::Cursor: Clone + Eq + Hash,
{
    type Cursor = PrenormalizationCursor<E, P::Cursor>;

    fn root(&self) -> Self::Cursor {
        let mut cursor = PrenormalizationCursor {
            phantom: PhantomData,
            tau_closed: HashSet::new(),
        };
        cursor.tau_close(std::iter::once(self.0.root()).collect());
        cursor
    }
}

impl<E, C> PrenormalizationCursor<E, C>
where
    C: Clone + Cursor<E> + Eq + Hash,
{
    fn tau_close(&mut self, cursors: VecDeque<C>)
    where
        E: From<Tau>,
    {
        let mut to_add = cursors.into_iter().collect::<VecDeque<_>>();
        while let Some(next) = to_add.pop_front() {
            if next.can_perform(&tau()) {
                let mut after = next.clone();
                after.perform(&tau());
                to_add.push_back(after);
            }
            self.tau_closed.insert(next);
        }
    }
}

impl<E, C> Cursor<E> for PrenormalizationCursor<E, C>
where
    E: From<Tau>,
    C: Clone + Cursor<E> + Eq + Hash,
{
    fn events<'a>(&'a self) -> Box<dyn Iterator<Item = E> + 'a> {
        Box::new(self.tau_closed.iter().flat_map(C::events))
    }

    fn can_perform(&self, event: &E) -> bool {
        self.tau_closed
            .iter()
            .any(|subcursor| subcursor.can_perform(event))
    }

    fn perform(&mut self, event: &E) {
        let afters = self
            .tau_closed
            .drain()
            .filter(|subcursor| subcursor.can_perform(event))
            .update(|subcursor| subcursor.perform(event))
            .collect();
        self.tau_close(afters);
    }
}

#[cfg(test)]
mod prenormalization_tests {
    use super::*;

    use proptest_attr_macro::proptest;

    use crate::csp::CSP;
    use crate::process::initials;
    use crate::process::maximal_finite_traces;
    use crate::test_support::TestEvent;

    #[proptest]
    fn check_prenormalization(p: CSP<TestEvent>) {
        let process = dbg!(prenormalize(p.clone()));
        assert!(initials(&p.root()).is_subset(&initials(&p.root())));
        assert_eq!(
            maximal_finite_traces(process.root()),
            maximal_finite_traces(p.root())
        );
    }
}
