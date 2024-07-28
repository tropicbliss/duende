use std::cell::{Cell, RefCell};

pub struct MutCell<T> {
    value: RefCell<T>,
    has_changed: Cell<bool>,
}

impl<T> MutCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: RefCell::new(value),
            has_changed: Cell::new(true),
        }
    }

    pub fn set(&self, value: T) {
        self.value.replace(value);
        self.has_changed.set(true);
    }

    pub fn execute_on_change<F>(&self, mut closure: F)
    where
        F: FnMut(&T),
    {
        if self.has_changed.get() {
            closure(&self.value.borrow());
            self.has_changed.set(false);
        }
    }
}
