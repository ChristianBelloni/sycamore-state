use std::{ops::Deref, rc::Rc};
use sycamore::prelude::*;

/// Wrapper type for [`&Signal`](Signal)<[`Vec`](Vec)<[`&Signal`](Signal)<`T`>>>
///
/// Base type for the [`#[collection]`](crate::State) attribute when using [`State`](crate::State) derive macro
#[derive(Debug, PartialEq, Eq)]
pub struct RefCollectionSignal<'a, T> {
    inner: &'a Signal<Vec<&'a Signal<T>>>,
}

impl<'a, T> Clone for RefCollectionSignal<'a, T> {
    fn clone(&self) -> Self {
        Self {
            inner: <&Signal<Vec<&Signal<T>>>>::clone(&self.inner),
        }
    }
}

impl<'a, T> Copy for RefCollectionSignal<'a, T> {}

impl<T> RefCollectionSignal<'_, T> {
    /// Create new [`RefCollection`](RefCollectionSignal) from an iterator
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
    /// Push new value with associated [`scope`](Scope)
    pub fn push_value<'b>(&self, cx: Scope<'a>, value: T)
    where
        T: 'b,
        'b: 'a,
    {
        self.inner
            .modify()
            .push(unsafe { create_signal_unsafe(cx, value) });
    }

    /// Push new value with associated [`scope`](Scope) and a closure
    pub fn push_deferred<F: Fn(Scope<'a>) -> T>(&self, cx: Scope<'a>, value: F)
    where
        T: 'a,
    {
        self.inner
            .modify()
            .push(unsafe { create_signal_unsafe(cx, value(cx)) });
    }

    /// Get position of value in collection
    ///
    /// ```ignore
    /// # use inner::RefCollectionSignal;
    /// let collection = RefCollectionSignal::new(cx, vec![1,2,3,4]);
    /// let value = collection.find(|a| *a == 3);
    /// # value.expect("found item");
    ///
    ///```
    pub fn position<F: Fn(&T) -> bool>(&self, f: F) -> Option<usize> {
        self.inner.get().iter().position(|a| f(&a.get()))
    }

    /// Find value in collection
    ///
    /// ```ignore
    /// # use inner::RefCollectionSignal;
    /// let collection = RefCollectionSignal::new(cx, vec![1,2,3,4]);
    /// let value = collection.find(|a| *a == 3);
    /// # value.expect("found item");
    ///
    ///```
    pub fn find<F: Fn(&T) -> bool>(&self, f: F) -> Option<Rc<T>> {
        self.inner
            .get()
            .iter()
            .find(|a| f(&a.get()))
            .map(|a| a.get())
    }

    /// Remove value from collection by index
    ///
    /// ```ignore
    /// # use inner::RefCollectionSignal;
    /// let collection = RefCollectionSignal::new(cx, vec![1,2,3,4]);
    /// let value = collection.remove(2);
    /// # value.expect("found item");
    /// # assert_eq!(collection.get().len(), 3);
    ///```
    pub fn remove(&self, index: usize) -> Rc<T> {
        self.inner.modify().remove(index).get()
    }

    /// Remove value from collection with predicate
    ///
    /// ```ignore
    /// # use inner::RefCollectionSignal;
    /// let collection = RefCollectionSignal::new(cx, vec![1,2,3,4]);
    /// let value = collection.remove_where(|a| *a == 3);
    /// # value.expect("found item");
    /// # assert_eq!(collection.get().len(), 3);
    ///```
    pub fn remove_where<F: Fn(&T) -> bool>(&self, f: F) -> Option<Rc<T>> {
        self.position(f).map(|index| self.remove(index))
    }
}

impl<'a, T> Deref for RefCollectionSignal<'a, T> {
    type Target = &'a Signal<Vec<&'a Signal<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
