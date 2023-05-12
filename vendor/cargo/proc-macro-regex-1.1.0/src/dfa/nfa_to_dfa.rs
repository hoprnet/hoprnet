use crate::{
    character::Character,
    dfa::Dfa,
    nfa::{Nfa, START_STATE},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::From,
    fmt::Debug,
};

type State = BTreeSet<usize>;
type Transition<T> = (State, T, State);

#[derive(Debug)]
pub(crate) struct NfaToDfaIter<T>
where
    T: Character + Copy,
{
    nfa: Nfa<T>,
    states: BTreeSet<State>,
    new_states: BTreeSet<State>,
    transitions: BTreeSet<Transition<T>>,
    accept_states: BTreeSet<State>,
}

impl<T> NfaToDfaIter<T>
where
    T: Character + Copy,
{
    pub(super) fn new(nfa: Nfa<T>) -> NfaToDfaIter<T> {
        let mut start_state = BTreeSet::new();
        start_state.insert(START_STATE.to_owned());

        // The start state is always there.
        let mut states: BTreeSet<BTreeSet<usize>> = BTreeSet::new();
        states.insert(start_state);

        // The start state is an accept-state and the end_text is false. This means that we are
        // already on a accept-state and we do not have to parse all the string. As a result, the
        // DFA is always true.
        let new_states = if nfa.is_accept_state(START_STATE) && !nfa.is_end_text() {
            BTreeSet::new()
        } else {
            states.clone()
        };

        let accept_states = if nfa.is_accept_state(START_STATE) {
            states.clone()
        } else {
            BTreeSet::new()
        };

        NfaToDfaIter {
            nfa,
            states,
            new_states,
            transitions: BTreeSet::new(),
            accept_states,
        }
    }

    /// Returns a set of all character the given state has a transition as source state.
    fn characters(&self, state: &State) -> BTreeSet<T> {
        let mut characters = BTreeSet::new();
        for s in state {
            self.nfa.chars(*s, &mut characters);
        }
        characters
    }

    ///
    fn simulate(&self, state: &State, c: T) -> State {
        let mut new_state = BTreeSet::new();
        for s in state {
            self.nfa.simulate(*s, c, &mut new_state);
        }
        new_state
    }

    fn is_accept_state(&self, state: &State) -> bool {
        for s in state {
            if self.nfa.is_accept_state(*s) {
                return true;
            }
        }
        false
    }

    fn next_step(&mut self) {
        let mut new_states = BTreeSet::new();
        for state in self.new_states.iter() {
            let chars = self.characters(state);
            for c in chars {
                let mut new_state = self.simulate(state, c);
                if !self.nfa.is_start_text() {
                    new_state.insert(START_STATE.to_owned());
                }

                if !self.states.contains(&new_state) {
                    self.states.insert(new_state.clone());
                    new_states.insert(new_state.clone());

                    if self.is_accept_state(&new_state) {
                        self.accept_states.insert(new_state.clone());
                    }
                }
                self.transitions.insert((state.clone(), c, new_state));
            }
        }
        self.new_states = new_states;
    }
}

impl<T> Iterator for &mut NfaToDfaIter<T>
where
    T: Character + Copy,
{
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.new_states.is_empty() {
            return None;
        }

        self.next_step();

        match self.new_states.len() {
            0 => None,
            len => Some(len),
        }
    }
}

impl<T> From<NfaToDfaIter<T>> for Dfa<T>
where
    T: Character + Copy,
{
    fn from(mut nfa_to_dfa: NfaToDfaIter<T>) -> Self {
        for _ in &mut nfa_to_dfa {}

        let mut states = BTreeSet::new();
        let mut accept_states = BTreeSet::new();
        let mut mapping = BTreeMap::new();

        let mut start_state = BTreeSet::new();
        start_state.insert(START_STATE);

        // It has to be ensured that the start-state is mapped to zero.
        // Therefore, the start-state has to be removed.
        nfa_to_dfa.states.remove(&start_state);
        states.insert(START_STATE);
        if nfa_to_dfa.accept_states.remove(&start_state) {
            accept_states.insert(START_STATE);
        }
        mapping.insert(start_state, START_STATE);

        // First map all non accept-states.
        for state in nfa_to_dfa.states {
            if !nfa_to_dfa.accept_states.contains(&state) {
                states.insert(mapping.len());
                mapping.insert(state, mapping.len());
            }
        }

        // Then map all accept-states.
        // Because if `end_text` equals true then the accept states are implemented differently so
        // all accept-states should be at the end of the mapping.
        for accept_state in nfa_to_dfa.accept_states.iter() {
            states.insert(mapping.len());
            mapping.insert(accept_state.clone(), mapping.len());
        }

        // Convert the transitions according the mapping.
        let mut transitions: BTreeMap<usize, BTreeSet<(T, usize)>> = BTreeMap::new();
        for (s, c, t) in nfa_to_dfa.transitions {
            let s = mapping.get(&s).unwrap();
            let t = mapping.get(&t).unwrap();
            if let Some(state_transitions) = transitions.get_mut(s) {
                state_transitions.insert((c, *t));
            } else {
                let mut state_transitions = BTreeSet::new();
                state_transitions.insert((c, *t));
                transitions.insert(*s, state_transitions);
            }
        }

        // Convert the accept states according the mapping.
        for s in nfa_to_dfa.accept_states {
            let s = mapping.get(&s).unwrap();
            accept_states.insert(*s);
        }

        Dfa {
            states,
            transitions,
            accept_states,
            start_text: nfa_to_dfa.nfa.is_start_text(),
            end_text: nfa_to_dfa.nfa.is_end_text(),
        }
    }
}
