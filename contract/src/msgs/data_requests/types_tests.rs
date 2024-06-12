use test_helpers::{calculate_dr_id_and_args, construct_dr};
use testing::MockStorage;
use types::*;

use super::*;
use crate::enumerable_map;

struct TestInfo<'a> {
    pub store: MockStorage,
    pub map:   EnumerableMap<'a>,
}

impl TestInfo<'_> {
    fn init() -> Self {
        let mut store = MockStorage::new();
        let map: EnumerableMap = enumerable_map!("test");
        map.initialize(&mut store).unwrap();
        Self { store, map }
    }

    #[track_caller]
    fn assert_len(&self, expected: u128) {
        assert_eq!(self.map.len(&self.store).unwrap(), expected);
    }

    #[track_caller]
    fn assert_index_to_key(&self, index: u128, key: Option<Hash>) {
        assert_eq!(self.map.index_to_key.may_load(&self.store, index).unwrap(), key);
    }

    #[track_caller]
    fn assert_key_to_index(&self, key: &Hash, index: Option<u128>) {
        assert_eq!(self.map.key_to_index.may_load(&self.store, key).unwrap(), index);
    }

    #[track_caller]
    fn assert_status_index_to_key(&self, status_index: StatusIndexKey, key: Option<Hash>) {
        assert_eq!(
            self.map.status_to_keys.may_load(&self.store, &status_index).unwrap(),
            key
        );
    }

    #[track_caller]
    fn insert(&mut self, key: &Hash, value: &StatusValue) {
        self.map.insert(&mut self.store, key, value).unwrap();
    }

    #[track_caller]
    fn update(&mut self, key: &Hash, value: &StatusValue) {
        self.map.update(&mut self.store, key, value).unwrap();
    }

    #[track_caller]
    fn swap_remove(&mut self, key: &Hash) {
        self.map.swap_remove(&mut self.store, key).unwrap();
    }

    #[track_caller]
    fn get_by_index(&self, index: u128) -> Option<DataRequest> {
        self.map.get_by_index(&self.store, index).unwrap()
    }

    #[track_caller]
    fn get_by_key(&self, key: &Hash) -> Option<DataRequest> {
        self.map.get_by_key(&self.store, key).unwrap()
    }

    #[track_caller]
    fn get_requests_by_status(&self, status: DataRequestStatus) -> Vec<DataRequest> {
        self.map.get_requests_by_status(&self.store, status).unwrap()
    }
}

fn create_test_dr(nonce: u128, status: Option<DataRequestStatus>) -> (Hash, StatusValue) {
    let args = calculate_dr_id_and_args(nonce, 2);
    let id = args.dr_binary_id;
    let dr = construct_dr(args.dr_binary_id, args, vec![], 1);
    (
        id,
        match status {
            Some(status) => StatusValue::with_status(dr, status),
            None => StatusValue::new(dr),
        },
    )
}

#[test]
fn enum_map_initialize() {
    let test_info = TestInfo::init();
    test_info.assert_len(0);
}

#[test]
fn enum_map_insert() {
    let mut test_info = TestInfo::init();
    let (key, val) = create_test_dr(1, None);
    test_info.insert(&key, &val);
    test_info.assert_len(1);
    test_info.assert_index_to_key(0, Some(key));
    test_info.assert_key_to_index(&key, Some(0));
    test_info.assert_status_index_to_key(StatusIndexKey::new(0, None), Some(key));
}

#[test]
fn enum_map_get() {
    let mut test_info = TestInfo::init();
    let (key, val) = create_test_dr(1, None);
    test_info.insert(&key, &val);
    assert_eq!(test_info.get_by_index(0), Some(val.req.clone()));
    assert_eq!(test_info.get_by_key(&key), Some(val.req));
}

#[test]
fn enum_map_get_non_existing() {
    let test_info = TestInfo::init();
    assert_eq!(test_info.get_by_index(0), None);
    assert_eq!(test_info.get_by_key(&"1".hash()), None);
    test_info.assert_status_index_to_key(StatusIndexKey::new(0, None), None);
}

#[test]
#[should_panic(expected = "Key already exists")]
fn enum_map_insert_duplicate() {
    let mut test_info = TestInfo::init();
    let (key, val) = create_test_dr(1, None);
    test_info.insert(&key, &val);
    test_info.insert(&key, &val);
}

#[test]
fn enum_map_update() {
    let mut test_info = TestInfo::init();
    let (key1, dr1) = create_test_dr(1, None);
    let (_, dr2) = create_test_dr(2, None);
    test_info.insert(&key1, &dr1);
    test_info.update(&key1, &dr2);
    test_info.assert_len(1);
    assert_eq!(test_info.get_by_index(0), Some(dr2.req));
    test_info.assert_status_index_to_key(StatusIndexKey::new(0, None), Some(key1));
}

#[test]
#[should_panic(expected = "Key does not exist")]
fn enum_map_update_non_existing() {
    let mut test_info = TestInfo::init();
    let (key, val) = create_test_dr(1, None);
    test_info.update(&key, &val);
}

