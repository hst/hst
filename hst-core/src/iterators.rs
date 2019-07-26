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

//! Provides several extra iterator helper methods.

/// An `Iterator` blanket implementation that provides the [`replace`] method.
///
/// [`replace`]: trait.Replace.html#method.replace
pub trait Replace: Iterator + Sized {
    /// Replaces the `n`th element of an iterator with a new value.
    fn replace(self, n: usize, replacement: Self::Item) -> ReplaceIter<Self, Self::Item> {
        ReplaceIter {
            enumerated: self.enumerate(),
            n,
            replacement: Some(replacement),
        }
    }
}

impl<I> Replace for I where I: Iterator {}

#[doc(hidden)]
pub struct ReplaceIter<I, E> {
    enumerated: std::iter::Enumerate<I>,
    n: usize,
    replacement: Option<E>,
}

impl<I, E> Iterator for ReplaceIter<I, E>
where
    I: Iterator<Item = E>,
{
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        match self.enumerated.next() {
            Some((idx, element)) => {
                if idx == self.n {
                    Some(self.replacement.take().expect("Should not replace twice"))
                } else {
                    Some(element)
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod replace_tests {
    use super::*;

    use proptest::arbitrary::any;
    use proptest::arbitrary::Arbitrary;
    use proptest::collection::vec;
    use proptest::strategy::BoxedStrategy;
    use proptest::strategy::Just;
    use proptest::strategy::Strategy;
    use proptest_attr_macro::proptest;

    #[derive(Clone, Debug)]
    struct TestState {
        input: Vec<u8>,
        n: usize,
        replacement: u8,
    }

    impl Arbitrary for TestState {
        type Parameters = ();
        type Strategy = BoxedStrategy<TestState>;

        fn arbitrary_with(_args: ()) -> Self::Strategy {
            (vec(any::<u8>(), 1..100), any::<u8>())
                .prop_flat_map(|(input, replacement)| {
                    let len = input.len();
                    (Just(input), 0..len, Just(replacement))
                })
                .prop_map(|(input, n, replacement)| TestState {
                    input,
                    n,
                    replacement,
                })
                .boxed()
        }
    }

    #[proptest]
    fn can_replace_elements(state: TestState) {
        let mut expected = state.input.clone();
        expected[state.n] = state.replacement;
        let actual: Vec<u8> = state
            .input
            .into_iter()
            .replace(state.n, state.replacement)
            .collect();
        assert_eq!(actual, expected);
    }
}
