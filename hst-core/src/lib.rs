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

mod csp;
mod event;
mod external_choice;
mod internal_choice;
mod prefix;
mod primitives;
mod process;
mod sequential_composition;

pub use csp::CSP;
pub use event::Alphabet;
pub use event::EmptyAlphabet;
pub use external_choice::external_choice;
pub use external_choice::replicated_external_choice;
pub use internal_choice::internal_choice;
pub use internal_choice::replicated_internal_choice;
pub use prefix::prefix;
pub use primitives::skip;
pub use primitives::stop;
pub use primitives::tau;
pub use primitives::tick;
pub use process::maximal_finite_traces;
pub use process::satisfies_trace;
pub use process::Cursor;
pub use process::Process;
pub use sequential_composition::sequential_composition;

#[cfg(test)]
mod test_support;