#[test]
fn enum_map_remove_first() {
    let mut test_info = TestInfo::init();
    let (key1, req1) = create_test_dr(1, None);
    let (key2, req2) = create_test_dr(2, None);
    let (key3, req3) = create_test_dr(3, None);

    test_info.insert(&key1, &req1); // 0
    test_info.insert(&key2, &req2); // 1
    test_info.insert(&key3, &req3); // 2

    test_info.swap_remove(&key1);
    test_info.assert_len(2);
    test_info.assert_index_to_key(0, Some(key3));
    test_info.assert_key_to_index(&key3, Some(0));
    test_info.assert_status_index_to_key(StatusIndexKey::new(0, None), Some(key3));

    // test that the index is updated
    assert_eq!(test_info.get_by_index(0), Some(req3.req.clone()));
    assert_eq!(test_info.get_by_index(1), Some(req2.req.clone()));

    // test that the req is removed
    assert_eq!(test_info.get_by_key(&key1), None);
    assert_eq!(test_info.get_by_index(2), None);
    test_info.assert_status_index_to_key(StatusIndexKey::new(2, None), None);
}

#[test]
fn enum_map_remove_last() {
    let mut test_info = TestInfo::init();
    let (key1, req1) = create_test_dr(1, None);
    let (key2, req2) = create_test_dr(2, None);
    let (key3, req3) = create_test_dr(3, None);

    test_info.insert(&key1, &req1); // 0
    test_info.insert(&key2, &req2); // 1
    test_info.insert(&key3, &req3); // 2
    test_info.assert_len(3);

    test_info.swap_remove(&key3);
    test_info.assert_len(2);
    test_info.assert_index_to_key(2, None);
    test_info.assert_key_to_index(&key3, None);
    test_info.assert_status_index_to_key(StatusIndexKey::new(2, None), None);

    // test that the index is updated
    assert_eq!(test_info.get_by_index(2), None);
    // test that the key is removed
    assert_eq!(test_info.get_by_key(&key3), None);

    // check that the other keys are still there
    assert_eq!(test_info.get_by_key(&key1), Some(req1.req.clone()));
    assert_eq!(test_info.get_by_key(&key2), Some(req2.req.clone()));

    // check that the other indexes are still there
    assert_eq!(test_info.get_by_index(0), Some(req1.req));
    assert_eq!(test_info.get_by_index(1), Some(req2.req));

    // test that the status indexes are still there
    test_info.assert_status_index_to_key(StatusIndexKey::new(0, None), Some(key1));
    test_info.assert_status_index_to_key(StatusIndexKey::new(1, None), Some(key2));
}

#[test]
fn enum_map_remove() {
    let mut test_info = TestInfo::init();
    let (key1, req1) = create_test_dr(1, None);
    let (key2, req2) = create_test_dr(2, None);
    let (key3, req3) = create_test_dr(3, None);
    let (key4, req4) = create_test_dr(4, None);

    test_info.insert(&key1, &req1); // 0
    test_info.insert(&key2, &req2); // 1
    test_info.insert(&key3, &req3); // 2
    test_info.insert(&key4, &req4); // 3
    test_info.assert_len(4);

    test_info.swap_remove(&key2);
    test_info.assert_len(3);
    test_info.assert_index_to_key(1, Some(key4));
    test_info.assert_key_to_index(&key4, Some(1));

    // test that the index is updated
    assert_eq!(test_info.get_by_index(0), Some(req1.req.clone()));
    assert_eq!(test_info.get_by_index(1), Some(req4.req.clone()));
    assert_eq!(test_info.get_by_index(2), Some(req3.req.clone()));
    assert_eq!(test_info.get_by_index(3), None);

    // test that the key is removed
    assert_eq!(test_info.get_by_key(&key2), None);

    // check that the other keys are still there
    assert_eq!(test_info.get_by_key(&key1), Some(req1.req));
    assert_eq!(test_info.get_by_key(&key3), Some(req3.req));
    assert_eq!(test_info.get_by_key(&key4), Some(req4.req));

    // check the status indexes
    test_info.assert_status_index_to_key(StatusIndexKey::new(0, None), Some(key1));
    test_info.assert_status_index_to_key(StatusIndexKey::new(1, None), Some(key4));
    test_info.assert_status_index_to_key(StatusIndexKey::new(2, None), Some(key3));
    test_info.assert_status_index_to_key(StatusIndexKey::new(3, None), None);
}

#[test]
#[should_panic(expected = "Key does not exist")]
fn enum_map_remove_non_existing() {
    let mut test_info = TestInfo::init();
    test_info.swap_remove(&2.to_string().hash());
}

#[test]
fn get_by_status() {
    let mut test_info = TestInfo::init();

    let (key1, req1) = create_test_dr(1, None);
    test_info.insert(&key1, &req1);

    let (key2, req2) = create_test_dr(2, Some(DataRequestStatus::Revealing));
    test_info.insert(&key2, &req2);

    let committing = test_info.get_requests_by_status(DataRequestStatus::Committing);
    assert_eq!(committing.len(), 1);
    assert_eq!(committing[0], req1.req);

    let revealing = test_info.get_requests_by_status(DataRequestStatus::Revealing);
    assert_eq!(revealing.len(), 1);
    assert_eq!(revealing[0], req2.req);
}
