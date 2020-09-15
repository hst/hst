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

mod csp;
mod event;
mod maximal_traces;
mod prefix;
mod primitives;

pub use csp::CSP;
pub use event::DisjointSum;
pub use event::EventSet;
pub use maximal_traces::maximal_finite_traces;
pub use maximal_traces::MaximalTraces;
pub use primitives::Tau;
pub use primitives::Tick;

#[cfg(test)]
mod test_support;
