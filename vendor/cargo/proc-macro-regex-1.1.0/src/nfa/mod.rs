mod repetition;

use crate::{
    character::{Character, CharacterError},
    macro_input::MacroInput,
};
use regex_syntax::{
    hir::{Anchor, Class, Group, Hir, HirKind, Literal, WordBoundary},
    ParserBuilder,
};
use std::{
    cmp::max,
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};
use syn::{Error as SynError, Result as SynResult};
use thiserror::Error;

pub const START_STATE: usize = 0;

type Transition<T> = BTreeMap<usize, BTreeMap<T, BTreeSet<usize>>>;

pub type NFAResult<T> = Result<T, NFAError>;

fn to_hir<T>(input: &MacroInput) -> SynResult<Hir>
where
    T: Character,
{
    let mut parser = ParserBuilder::new()
        .unicode(T::unicode())
        .allow_invalid_utf8(T::allow_invalid_utf8())
        .build();
    match parser.parse(&input.get_regex()) {
        Ok(hir) => Ok(hir),
        Err(e) => Err(SynError::new(
            input.get_regex_span(),
            format!("Could not parse regex: {:?}", e),
        )),
    }
}

#[derive(Debug, Error)]
pub enum NFAError {
    #[error("alternation has zero lenght")]
    AlternationZeroLen,
    #[error("CharacterError: {0}")]
    CharacterError(#[from] CharacterError),
    #[error("Start text was not at the beginning of the regex")]
    StartTextError,
    #[error("End text was not at the end of the text")]
    EndTextError,
}

#[derive(Debug)]
pub struct Nfa<T>
where
    T: Character + Copy,
{
    states: BTreeSet<usize>,
    transitions: Transition<T>,
    accept_states: BTreeSet<usize>,
    state_count: usize,
    start_text: bool,
    end_text: bool,
}

impl<T> Nfa<T>
where
    T: Character + Copy,
{
    fn add_transition(
        transitions: &mut Transition<T>,
        source_state: usize,
        character: T,
        target_state: usize,
    ) {
        if let Some(characters_to_targets) = transitions.get_mut(&source_state) {
            if let Some(targets) = characters_to_targets.get_mut(&character) {
                targets.insert(target_state);
            } else {
                let mut targets = BTreeSet::new();
                targets.insert(target_state);

                characters_to_targets.insert(character, targets);
            }
        } else {
            let mut targets = BTreeSet::new();
            targets.insert(target_state);

            let mut characters_to_targets = BTreeMap::new();
            characters_to_targets.insert(character, targets);

            transitions.insert(source_state, characters_to_targets);
        }
    }

    fn extend_transitions(d: &mut Transition<T>, s: &Transition<T>) {
        for (new_source_state, new_characters_to_targets) in s.iter() {
            if let Some(characters_to_targets) = d.get_mut(new_source_state) {
                for (new_character, new_targets) in new_characters_to_targets.iter() {
                    if let Some(targets) = characters_to_targets.get_mut(new_character) {
                        for new_target_state in new_targets {
                            targets.insert(*new_target_state);
                        }
                    } else {
                        characters_to_targets.insert(*new_character, new_targets.clone());
                    }
                }
            } else {
                d.insert(*new_source_state, new_characters_to_targets.clone());
            }
        }
    }

    fn add_state(&mut self, new_state: usize) -> NFAResult<()> {
        if self.end_text {
            return Err(NFAError::EndTextError);
        }

        let assert = self.states.insert(new_state);
        debug_assert!(assert);
        self.state_count = max(new_state, self.state_count);
        Ok(())
    }

    fn append_states(&mut self, nfa: &Nfa<T>) -> NFAResult<()> {
        self.set_start_text(nfa.start_text)?;
        if !nfa.states.is_empty() {
            if self.end_text {
                return Err(NFAError::EndTextError);
            }

            for new_state in nfa.states.iter() {
                let assert = self.states.insert(*new_state);
                debug_assert!(assert);
                self.state_count = max(*new_state, self.state_count);
            }
            Nfa::extend_transitions(&mut self.transitions, &nfa.transitions);
        }
        self.end_text = nfa.end_text;
        Ok(())
    }

    fn next_state_count(&mut self) -> usize {
        self.state_count += 1;
        self.state_count
    }

    fn next_state(&mut self) -> NFAResult<usize> {
        let new_state = self.next_state_count();
        self.add_state(new_state)?;
        Ok(new_state)
    }

    fn sub(&mut self, hir: Hir) -> NFAResult<Nfa<T>> {
        let mut nfa = Nfa {
            states: BTreeSet::new(),
            transitions: Transition::new(),
            accept_states: self.accept_states.clone(),
            state_count: self.next_state_count(),
            start_text: false,
            end_text: self.end_text,
        };
        nfa.hir(hir)?;
        Ok(nfa)
    }

    fn new() -> Nfa<T> {
        let mut states = BTreeSet::new();
        states.insert(START_STATE);

        Nfa {
            states: states.clone(),
            transitions: Transition::new(),
            accept_states: states,
            state_count: START_STATE,
            start_text: false,
            end_text: false,
        }
    }

    fn char(&mut self, c: T) -> NFAResult<()> {
        let state = self.next_state()?;
        for s in self.accept_states.iter() {
            Nfa::add_transition(&mut self.transitions, *s, c, state);
        }
        self.accept_states = BTreeSet::new();
        self.accept_states.insert(state);
        Ok(())
    }

    fn literal(&mut self, literal: Literal) -> NFAResult<()> {
        let c = T::from_literal(literal)?;
        self.char(c)
    }

    fn class(&mut self, class: Class) -> NFAResult<()> {
        let state = self.next_state()?;
        let cs = T::from_class(class)?;
        for c in cs {
            for s in &self.accept_states {
                Nfa::add_transition(&mut self.transitions, *s, c, state);
            }
        }
        self.accept_states = BTreeSet::new();
        self.accept_states.insert(state);
        Ok(())
    }

    fn alternation(&mut self, alternation: Vec<Hir>) -> NFAResult<()> {
        if alternation.is_empty() {
            return Err(NFAError::AlternationZeroLen);
        }

        let mut accept_states = BTreeSet::new();
        for hir in alternation {
            let nfa = self.sub(hir)?;
            self.append_states(&nfa)?;
            accept_states.extend(nfa.accept_states)
        }
        self.accept_states = accept_states;
        Ok(())
    }

    fn conact(&mut self, concat: Vec<Hir>) -> NFAResult<()> {
        for hir in concat {
            self.hir(hir)?;
        }
        Ok(())
    }

    fn group(&mut self, group: Group) -> NFAResult<()> {
        self.hir(*group.hir)
    }

    fn word_boundary(&mut self, _word_boundary: WordBoundary) -> NFAResult<()> {
        unimplemented!();
    }

    fn set_start_text(&mut self, start_text: bool) -> NFAResult<()> {
        if start_text {
            if self.state_count == 0 {
                self.start_text = true;
            } else {
                return Err(NFAError::StartTextError);
            }
        }

        Ok(())
    }

    fn anchor(&mut self, anchor: Anchor) -> NFAResult<()> {
        match anchor {
            Anchor::StartLine => self.char(T::new_line()),
            Anchor::EndLine => self.char(T::new_line()),
            Anchor::StartText => self.set_start_text(true),
            Anchor::EndText => {
                self.end_text = true;
                Ok(())
            }
        }
    }

    fn hir(&mut self, hir: Hir) -> NFAResult<()> {
        match hir.into_kind() {
            HirKind::Empty => Ok(()),
            HirKind::Literal(literal) => self.literal(literal),
            HirKind::Class(class) => self.class(class),
            HirKind::Alternation(alternation) => self.alternation(alternation),
            HirKind::Concat(concat) => self.conact(concat),
            HirKind::Repetition(repetition) => self.repetition(repetition),
            HirKind::Group(group) => self.group(group),
            HirKind::WordBoundary(word_boundary) => self.word_boundary(word_boundary),
            HirKind::Anchor(anchor) => self.anchor(anchor),
        }
    }

    pub fn chars(&self, source_state: usize, characters: &mut BTreeSet<T>) {
        if let Some(characters_to_targets) = self.transitions.get(&source_state) {
            characters.extend(characters_to_targets.keys());
        }
    }

    pub fn simulate(&self, source_state: usize, character: T, new_targets: &mut BTreeSet<usize>) {
        if let Some(characters_to_targets) = self.transitions.get(&source_state) {
            if let Some(targets) = characters_to_targets.get(&character) {
                for target_state in targets {
                    new_targets.insert(*target_state);
                }
            }
        }
    }

    pub fn is_accept_state(&self, state: usize) -> bool {
        self.accept_states.contains(&state)
    }

    pub fn is_start_text(&self) -> bool {
        self.start_text
    }

    pub fn is_end_text(&self) -> bool {
        self.end_text
    }
}

impl<T> TryFrom<&MacroInput> for Nfa<T>
where
    T: Character + Copy,
{
    type Error = SynError;

    fn try_from(input: &MacroInput) -> SynResult<Self> {
        let hir = to_hir::<T>(input)?;

        let mut nfa = Nfa::new();
        match nfa.hir(hir) {
            Ok(_) => Ok(nfa),
            Err(e) => Err(SynError::new(
                input.get_regex_span(),
                format!("Error create the NFA: {:?}", e),
            )),
        }
    }
}
