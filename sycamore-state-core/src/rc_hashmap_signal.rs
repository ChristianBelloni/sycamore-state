use std::{collections::HashMap, hash::Hash, ops::Deref, rc::Rc};

use sycamore::reactive::{create_rc_signal, RcSignal};

pub struct RcHashMapSignal<'a, K, V> {
    inner: RcSignal<HashMap<K, RcHashMapItem<'a, V>>>,
}

#[derive(Clone)]
pub struct RcHashMapItem<'a, T> {
    inner: RcSignal<T>,
    remover: Rc<Box<dyn Fn() + 'a>>,
}

impl<'a, T> RcHashMapItem<'a, T> {
    pub fn remove(&self) {
        (self.remover)()
    }
}

impl<'a, T> Deref for RcHashMapItem<'a, T> {
    type Target = RcSignal<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> RcHashMapItem<'a, T> {
    fn new(inner: T, remover: impl Fn() + 'a) -> Self {
        Self {
            inner: create_rc_signal(inner),
            remover: Rc::new(Box::new(remover)),
        }
    }
}

impl<'a, K, V> Deref for RcHashMapSignal<'a, K, V> {
    type Target = RcSignal<HashMap<K, RcHashMapItem<'a, V>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, K: Hash + Eq + Send + Sync + Clone + 'a, V: Send + Sync + Clone + 'a>
    RcHashMapSignal<'a, K, V>
{
    pub fn new(map: HashMap<K, V>) -> Self {
        let inner = HashMap::new();
        let inner = create_rc_signal(inner);
        for (k, v) in map {
            let inner_clone = inner.clone();
            let _k = k.clone();
            let remover = move || {
                inner_clone.modify().remove(&k);
            };
            let item = RcHashMapItem::new(v, remover);
            inner.modify().insert(_k, item);
        }
        Self { inner }
    }

    pub fn insert(&self, key: K, value: V) {
        let clone = self.inner.clone();
        self.inner.modify().insert(
            key.clone(),
            RcHashMapItem::new(value, move || {
                clone.modify().remove(&key);
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap};

    use crate::RcHashMapSignal;

    #[test]
    pub fn hashmap_test() {
        let map = HashMap::default();
        let map = RcHashMapSignal::new(map);
        map.insert("hello", 3);
        map.insert("hello2", 5);
        map.insert("hello4", 7);
        map.insert("hello6", 8);
        map.insert("hello7", 59);
        assert_eq!(map.get().len(), 5);
        for (_k, v) in map.get().iter() {
            v.remove();
        }
        assert_eq!(map.get().len(), 0);
    }
}
