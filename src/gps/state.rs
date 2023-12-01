use std::cmp::Ordering;
use std::collections::HashMap;

use super::condition::{Condition, ConditionImpl};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    name: String,
    data: StateData,
}

#[derive(Debug, Clone, Eq, Hash)]
pub enum StateData {
    Symbol,
    Integer(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateSet {
    states: HashMap<String, StateData>,
}

impl State {
    pub fn new(name: String, data: StateData) -> Self {
        Self { name, data }
    }

    pub fn new_symbol(name: String) -> Self {
        Self {
            name,
            data: StateData::Symbol,
        }
    }

    pub fn new_integer(name: String, data: i32) -> Self {
        Self {
            name,
            data: StateData::Integer(data),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data(&self) -> &StateData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut StateData {
        &mut self.data
    }

    pub fn into_inner(self) -> (String, StateData) {
        (self.name, self.data)
    }
}

impl PartialEq for StateData {
    fn eq(&self, other: &Self) -> bool {
        match self {
            StateData::Symbol => matches!(other, StateData::Symbol),
            StateData::Integer(x) => match other {
                StateData::Integer(y) => x == y,
                _ => false,
            },
        }
    }
}

impl PartialOrd for StateData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            StateData::Symbol => match other {
                StateData::Symbol => Some(Ordering::Equal),
                _ => None,
            },
            StateData::Integer(x) => match other {
                StateData::Integer(y) => x.partial_cmp(y),
                _ => None,
            },
        }
    }
}

impl StateSet {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    pub fn insert(&mut self, state: State) -> bool {
        let (name, data) = state.into_inner();
        self.states.insert(name, data).is_none()
    }

    pub fn remove(&mut self, name: &str) -> Option<StateData> {
        self.states.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<&StateData> {
        self.states.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut StateData> {
        self.states.get_mut(name)
    }

    pub fn contains(&self, state: &State) -> bool {
        match self.states.get(state.name()) {
            Some(data) => data == state.data(),
            None => false,
        }
    }
    pub fn has_reached(&self, goals: &Vec<ConditionImpl>) -> bool {
        // Test if the goal state is a subset of current states.
        goals.iter().all(|condition| condition.check(self))
    }
}
