# Derive macro for State Management
## Overview

sycamore-state is a utility library for better state management using sycamore's reactive primitives

the main features of this crate are the [`State`](State) derive macro and Rc/Ref Collection signal types

currently for lifetime management this crate uses widely [`sycamore::reactive::create_signal_unsafe`],\n
if you think there are possible unsafe errors feel free to open an [`issue`](https://github.com/ChristianBelloni/sycamore-state/issues)


## Usage
```rust
#[derive(Debug, State, Clone)]
#[state(clone, eq, debug)] // avaliable derive macros are: (clone, debug, eq, ord)
pub struct MyState<'a> {
    pub field_1: String,
    pub field_2: u32,
    #[state]
    pub field_3: MyInnerState<'a>,
    #[state]
    #[collection]
    pub state_collection: Vec<MyInnerState<'a>>
}

#[derive(Debug, State, Clone)]
#[state(clone, eq, debug)]
pub struct MyInnerState<'a> {
    pub field_1: i64,
    #[collection]
    pub collection: Vec<&'a str>
}


let ref_state = RefMyState::new(cx, MyState {
    field_1: "my_string".into(),
    field_2: 5,
    field_3: MyInnerState {
        field_1: 20,
        collection: vec!["my", "string", "collection"],
    },
    state_collection: Default::default()
});  
```
## Generated Structs
```
pub struct RcMyState<'a> {
    pub field_1: RcSignal<String>,
    pub field_2: RcSignal<u32>,
    pub field_3: RcSignal<RcMyInnerState<'a>>,
    pub state_collection: RcCollectionSignal<RcMyInnerState<'a>>,
}

pub struct RcMyInnerState<'a> {
    pub field_1: RcSignal<i64>,
    pub collection: RcCollectionSignal<&'a str>,
}

pub struct RefMyState<'a, 'stateful> {
    pub field_1: &'stateful Signal<String>,
    pub field_2: &'stateful Signal<u32>,
    pub field_3: &'stateful Signal<RefMyInnerState<'a, 'stateful>>,
    pub state_collection: RefCollectionSignal<'stateful, RefMyInnerState<'a, 'stateful>>,
}

pub struct RefMyInnerState<'a, 'stateful> {
    pub field_1: &'stateful Signal<i64>,
    pub collection: RefCollectionSignal<'stateful, &'a str>,
}
```