use crate::{
    character::Character,
    nfa::{NFAResult, Nfa},
};
use regex_syntax::hir::{Hir, Repetition, RepetitionKind, RepetitionRange};
use std::collections::{BTreeMap, BTreeSet};

impl<T> Nfa<T>
where
    T: Character + Copy,
{
    fn repetition_range_exactly(&mut self, hir: Hir, exactly: u32) -> NFAResult<()> {
        for _ in 0..exactly {
            let nfa = self.sub(hir.clone())?;
            self.append_states(&nfa)?;
            self.accept_states = nfa.accept_states;
        }
        Ok(())
    }

    fn repetition_range_at_least(&mut self, hir: Hir, at_least: u32) -> NFAResult<()> {
        for _ in 0..at_least {
            let nfa = self.sub(hir.clone())?;
            self.append_states(&nfa)?;
            self.accept_states = nfa.accept_states;
        }
        self.repetition_zero_or_more(hir)
    }

    fn repetition_range_bounded(&mut self, hir: Hir, m: u32, n: u32) -> NFAResult<()> {
        if m != 0 {
            self.repetition_range_exactly(hir.clone(), m)?;
        }

        let mut accept_states = self.accept_states.clone();
        for _ in m..n {
            let nfa = self.sub(hir.clone())?;
            self.append_states(&nfa)?;
            accept_states.extend(nfa.accept_states.clone());
            self.accept_states = nfa.accept_states;
        }
        self.accept_states = accept_states;

        Ok(())
    }

    fn repetition_range(&mut self, hir: Hir, repetition_range: RepetitionRange) -> NFAResult<()> {
        match repetition_range {
            RepetitionRange::Exactly(exactly) => self.repetition_range_exactly(hir, exactly),
            RepetitionRange::AtLeast(at_least) => self.repetition_range_at_least(hir, at_least),
            RepetitionRange::Bounded(m, n) => self.repetition_range_bounded(hir, m, n),
        }
    }

    fn repetition_zero_or_one(&mut self, hir: Hir) -> NFAResult<()> {
        let nfa = self.sub(hir)?;
        self.append_states(&nfa)?;
        self.accept_states.extend(nfa.accept_states);
        Ok(())
    }

    fn repetition_zero_or_more(&mut self, hir: Hir) -> NFAResult<()> {
        let nfa = self.sub(hir)?;
        for state in nfa.states {
            if !nfa.accept_states.contains(&state) {
                self.add_state(state)?;
            }
        }

        for (source_state, characters_to_targets) in nfa.transitions {
            for (character, targets) in characters_to_targets {
                for target_state in targets {
                    let s_accept = nfa.accept_states.contains(&source_state);
                    let t_accept = nfa.accept_states.contains(&target_state);
                    match (s_accept, t_accept) {
                        (true, true) => {
                            for source_state in self.accept_states.iter() {
                                for target_state in self.accept_states.iter() {
                                    Nfa::add_transition(
                                        &mut self.transitions,
                                        *target_state,
                                        character,
                                        *source_state,
                                    );
                                }
                            }
                        }
                        (true, false) => {
                            for source_state in self.accept_states.iter() {
                                Nfa::add_transition(
                                    &mut self.transitions,
                                    *source_state,
                                    character,
                                    target_state,
                                );
                            }
                        }
                        (false, true) => {
                            for target_state in self.accept_states.iter() {
                                Nfa::add_transition(
                                    &mut self.transitions,
                                    source_state,
                                    character,
                                    *target_state,
                                );
                            }
                        }
                        (false, false) => {
                            Nfa::add_transition(
                                &mut self.transitions,
                                source_state,
                                character,
                                target_state,
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn repetition_one_or_more(&mut self, hir: Hir) -> NFAResult<()> {
        let mut nfa = self.sub(hir)?;
        let mut backwards_characters_to_targets: BTreeMap<T, BTreeSet<usize>> = BTreeMap::new();
        for accept_state in self.accept_states.iter() {
            if let Some(characters_to_targets) = nfa.transitions.get(accept_state) {
                for (character, targets) in characters_to_targets.iter() {
                    if let Some(backwards_targets) =
                        backwards_characters_to_targets.get_mut(character)
                    {
                        for target in targets {
                            backwards_targets.insert(*target);
                        }
                    } else {
                        backwards_characters_to_targets.insert(*character, targets.clone());
                    }
                }
            }
        }

        for (character, targets) in backwards_characters_to_targets {
            for target_state in targets {
                for accept_state in nfa.accept_states.iter() {
                    Nfa::add_transition(
                        &mut nfa.transitions,
                        *accept_state,
                        character,
                        target_state,
                    );
                }
            }
        }

        self.append_states(&nfa)?;
        self.accept_states = nfa.accept_states;
        Ok(())
    }

    pub(super) fn repetition(&mut self, repetition: Repetition) -> NFAResult<()> {
        match repetition.kind {
            RepetitionKind::ZeroOrOne => self.repetition_zero_or_one(*repetition.hir),
            RepetitionKind::ZeroOrMore => self.repetition_zero_or_more(*repetition.hir),
            RepetitionKind::OneOrMore => self.repetition_one_or_more(*repetition.hir),
            RepetitionKind::Range(repetition_range) => {
                self.repetition_range(*repetition.hir, repetition_range)
            }
        }
    }
}
