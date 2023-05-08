# Derive macro for State Management
## Overview
[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge][docs-url]]

[crates-badge]: https://img.shields.io/crates/v/sycamore-state-manager.svg
[crates-url]: https://crates.io/crates/sycamore-state-manager

[docs-url]: https://docs.rs/sycamore-state-manager/0.0.2/sycamore_state_manager/
[docs-badge]: https://img.shields.io/docsrs/sycamore-state-manager/latest

sycamore-state is a utility library for better state management using sycamore's reactive primitives

the main features of this crate are the `State` derive macro and the Rc/Ref Collection signal types

currently for lifetime management this crate uses widely `sycamore::reactive::create_signal_unsafe`
if you think there are possible unsafe errors feel free to open an [`issue`](https://github.com/ChristianBelloni/sycamore-state/issues)

## Current Features

 - [x] Support for Generic States
 - [x] Support for lifetimes

## Planned Features

 - [ ] Better compile errors
 - [ ] Support for closure bindings
 - [ ] Support for derived state
 - [ ] macro for automatic context providing

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

```rust
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
