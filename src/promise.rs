use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;
use crate::vm::Fiber;

/// Promise internal state
#[derive(Debug, Clone, PartialEq)]
pub enum PromiseState {
    Pending,
    Fulfilled(Value),
    Rejected(Value),
}

pub enum Reaction {
    /// Resumes a suspended fiber
    ResumeFiber(Rc<RefCell<Fiber>>),
    /// Standard callback (Chen Lang function or Native function)
    Callback {
        on_fulfilled: Option<Value>,
        on_rejected: Option<Value>,
        next_promise: Rc<RefCell<Promise>>,
    },
    Finally {
        on_finally: Value,
        next_promise: Rc<RefCell<Promise>>,
    },
}

/// Promise object structure
pub struct Promise {
    pub state: PromiseState,
    pub reactions: Vec<Reaction>,
    pub is_handled: bool,
}

impl Default for Promise {
    fn default() -> Self {
        Self::new()
    }
}

impl Promise {
    pub fn new() -> Self {
        Promise {
            state: PromiseState::Pending,
            reactions: Vec::new(),
            is_handled: false,
        }
    }

    /// Create a resolved promise
    pub fn resolve_static(value: Value) -> Value {
        let p = Rc::new(RefCell::new(Promise {
            state: PromiseState::Fulfilled(value),
            reactions: Vec::new(),
            is_handled: false,
        }));
        Value::Promise(p)
    }

    /// Create a rejected promise
    pub fn reject_static(reason: Value) -> Value {
        let p = Rc::new(RefCell::new(Promise {
            state: PromiseState::Rejected(reason),
            reactions: Vec::new(),
            is_handled: false,
        }));
        Value::Promise(p)
    }

    pub fn resolve(&mut self, value: Value) -> Vec<Reaction> {
        if let PromiseState::Pending = self.state {
            self.state = PromiseState::Fulfilled(value);
            std::mem::take(&mut self.reactions)
        } else {
            Vec::new()
        }
    }

    pub fn reject(&mut self, reason: Value) -> Vec<Reaction> {
        if let PromiseState::Pending = self.state {
            self.state = PromiseState::Rejected(reason);
            std::mem::take(&mut self.reactions)
        } else {
            Vec::new()
        }
    }

    pub fn settle(&mut self, state: PromiseState) -> Vec<Reaction> {
        if let PromiseState::Pending = self.state {
            self.state = state;
            std::mem::take(&mut self.reactions)
        } else {
            Vec::new()
        }
    }

    pub fn add_reaction(&mut self, reaction: Reaction) {
        match self.state {
            PromiseState::Pending => {
                self.reactions.push(reaction);
            }
            _ => {
                // Promise already settled, reaction should be scheduled immediately
                // In this implementation, we expect the caller to handle this
                self.reactions.push(reaction);
            }
        }
    }

    pub fn then(&mut self, on_fulfilled: Option<Value>, on_rejected: Option<Value>) -> Value {
        let next_promise = Rc::new(RefCell::new(Promise::new()));
        let reaction = Reaction::Callback {
            on_fulfilled,
            on_rejected,
            next_promise: next_promise.clone(),
        };
        self.add_reaction(reaction);
        Value::Promise(next_promise)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_promise_basic_state() {
        let mut p = Promise::new();
        assert_eq!(p.state, PromiseState::Pending);

        let reactions = p.resolve(Value::int(42));
        assert_eq!(p.state, PromiseState::Fulfilled(Value::int(42)));
        assert_eq!(reactions.len(), 0);
    }
}
