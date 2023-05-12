mod nfa_to_dfa;
mod to_tokens;

pub(super) use crate::dfa::to_tokens::DfaToTokens;
use crate::{character::Character, dfa::nfa_to_dfa::NfaToDfaIter, nfa::Nfa};
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::From,
    fmt::Debug,
};

#[derive(Debug)]
pub(crate) struct Dfa<T>
where
    T: Character,
{
    states: BTreeSet<usize>,
    transitions: BTreeMap<usize, BTreeSet<(T, usize)>>,
    accept_states: BTreeSet<usize>,
    start_text: bool,
    end_text: bool,
}

impl<T> From<Nfa<T>> for Dfa<T>
where
    T: Character + Copy,
{
    fn from(nfa: Nfa<T>) -> Self {
        let nfa_to_dfa = NfaToDfaIter::new(nfa);
        Dfa::from(nfa_to_dfa)
    }
}
