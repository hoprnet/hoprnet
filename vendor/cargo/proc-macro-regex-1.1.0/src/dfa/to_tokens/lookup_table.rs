use crate::{
    character::Character,
    dfa::to_tokens::{usize_to_lit_int, DfaToTokens},
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::{
    collections::{BTreeMap, BTreeSet},
    mem::size_of,
};
use syn::Ident;

impl<T> DfaToTokens<T>
where
    T: Character + ToTokens + Copy + Into<u32>,
{
    fn lookup_table_u8_row_map(transitions: &BTreeSet<(T, usize)>) -> Option<BTreeMap<u8, usize>> {
        let mut transitions_u8 = BTreeMap::new();
        for (ch, t) in transitions.iter() {
            let ch = ch.to_byte()?;
            transitions_u8.insert(ch, *t);
        }
        Some(transitions_u8)
    }

    fn lookup_table_row_no_transition_default(&self, int_type: &Ident) -> TokenStream {
        if self.dfa.start_text {
            quote! {
                #int_type::MAX
            }
        } else {
            quote! {
                0
            }
        }
    }

    fn lookup_table_row_accept_transition_end(&self, int_type: &Ident) -> TokenStream {
        if self.dfa.start_text {
            quote! {
                #int_type::MAX - 1
            }
        } else {
            quote! {
                #int_type::MAX
            }
        }
    }

    fn lookup_table_row(
        &self,
        transitions_u8: &BTreeMap<u8, usize>,
        int_type: &Ident,
    ) -> Vec<TokenStream> {
        let no_transition_default = self.lookup_table_row_no_transition_default(int_type);
        let accept_transition_end = self.lookup_table_row_accept_transition_end(int_type);
        let mut row = Vec::with_capacity(256);
        for i in 0..=u8::MAX {
            let new_state = if let Some(t) = transitions_u8.get(&i) {
                if !self.dfa.end_text && self.dfa.accept_states.contains(t) {
                    accept_transition_end.clone()
                } else {
                    let new_state = usize_to_lit_int(*t);
                    quote! {
                        #new_state
                    }
                }
            } else {
                no_transition_default.clone()
            };
            row.push(new_state);
        }
        row
    }

    fn lookup_table_row_default(&self, int_type: &Ident) -> Vec<TokenStream> {
        let no_transition_default = self.lookup_table_row_no_transition_default(int_type);
        vec![no_transition_default; 256]
    }

    fn transitions_lookup_table(&self, int_type: &Ident) -> Option<TokenStream> {
        let mut table = Vec::new();
        for state in self.required_states.iter() {
            let row = if let Some(transitions) = self.dfa.transitions.get(state) {
                let transitions_u8 = DfaToTokens::<T>::lookup_table_u8_row_map(transitions)?;
                self.lookup_table_row(&transitions_u8, int_type)
            } else {
                self.lookup_table_row_default(int_type)
            };
            table.push(quote! {
                [#(#row),*]
            });
        }
        let len = table.len();
        let transitions = quote! {
            static TABLE: [[#int_type; 256]; #len] = [#(#table),*]
        };
        Some(transitions)
    }

    fn for_each_lookup_table_check(&self, int_type: &Ident) -> TokenStream {
        match (self.dfa.start_text, self.dfa.end_text) {
            (false, false) => quote! {
                if state == #int_type::MAX {
                    return true;
                }
            },
            (false, true) => quote! {},
            (true, false) => quote! {
                if state == #int_type::MAX {
                    return false;
                } else if state == #int_type::MAX - 1 {
                    return true;
                }
            },
            (true, true) => quote! {
                if state == #int_type::MAX {
                    return false;
                }
            },
        }
    }

    pub(super) fn for_each_lookup_table(&self) -> Option<TokenStream> {
        if !self.is_byte {
            return None;
        }

        let int_type = self.get_int_type()?;
        let transitions_lookup_table = self.transitions_lookup_table(&int_type)?;
        let iterator = T::get_iterator_function(self.is_byte);
        let c_to_usize = T::to_usize(Ident::new("c", Span::call_site()), self.is_byte);
        let check = self.for_each_lookup_table_check(&int_type);
        let for_each = quote! {
            #transitions_lookup_table;
            let mut state = 0;

            for c in s.#iterator() {
                state = TABLE[state as usize][#c_to_usize];

                #check
            }
        };
        Some(for_each)
    }

    fn lookup_table_states(&self) -> Option<usize> {
        let mut additional_state = 0;

        if self.dfa.start_text {
            additional_state += 1;
        }

        if !self.dfa.end_text {
            additional_state += 1;
        }

        self.required_states.len().checked_add(additional_state)
    }

    fn lookup_table_size(&self) -> Option<(usize, Ident)> {
        let states = self.lookup_table_states()?;
        let states_character = states.checked_mul(256)?;
        let ret = if states < (u8::MAX) as usize {
            (
                states_character.checked_mul(size_of::<u8>())?,
                Ident::new("u8", Span::call_site()),
            )
        } else if states < (u16::MAX) as usize {
            (
                states_character.checked_mul(size_of::<u16>())?,
                Ident::new("u16", Span::call_site()),
            )
        } else if states < (u32::MAX) as usize {
            (
                states_character.checked_mul(size_of::<u32>())?,
                Ident::new("u32", Span::call_site()),
            )
        } else {
            (
                states_character.checked_mul(size_of::<u64>())?,
                Ident::new("u64", Span::call_site()),
            )
        };
        Some(ret)
    }

    pub(super) fn get_int_type(&self) -> Option<Ident> {
        let (lookup_table_size, int_type) = self.lookup_table_size()?;
        if lookup_table_size <= self.threshold {
            Some(int_type)
        } else {
            None
        }
    }
}
