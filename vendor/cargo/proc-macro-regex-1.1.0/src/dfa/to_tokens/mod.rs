mod binary_search;
mod lookup_table;

use crate::{character::Character, dfa::Dfa, nfa::START_STATE};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::BTreeSet;
use syn::LitInt;

fn usize_to_lit_int(i: usize) -> LitInt {
    let s = format!("{}", i);
    LitInt::new(&s, Span::call_site())
}

pub(crate) struct DfaToTokens<T>
where
    T: Character,
{
    dfa: Dfa<T>,
    threshold: usize,
    required_states: BTreeSet<usize>,
    is_byte: bool,
}

impl<T> DfaToTokens<T>
where
    T: Character,
{
    /// If `self.end_text` is `true` then only no accept-states have to be implemented.
    /// Because if the state machine reaches an accept-state, then it stops.
    fn get_required_states(dfa: &Dfa<T>) -> BTreeSet<usize> {
        if dfa.end_text {
            dfa.states.clone()
        } else {
            let mut required_states = BTreeSet::new();
            for state in dfa.states.iter() {
                if !dfa.accept_states.contains(state) {
                    required_states.insert(*state);
                }
            }
            required_states
        }
    }

    fn is_byte(dfa: &Dfa<T>) -> bool {
        for (_, transitions) in dfa.transitions.iter() {
            for (ch, _) in transitions.iter() {
                if !ch.is_byte() {
                    return false;
                }
            }
        }
        true
    }

    fn returns_true(&self) -> bool {
        if self.dfa.accept_states.contains(&START_STATE) {
            if self.dfa.end_text {
                !self.dfa.start_text && self.dfa.states.len() == 1
            } else {
                true
            }
        } else {
            false
        }
    }
}

impl<T> DfaToTokens<T>
where
    T: Character + ToTokens + Copy + Into<u32>,
{
    pub(crate) fn new(dfa: Dfa<T>, threshold: usize) -> DfaToTokens<T> {
        let required_states = DfaToTokens::get_required_states(&dfa);
        let is_byte = DfaToTokens::is_byte(&dfa);
        DfaToTokens {
            dfa,
            required_states,
            threshold,
            is_byte,
        }
    }

    fn last_check(&self) -> TokenStream {
        if self.dfa.end_text {
            let accept_states: Vec<LitInt> = self
                .dfa
                .accept_states
                .iter()
                .map(|u| usize_to_lit_int(*u))
                .collect();
            quote! {
                match state {
                    #(#accept_states => true,)*
                    _ => false,
                }
            }
        } else {
            quote! {
                false
            }
        }
    }

    fn for_each(&self) -> TokenStream {
        if let Some(for_each_lookup_table) = self.for_each_lookup_table() {
            for_each_lookup_table
        } else {
            self.for_each_binary_search()
        }
    }

    pub fn get_token_streams(&self) -> TokenStream {
        if self.returns_true() {
            quote! {true}
        } else {
            let for_each = self.for_each();
            let last_check = self.last_check();
            quote! {
                #for_each

                #last_check
            }
        }
    }
}
