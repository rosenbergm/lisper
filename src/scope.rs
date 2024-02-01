//! Scope and environment handling

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::expr::Expr;

/// Holds information on any stored variables and functions
#[derive(Debug, PartialEq)]
pub struct Scope {
    /// Stores all defined variables and functions
    entities: HashMap<String, Expr>,
    /// Reference to parent scope, eg. when calling a function
    parent: Option<PassableScope>,
}

impl Scope {
    /// Create a new scope with defaults
    pub fn new() -> Self {
        Scope {
            entities: HashMap::new(),
            parent: None,
        }
    }

    /// Wraps a `Scope` object to be easily passable
    pub fn wrap(self) -> PassableScope {
        Rc::new(RefCell::new(self))
    }

    /// Creates a new `Scope` while setting `from` to be the parent
    pub fn extend(from: PassableScope) -> PassableScope {
        Scope {
            entities: HashMap::new(),
            parent: Some(from),
        }
        .wrap()
    }

    /// Set a value in a `Scope`
    pub fn set(&mut self, key: String, value: Expr) {
        self.entities.insert(key, value);
    }

    /// Gets a value from a `Scope`
    pub fn get(&self, key: &str) -> Option<Expr> {
        match self.entities.get(key) {
            Some(value) => Some(value.clone()),
            None => self
                .parent
                .as_ref()
                .and_then(|parent| parent.borrow().get(key)),
        }
    }
}

pub type PassableScope = Rc<RefCell<Scope>>;
