use testing::MockStorage;
use types::*;

use super::*;
use crate::enumerable_map;

struct TestInfo<'a> {
    pub store: MockStorage,
    pub map:   EnumerableMap<'a, u8, u8>,
}
impl TestInfo<'_> {
    fn init() -> Self {
        let mut store = MockStorage::new();
        let map: EnumerableMap<u8, u8> = enumerable_map!("test");
        map.initialize(&mut store).unwrap();
        Self { store, map }
    }

    #[track_caller]
    fn assert_len(&self, expected: u128) {
        assert_eq!(self.map.len(&self.store).unwrap(), expected);
    }

    #[track_caller]
    fn assert_index_to_key(&self, index: u128, key: Option<u8>) {
        assert_eq!(self.map.index_to_key.may_load(&self.store, index).unwrap(), key);
    }

    #[track_caller]
    fn assert_key_to_index(&self, key: u8, index: Option<u128>) {
        assert_eq!(self.map.key_to_index.may_load(&self.store, key).unwrap(), index);
    }

    fn insert(&mut self, key: u8, value: u8) {
        self.map.insert(&mut self.store, key, value).unwrap();
    }

    fn update(&mut self, key: u8, value: u8) {
        self.map.update(&mut self.store, key, &value).unwrap();
    }

    fn swap_remove(&mut self, key: u8) {
        self.map.swap_remove(&mut self.store, key).unwrap();
    }

    fn get_by_index(&self, index: u128) -> Option<u8> {
        self.map.get_by_index(&self.store, index).unwrap()
    }

    fn get_by_key(&self, key: u8) -> Option<u8> {
        self.map.get_by_key(&self.store, key).unwrap()
    }
}

#[test]
fn enum_map_initialize() {
    let test_info = TestInfo::init();
    test_info.assert_len(0);
}

#[test]
fn enum_map_insert() {
    let mut test_info = TestInfo::init();
    test_info.insert(1, 2);
    test_info.assert_len(1);
}

#[test]
fn enum_map_get() {
    let mut test_info = TestInfo::init();
    test_info.insert(1, 2);
    assert_eq!(test_info.get_by_index(0), Some(2));
    assert_eq!(test_info.get_by_key(1), Some(2));
}

#[test]
fn enum_map_get_non_existing() {
    let test_info = TestInfo::init();
    assert_eq!(test_info.get_by_index(0), None);
    assert_eq!(test_info.get_by_key(1), None);
}

#[test]
#[should_panic(expected = "Key already exists")]
fn enum_map_insert_duplicate() {
    let mut test_info = TestInfo::init();
    test_info.insert(1, 1);
    test_info.insert(1, 2);
}

#[test]
fn enum_map_update() {
    let mut test_info = TestInfo::init();
    test_info.insert(1, 1);
    test_info.update(1, 2);
    test_info.assert_len(1);
    assert_eq!(test_info.get_by_index(0), Some(2));
}

#[test]
#[should_panic(expected = "Key does not exist")]
fn enum_map_update_non_existing() {
    let mut test_info = TestInfo::init();
    test_info.update(1, 2);
}

#[test]
fn enum_map_remove_last_key() {
    let mut test_info = TestInfo::init();
    test_info.insert(1, 1); // 0
    test_info.insert(2, 2); // 1
    test_info.insert(3, 3); // 2
    test_info.assert_len(3);

    test_info.swap_remove(3);
    test_info.assert_len(2);
    test_info.assert_index_to_key(2, None);
    test_info.assert_key_to_index(3, None);

    // test that the index is updated
    assert_eq!(test_info.get_by_index(2), None);
    // test that the key is removed
    assert_eq!(test_info.get_by_key(3), None);

    // check that the other keys are still there
    assert_eq!(test_info.get_by_key(1), Some(1));
    assert_eq!(test_info.get_by_key(2), Some(2));

    // check that the other indexes are still there
    assert_eq!(test_info.get_by_index(0), Some(1));
    assert_eq!(test_info.get_by_index(1), Some(2));
}

#[test]
fn enum_map_remove_key() {
    let mut test_info = TestInfo::init();
    test_info.insert(1, 1); // 0
    test_info.insert(2, 2); // 1
    test_info.insert(3, 3); // 2
    test_info.insert(4, 4); // 3
    test_info.assert_len(4);

    test_info.swap_remove(2);
    test_info.assert_len(3);
    test_info.assert_index_to_key(1, Some(4));
    test_info.assert_key_to_index(4, Some(1));

    // test that the index is updated
    assert_eq!(test_info.get_by_index(0), Some(1));
    assert_eq!(test_info.get_by_index(1), Some(4));
    assert_eq!(test_info.get_by_index(2), Some(3));
    assert_eq!(test_info.get_by_index(3), None);

    // test that the key is removed
    assert_eq!(test_info.get_by_key(2), None);

    // check that the other keys are still there
    assert_eq!(test_info.get_by_key(1), Some(1));
    assert_eq!(test_info.get_by_key(3), Some(3));
    assert_eq!(test_info.get_by_key(4), Some(4));
}

#[test]
#[should_panic(expected = "Key does not exist")]
fn enum_map_remove_non_existing() {
    let mut test_info = TestInfo::init();
    test_info.swap_remove(2);
}
