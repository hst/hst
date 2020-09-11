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

//! Defines primitive CSP events and processes.

use std::fmt::Debug;
use std::fmt::Display;

use crate::event::EventSet;

//-------------------------------------------------------------------------------------------------
// Built-in CSP events

/// The _tau_ event (τ).  This is the hidden event that expresses nondeterminism in a CSP process.
/// You should rarely have to construct it directly, unless you're digging through the transitions
/// of a process.
pub trait Tau {
    fn tau() -> Self;
    fn can_perform_tau(&self) -> bool;
}

/// The _tick_ event (✔).  This is the hidden event that represents the end of a process that can
/// be sequentially composed with another process.  You should rarely have to construct it
/// directly, unless you're digging through the transitions of a process.
pub trait Tick {
    fn tick() -> Self;
    fn can_perform_tick(&self) -> bool;
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct PrimitiveEvents {
    contains_tau: bool,
    contains_tick: bool,
}

impl Debug for PrimitiveEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match (self.contains_tau, self.contains_tick) {
            (true, true) => write!(f, "PrimitiveEvents {{τ,✔}}"),
            (true, false) => write!(f, "PrimitiveEvents {{τ}}"),
            (false, true) => write!(f, "PrimitiveEvents {{✔}}"),
            (false, false) => write!(f, "PrimitiveEvents {{}}"),
        }
    }
}

impl Display for PrimitiveEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match (self.contains_tau, self.contains_tick) {
            (true, true) => write!(f, "{{τ,✔}}"),
            (true, false) => write!(f, "{{τ}}"),
            (false, true) => write!(f, "{{✔}}"),
            (false, false) => write!(f, "{{}}"),
        }
    }
}

impl EventSet for PrimitiveEvents {
    fn empty() -> Self {
        PrimitiveEvents {
            contains_tau: false,
            contains_tick: false,
        }
    }

    fn intersect(&mut self, other: &Self) {
        self.contains_tau &= other.contains_tau;
        self.contains_tick &= other.contains_tick;
    }

    fn is_empty(&self) -> bool {
        !self.contains_tau && !self.contains_tick
    }

    fn negate(&mut self) {
        self.contains_tau = !self.contains_tau;
        self.contains_tick = !self.contains_tick;
    }

    fn subtract(&mut self, other: &Self) {
        self.contains_tau &= !other.contains_tau;
        self.contains_tick &= !other.contains_tick;
    }

    fn union(&mut self, other: &Self) {
        self.contains_tau |= other.contains_tau;
        self.contains_tick |= other.contains_tick;
    }

    fn universe() -> Self {
        PrimitiveEvents {
            contains_tau: true,
            contains_tick: true,
        }
    }
}

impl Tau for PrimitiveEvents {
    fn tau() -> Self {
        PrimitiveEvents {
            contains_tau: true,
            contains_tick: false,
        }
    }

    fn can_perform_tau(&self) -> bool {
        self.contains_tau
    }
}

impl Tick for PrimitiveEvents {
    fn tick() -> Self {
        PrimitiveEvents {
            contains_tau: false,
            contains_tick: true,
        }
    }

    fn can_perform_tick(&self) -> bool {
        self.contains_tick
    }
}

#[cfg(test)]
mod primitive_events_tests {
    use super::*;

    #[test]
    fn can_check_for_tau() {
        assert!(!PrimitiveEvents::empty().can_perform_tau());
        assert!(PrimitiveEvents::tau().can_perform_tau());
        assert!(PrimitiveEvents::universe().can_perform_tau());
    }

    #[test]
    fn can_check_for_tick() {
        assert!(!PrimitiveEvents::empty().can_perform_tick());
        assert!(PrimitiveEvents::tick().can_perform_tick());
        assert!(PrimitiveEvents::universe().can_perform_tick());
    }
}
