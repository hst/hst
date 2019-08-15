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

//! Defines the different semantic models that you can use to interpret a CSP process.

use std::collections::HashSet;
use std::hash::Hash;

use crate::primitives::tau;
use crate::primitives::Tau;
use crate::process::Cursor;

//-------------------------------------------------------------------------------------------------
// Built-in CSP events

/// A semantic model of CSP.  Each semantic model defines its own _behavior_ type - the set of
/// information that fully describes the behavior of a process in this model.
pub trait SemanticModel<E> {
    type Behavior: Behavior;

    /// Returns the behavior of a process in this semantic model.
    fn get_behavior<C>(cursor: &C) -> Self::Behavior
    where
        C: Cursor<E>;
}

/// The behavior of a process under a particular semantic model.  The only operation that all
/// models have in common is whether one behavior is a _refinement_ of another.
pub trait Behavior {
    fn refined_by(&self, other: &Self) -> bool;
}

//-------------------------------------------------------------------------------------------------
// Traces

/// In the traces model, the behavior of a process is the set of non-τ events that it can perform.
pub struct Traces;

impl<E> SemanticModel<E> for Traces
where
    E: Eq + From<Tau> + Hash,
{
    type Behavior = HashSet<E>;

    fn get_behavior<C>(cursor: &C) -> Self::Behavior
    where
        C: Cursor<E>,
    {
        cursor.events().filter(|event| *event != tau()).collect()
    }
}

impl<E> Behavior for HashSet<E>
where
    E: Eq + Hash,
{
    fn refined_by(&self, other: &Self) -> bool {
        self.is_subset(other)
    }
}
