use crate::{character::Character, dfa::to_tokens::DfaToTokens};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::{BTreeMap, BTreeSet};

fn transition_condition_to_tokens<T>(start: T, end: T) -> TokenStream
where
    T: ToTokens + Ord,
{
    if start == end {
        quote! {
            #start
        }
    } else {
        quote! {
            #start..=#end
        }
    }
}

impl<T> DfaToTokens<T>
where
    T: Character + ToTokens + Ord,
{
    fn transition_condition(&self, start: T, end: T) -> TokenStream {
        if self.is_byte {
            if let Some(start) = start.to_byte() {
                if let Some(end) = end.to_byte() {
                    return transition_condition_to_tokens::<u8>(start, end);
                }
            }
        }

        transition_condition_to_tokens(start, end)
    }
}

impl<T> DfaToTokens<T>
where
    T: Character + ToTokens + Copy,
{
    fn transitions_inverse(transitions: &BTreeSet<(T, usize)>) -> BTreeMap<usize, BTreeSet<T>> {
        let mut result: BTreeMap<usize, BTreeSet<T>> = BTreeMap::new();
        for (c, t) in transitions {
            if let Some(set) = result.get_mut(t) {
                set.insert(*c);
            } else {
                let mut set = BTreeSet::new();
                set.insert(*c);
                result.insert(*t, set);
            }
        }
        result
    }

    fn transitions_inverse_pack(
        transitions_inverse: BTreeMap<usize, BTreeSet<T>>,
    ) -> BTreeMap<usize, BTreeSet<(T, T)>> {
        let mut result = BTreeMap::new();
        for (t, cs) in transitions_inverse {
            let mut ranges = BTreeSet::new();
            let mut start = None;
            let mut prev: Option<T> = None;
            for character in cs {
                if let Some(prev) = prev {
                    if !prev.is_next(&character) {
                        ranges.insert((start.unwrap(), prev));
                        start = Some(character);
                    }
                } else {
                    start = Some(character);
                }
                prev = Some(character);
            }
            if let Some(start) = start {
                if let Some(prev) = prev {
                    ranges.insert((start, prev));
                } else {
                    panic!()
                }
            }
            result.insert(t, ranges);
        }
        result
    }

    fn transitions_inverse_condition(&self, ranges: BTreeSet<(T, T)>) -> TokenStream {
        let mut conditions = Vec::new();
        for (start, end) in ranges {
            let condition = self.transition_condition(start, end);
            conditions.push(condition);
        }
        quote! {
            #(#conditions )|*
        }
    }

    fn transitions_default(&self) -> TokenStream {
        if self.dfa.start_text {
            quote! {
                return false;
            }
        } else {
            quote! {
                0usize
            }
        }
    }

    fn transitions_binary_search_match_inner(&self, state: usize) -> TokenStream {
        let default = self.transitions_default();
        if let Some(transitions) = self.dfa.transitions.get(&state) {
            let transitions_inverse = DfaToTokens::<T>::transitions_inverse(transitions);
            let transitions_inverse_pack =
                DfaToTokens::<T>::transitions_inverse_pack(transitions_inverse);
            let mut arms = Vec::new();
            for (t, ranges) in transitions_inverse_pack {
                let condition = self.transitions_inverse_condition(ranges);
                let arm = if !self.dfa.end_text && self.dfa.accept_states.contains(&t) {
                    quote! {
                        #condition => return true
                    }
                } else {
                    quote! {
                        #condition => #t
                    }
                };
                arms.push(arm);
            }

            quote! {
                match c {
                    #(#arms,)*
                    _ => {
                        #default
                    },
                }
            }
        } else {
            default
        }
    }

    fn transitions_binary_search_recursive(
        &self,
        states: &[usize],
        start: usize,
        len: usize,
    ) -> TokenStream {
        if len == 1 {
            self.transitions_binary_search_match_inner(states[start])
        } else if len == 2 {
            let left_state = states[start];
            let right_state = states[start + 1];
            let left = self.transitions_binary_search_match_inner(left_state);
            let right = self.transitions_binary_search_match_inner(right_state);
            quote! {
                if state == #left_state {
                    #left
                } else {
                    #right
                }
            }
        } else {
            let new_len = len / 2;
            let remain = len % 2;
            let new_start = start + new_len;
            let new_state = states[new_start];
            let left = self.transitions_binary_search_recursive(states, start, new_len);
            let right =
                self.transitions_binary_search_recursive(states, new_start, new_len + remain);
            quote! {
                if state < #new_state {
                    #left
                } else {
                    #right
                }
            }
        }
    }

    pub(super) fn for_each_binary_search(&self) -> TokenStream {
        let states: Vec<usize> = self.required_states.iter().copied().collect();
        let iterator = T::get_iterator_function(self.is_byte);
        let transitions = self.transitions_binary_search_recursive(&states[..], 0, states.len());
        quote! {
            let mut state = 0;

            for c in s.#iterator() {
                state = #transitions;
            }
        }
    }
}
