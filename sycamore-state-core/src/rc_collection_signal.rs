use std::{ops::Deref, rc::Rc};
use sycamore::prelude::*;

/// Wrapper type for [`RcSignal`](RcSignal)<[`Vec`](Vec)<[`RcSignal`](RcSignal)<`T`>>>
///
/// Base type for the [`#[collection]`](crate::State) attribute when using [`State`](crate::State) derive macro
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RcCollectionSignal<T> {
    inner: RcSignal<Vec<RcSignal<T>>>,
}

impl<T> RcCollectionSignal<T> {
    /// Create new RcCollection from an iterator
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
    /// Push new value into collection
    ///
    /// ```rust
    /// # use sycamore_state_core::RcCollectionSignal;
    /// let collection = RcCollectionSignal::new(vec![1,2,3,4]);
    /// collection.push(8);
    /// # assert_eq!(collection.get().len(), 5);

    ///```
    pub fn push(&self, value: T) {
        self.inner.modify().push(create_rc_signal(value));
    }

    /// Get position of value in collection
    ///
    /// ```rust
    /// # use sycamore_state_core::RcCollectionSignal;
    /// let collection = RcCollectionSignal::new(vec![1,2,3,4]);
    /// let value = collection.find(|a| *a == 3);
    /// # value.expect("found item");
    ///
    ///```
    pub fn position<F: Fn(&T) -> bool>(&self, f: F) -> Option<usize> {
        self.inner.get().iter().position(|a| f(&a.get()))
    }

    /// Find value in collection
    ///
    /// ```rust
    /// # use sycamore_state_core::RcCollectionSignal;
    /// let collection = RcCollectionSignal::new(vec![1,2,3,4]);
    /// let value = collection.find(|a| *a == 3);
    /// # value.expect("found item");
    ///
    ///```
    pub fn find<F: Fn(&T) -> bool>(&self, f: F) -> Option<Rc<T>> {
        self.inner
            .get()
            .iter()
            .find(|a| f(&a.get()))
            .and_then(|a| Some(a.get()))
    }

    /// Remove value from collection by index
    ///
    /// ```rust
    /// # use sycamore_state_core::RcCollectionSignal;
    /// let collection = RcCollectionSignal::new(vec![1,2,3,4]);
    /// let value = collection.remove(2);
    /// # assert_eq!(collection.get().len(), 3);
    ///```
    pub fn remove(&self, index: usize) -> Rc<T> {
        self.inner.modify().remove(index).get()
    }

    /// Remove value from collection with predicate
    ///
    /// ```rust
    /// # use sycamore_state_core::RcCollectionSignal;
    /// let collection = RcCollectionSignal::new(vec![1,2,3,4]);
    /// let value = collection.remove_where(|a| *a == 3);
    /// # value.expect("found item");
    /// # assert_eq!(collection.get().len(), 3);
    ///```
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
