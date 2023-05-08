use inner_macros::State;
use sycamore::reactive::create_scope;

#[derive(State)]
pub enum SimpleEnum {
    Variant1(String),
    Variant2(i32),
}

#[derive(State)]
pub struct InnerState {
    pub field1: String,
    pub field2: i32,
}

#[derive(State)]
pub enum StatefulEnum {
    #[stateful]
    Variant1(InnerState),
}

#[derive(State)]
pub enum CollectionEnum {
    #[collection]
    Variant1(Vec<String>),
}

#[derive(State)]
pub enum StatefulCollectionEnum {
    #[stateful]
    #[collection]
    Variant1(Vec<InnerState>),
}

#[test]
fn basic_enum_test() {
    _ = create_scope(|cx| {
        let ref_state = RefStatefulCollectionEnum::new(
            cx,
            StatefulCollectionEnum::Variant1(vec![InnerState {
                field1: "Hello".into(),
                field2: 5,
            }]),
        );

        match ref_state {
            RefStatefulCollectionEnum::Variant1(data) => {
                data.push_value(
                    cx,
                    RefInnerState::new(
                        cx,
                        InnerState {
                            field1: "prova".into(),
                            field2: 2,
                        },
                    ),
                );
                data.push_deferred(cx, |cx| {
                    RefInnerState::new(
                        cx,
                        InnerState {
                            field1: "prova".into(),
                            field2: 2,
                        },
                    )
                });
            }
        }
    });
}
