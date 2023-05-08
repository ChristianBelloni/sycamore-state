use std::{ops::Deref, rc::Rc};
use sycamore::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RcCollectionSignal<T> {
    inner: RcSignal<Vec<RcSignal<T>>>,
}

impl<T> RcCollectionSignal<T> {
    pub fn new(inner: impl IntoIterator<Item = T>) -> RcCollectionSignal<T> {
        let collected = inner
            .into_iter()
            .map(|a| create_rc_signal(a))
            .collect::<Vec<_>>();
        RcCollectionSignal {
            inner: create_rc_signal(collected),
        }
    }
}

impl<T> RcCollectionSignal<T> {
    pub fn push(&self, value: T) {
        self.inner.modify().push(create_rc_signal(value));
    }

    pub fn position<F: Fn(&T) -> bool>(&self, f: F) -> Option<usize> {
        self.inner.get().iter().position(|a| f(&a.get()))
    }

    pub fn find<F: Fn(&T) -> bool>(&self, f: F) -> Option<Rc<T>> {
        self.inner
            .get()
            .iter()
            .find(|a| f(&a.get()))
            .and_then(|a| Some(a.get()))
    }

    pub fn remove(&self, index: usize) -> Rc<T> {
        self.inner.modify().remove(index).get()
    }

    pub fn remove_where<F: Fn(&T) -> bool>(&self, f: F) -> Option<Rc<T>> {
        if let Some(index) = self.position(f) {
            Some(self.remove(index))
        } else {
            None
        }
    }
}

impl<T> Deref for RcCollectionSignal<T> {
    type Target = Signal<Vec<RcSignal<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
