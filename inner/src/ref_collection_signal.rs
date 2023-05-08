use std::{ops::Deref, rc::Rc};
use sycamore::prelude::*;

pub trait State {}

#[derive(Debug, PartialEq, Eq)]
pub struct RefCollectionSignal<'a, T> {
    inner: &'a Signal<Vec<&'a Signal<T>>>,
}

impl<'a, T> Clone for RefCollectionSignal<'a, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<'a, T> Copy for RefCollectionSignal<'a, T> {}

impl<T> RefCollectionSignal<'_, T> {
    pub fn new<'a>(cx: Scope<'a>, inner: impl IntoIterator<Item = T>) -> RefCollectionSignal<T>
    where
        T: 'a,
    {
        let collected = inner
            .into_iter()
            .map(|a| unsafe { create_signal_unsafe(cx, a) })
            .collect::<Vec<_>>();
        RefCollectionSignal::<'a, T> {
            inner: unsafe { create_signal_unsafe(cx, collected) },
        }
    }
}

impl<'a, T> RefCollectionSignal<'a, T> {
    pub fn push_value<'b>(&self, cx: Scope<'a>, value: T)
    where
        T: 'b,
        'b: 'a,
    {
        self.inner
            .modify()
            .push(unsafe { create_signal_unsafe(cx, value) });
    }

    pub fn push_deferred<F: Fn(Scope<'a>) -> T>(&self, cx: Scope<'a>, value: F)
    where
        T: 'a,
    {
        self.inner
            .modify()
            .push(unsafe { create_signal_unsafe(cx, value(cx)) });
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

    pub fn remove_where<'b, F: Fn(&T) -> bool>(&'b self, f: F) -> Option<Rc<T>> {
        if let Some(index) = self.position(f) {
            Some(self.remove(index))
        } else {
            None
        }
    }
}

impl<'a, T> Deref for RefCollectionSignal<'a, T> {
    type Target = &'a Signal<Vec<&'a Signal<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
