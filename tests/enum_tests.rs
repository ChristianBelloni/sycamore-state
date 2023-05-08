use sycamore::reactive::{create_effect, create_scope, create_signal};
use sycamore_state_macros::State;

#[derive(State, Clone)]
#[state(clone)]
pub enum SimpleEnum {
    Variant1(String),
    Variant2(i32),
}

#[derive(State, Clone)]
#[state(clone)]
pub struct InnerState {
    pub field1: String,
    pub field2: i32,
}

#[derive(State, Clone)]
#[state(clone)]
pub enum StatefulEnum {
    #[state]
    Variant1(InnerState),
}

#[derive(State)]
#[state(clone)]
pub enum CollectionEnum {
    #[collection]
    Variant1(Vec<String>),
}

#[derive(State, Clone)]
#[state(clone)]
pub enum StatefulCollectionEnum {
    #[state]
    #[collection]
    Variant1(Vec<InnerState>),
}

#[test]
fn simple_enum_test() {
    _ = create_scope(|cx| {
        let ref_simple_enum = RefSimpleEnum::new(cx, SimpleEnum::Variant1("test".into()));
        let rc_simple_enum = RcSimpleEnum::new(SimpleEnum::Variant2(15));
        let changes_counter = create_signal(cx, 0);
        let ref_clone = ref_simple_enum.clone();
        let rc_clone = rc_simple_enum.clone();
        create_effect(cx, move || {
            match &ref_clone {
                RefSimpleEnum::Variant1(data) => {
                    _ = data.get();
                }
                RefSimpleEnum::Variant2(data) => {
                    _ = data.get();
                }
            };
            match &rc_clone {
                RcSimpleEnum::Variant1(data) => {
                    _ = data.get();
                }
                RcSimpleEnum::Variant2(data) => {
                    _ = data.get();
                }
            };
            *changes_counter.modify() += 1;
        });

        let inner_ref = match &ref_simple_enum {
            RefSimpleEnum::Variant1(data) => data.get(),
            _ => unreachable!(),
        };

        assert_eq!(inner_ref.as_str(), "test");

        let inner_rc = match &rc_simple_enum {
            RcSimpleEnum::Variant1(_) => unreachable!(),
            RcSimpleEnum::Variant2(data) => data.get(),
        };

        assert_eq!(*inner_rc, 15);

        match &ref_simple_enum {
            RefSimpleEnum::Variant1(data) => data.set("test2".into()),
            RefSimpleEnum::Variant2(_) => unreachable!(),
        }

        let inner_ref = match &ref_simple_enum {
            RefSimpleEnum::Variant1(data) => data.get(),
            _ => unreachable!(),
        };

        assert_eq!(inner_ref.as_str(), "test2");

        match &rc_simple_enum {
            RcSimpleEnum::Variant1(_) => unreachable!(),
            RcSimpleEnum::Variant2(data) => data.set(20),
        }

        let inner_rc = match &rc_simple_enum {
            RcSimpleEnum::Variant1(_) => unreachable!(),
            RcSimpleEnum::Variant2(data) => data.get(),
        };

        assert_eq!(*inner_rc, 20);
        let changes = changes_counter.get();

        assert_eq!(*changes, 3)
    });
}

#[test]
fn stateful_enum_test() {
    _ = create_scope(|cx| {
        let ref_simple_enum = RefStatefulEnum::new(
            cx,
            StatefulEnum::Variant1(InnerState {
                field1: "test".into(),
                field2: 15,
            }),
        );
        let rc_simple_enum = RcStatefulEnum::new(StatefulEnum::Variant1(InnerState {
            field1: "test".into(),
            field2: 15,
        }));
        let changes_counter = create_signal(cx, 0);
        let ref_clone = ref_simple_enum.clone();
        let rc_clone = rc_simple_enum.clone();
        create_effect(cx, move || {
            match &ref_clone {
                RefStatefulEnum::Variant1(data) => {
                    let inner = data.get();
                    _ = inner.field1.get();
                    _ = inner.field2.get();
                }
            };
            match &rc_clone {
                RcStatefulEnum::Variant1(data) => {
                    let inner = data.get();
                    _ = inner.field1.get();
                    _ = inner.field2.get();
                }
            };
            *changes_counter.modify() += 1;
        });

        let inner_ref = match &ref_simple_enum {
            RefStatefulEnum::Variant1(data) => data.get(),
        };

        assert_eq!(inner_ref.field1.get().as_str(), "test");

        let inner_rc = match &rc_simple_enum {
            RcStatefulEnum::Variant1(data) => data,
        };

        assert_eq!(*inner_rc.get().field2.get(), 15);

        match &ref_simple_enum {
            RefStatefulEnum::Variant1(data) => data.get().field1.set("test2".into()),
        }

        let inner_ref = match &ref_simple_enum {
            RefStatefulEnum::Variant1(data) => data.get(),
        };

        assert_eq!(inner_ref.field1.get().as_str(), "test2");

        match &rc_simple_enum {
            RcStatefulEnum::Variant1(data) => data.get().field2.set(20),
        }

        let inner_rc = match &rc_simple_enum {
            RcStatefulEnum::Variant1(data) => data.get(),
        };

        assert_eq!(*inner_rc.field2.get(), 20);
        let changes = changes_counter.get();

        assert_eq!(*changes, 3)
    });
}
