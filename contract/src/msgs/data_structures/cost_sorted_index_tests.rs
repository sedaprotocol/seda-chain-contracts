use std::borrow::Cow;

use cosmwasm_std::{testing::MockStorage, Uint128};
use seda_common::types::Hash;

use super::*;
use crate::cost_sorted_index;

struct TestInfo<'a> {
    pub store: MockStorage,
    pub csi:   CostSortedIndex<'a>,
}

impl TestInfo<'_> {
    fn init() -> Self {
        let mut store = MockStorage::new();
        let csi: CostSortedIndex = cost_sorted_index!("test");
        csi.initialize(&mut store).unwrap();
        Self { store, csi }
    }

    #[track_caller]
    fn get_index(&self, key: &Hash) -> u32 {
        self.csi.get_index(&self.store, key).unwrap()
    }

    #[track_caller]
    fn len(&self) -> u32 {
        self.csi.len(&self.store).unwrap()
    }

    #[track_caller]
    fn has(&self, key: &Hash) -> bool {
        self.csi.has(&self.store, key)
    }

    #[track_caller]
    fn get_entry(&self, index: u32) -> Entry {
        self.csi.index_to_value.load(&self.store, index).unwrap()
    }

    #[track_caller]
    fn get_index_by_key(&self, key: &Hash) -> u32 {
        self.csi.key_to_index.load(&self.store, key).unwrap()
    }

    #[track_caller]
    fn get_key_by_index(&self, index: u32) -> Hash {
        self.csi
            .index_to_value
            .load(&self.store, index)
            .unwrap()
            .key
            .into_owned()
    }

    #[track_caller]
    fn add(&mut self, cost: u128, key: &Hash) {
        self.csi
            .add(
                &mut self.store,
                Entry {
                    cost: cost.into(),
                    key:  Cow::Borrowed(key),
                },
            )
            .unwrap();
    }

    #[track_caller]
    fn remove(&mut self, key: &Hash) {
        self.csi.remove(&mut self.store, key).unwrap();
    }
}

#[test]
fn add() {
    let mut info = TestInfo::init();
    let key = [1; 32];
    let cost = 1000;

    info.add(cost, &key);

    assert_eq!(info.len(), 1);
    assert_eq!(info.get_index(&key), 0);
    assert_eq!(info.get_index_by_key(&key), 0);
    assert_eq!(info.get_key_by_index(0), key);
    assert_eq!(info.get_entry(0).cost, Uint128::new(cost));
}

#[test]
fn add_sorts_by_cost() {
    let mut info = TestInfo::init();
    let key1 = [1; 32];
    let key2 = [2; 32];
    let key3 = [3; 32];
    let cost1 = 1000;
    let cost2 = 2000;
    let cost3 = 1500;

    info.add(cost1, &key1);
    info.add(cost2, &key2);
    info.add(cost3, &key3);

    assert_eq!(info.len(), 3);
    assert_eq!(info.get_index(&key1), 2);
    assert_eq!(info.get_index(&key2), 0);
    assert_eq!(info.get_index(&key3), 1);
}

#[test]
fn adding_same_costs_is_last_in_last_out() {
    let mut info = TestInfo::init();
    let key1 = [1; 32];
    let key2 = [2; 32];
    let key3 = [3; 32];
    let cost = 1000;

    info.add(cost, &key1);
    info.add(cost, &key2);
    info.add(cost, &key3);

    assert_eq!(info.len(), 3);
    assert_eq!(info.get_index(&key1), 0);
    assert_eq!(info.get_index(&key2), 1);
    assert_eq!(info.get_index(&key3), 2);
}

#[test]
fn remove() {
    let mut info = TestInfo::init();
    let key = [1; 32];
    let cost = 1000;

    info.add(cost, &key);
    info.remove(&key);

    assert_eq!(info.len(), 0);
    assert!(!info.has(&key));
}

#[test]
fn remove_middle() {
    let mut info = TestInfo::init();
    let key1 = [1; 32];
    let key2 = [2; 32];
    let key3 = [3; 32];
    let cost1 = 1000;
    let cost2 = 2000;
    let cost3 = 1500;

    info.add(cost1, &key1);
    info.add(cost2, &key2);
    info.add(cost3, &key3);

    // Remove the key at index 1
    info.remove(&key3);
    assert_eq!(info.len(), 2);
    assert_eq!(info.get_index(&key1), 1);
    assert_eq!(info.get_index(&key2), 0);
}

#[test]
fn remove_last() {
    let mut info = TestInfo::init();
    let key1 = [1; 32];
    let key2 = [2; 32];
    let key3 = [3; 32];
    let cost1 = 1000;
    let cost2 = 2000;
    let cost3 = 1500;

    info.add(cost1, &key1);
    info.add(cost2, &key2);
    info.add(cost3, &key3);

    // Remove the key at index 2
    info.remove(&key1);
    assert_eq!(info.len(), 2);
    assert_eq!(info.get_index(&key2), 0);
    assert_eq!(info.get_index(&key3), 1);
}

#[test]
fn remove_first() {
    let mut info = TestInfo::init();
    let key1 = [1; 32];
    let key2 = [2; 32];
    let key3 = [3; 32];
    let cost1 = 1000;
    let cost2 = 2000;
    let cost3 = 1500;

    info.add(cost1, &key1);
    info.add(cost2, &key2);
    info.add(cost3, &key3);

    // Remove the key at index 0
    info.remove(&key2);
    assert_eq!(info.len(), 2);
    assert_eq!(info.get_index(&key1), 1);
    assert_eq!(info.get_index(&key3), 0);
}

#[test]
fn remove_same_cost() {
    let mut info = TestInfo::init();
    let key1 = [1; 32];
    let key2 = [2; 32];
    let key3 = [3; 32];
    let key4 = [4; 32];
    let cost = 1000;

    info.add(cost, &key1);
    info.add(cost, &key2);
    info.add(cost, &key3);
    info.add(cost, &key4);

    // check all are there
    assert_eq!(info.len(), 4);
    assert_eq!(info.get_index(&key1), 0);
    assert_eq!(info.get_index(&key2), 1);
    assert_eq!(info.get_index(&key3), 2);
    assert_eq!(info.get_index(&key4), 3);

    // remove index 1 - should be key2
    info.remove(&key2);

    assert_eq!(info.len(), 3);
    assert_eq!(info.get_index(&key1), 0);
    assert_eq!(info.get_index(&key3), 1);
    assert_eq!(info.get_index(&key4), 2);
}
