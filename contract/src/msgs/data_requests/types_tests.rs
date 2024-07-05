use test_helpers::{calculate_dr_id_and_args, construct_dr};
use testing::MockStorage;
use types::*;

use super::*;
use crate::enumerable_status_map;

struct TestInfo<'a> {
    pub store: MockStorage,
    pub map:   DataRequestsMap<'a>,
}

impl TestInfo<'_> {
    fn init() -> Self {
        let mut store = MockStorage::new();
        let map: DataRequestsMap = enumerable_status_map!("test");
        map.initialize(&mut store).unwrap();
        Self { store, map }
    }

    #[track_caller]
    fn assert_len(&self, expected: u32) {
        assert_eq!(self.map.len(&self.store).unwrap(), expected);
    }

    #[track_caller]
    fn assert_status_len(&self, expected: u32, status: &DataRequestStatus) {
        assert_eq!(
            self.map.get_status_len_item(status).load(&self.store).unwrap(),
            expected
        );
    }

    #[track_caller]
    fn assert_index_to_key(&self, index: u32, key: Option<Hash>) {
        assert_eq!(self.map.index_to_key.may_load(&self.store, index).unwrap(), key);
    }

    #[track_caller]
    fn assert_key_to_index(&self, key: &Hash, index: Option<u32>) {
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
    fn insert(&mut self, key: &Hash, value: DataRequest) {
        self.map.insert(&mut self.store, key, value).unwrap();
    }

    #[track_caller]
    fn update(&mut self, key: &Hash, dr: DataRequest, status: Option<DataRequestStatus>) {
        self.map.update(&mut self.store, key, dr, status).unwrap();
    }

    #[track_caller]
    fn swap_remove(&mut self, key: &Hash) {
        self.map.swap_remove(&mut self.store, key).unwrap();
    }

    #[track_caller]
    fn get_by_index(&self, index: u32) -> Option<DataRequest> {
        self.map.may_get_by_index(&self.store, index).unwrap()
    }

    #[track_caller]
    fn get_by_key(&self, key: &Hash) -> Option<DataRequest> {
        self.map.may_get_by_key(&self.store, key).unwrap()
    }

    #[track_caller]
    fn get_requests_by_status(&self, status: DataRequestStatus, offset: u32, limit: u32) -> Vec<DataRequest> {
        self.map
            .get_requests_by_status(&self.store, status, offset, limit)
            .unwrap()
    }
}

fn create_test_dr(nonce: u128) -> (Hash, DataRequest) {
    let args = calculate_dr_id_and_args(nonce, 2);
    let id = nonce.to_string().hash();
    let dr = construct_dr(args, vec![], nonce as u64);

    (id, dr)
}

#[test]
fn enum_map_initialize() {
    let test_info = TestInfo::init();
    test_info.assert_len(0);
}

#[test]
fn enum_map_insert() {
    let mut test_info = TestInfo::init();
    let (key, val) = create_test_dr(1);
    test_info.insert(&key, val);
    test_info.assert_len(1);
    test_info.assert_status_len(1, &DataRequestStatus::Committing);
    test_info.assert_index_to_key(0, Some(key));
    test_info.assert_key_to_index(&key, Some(0));
    test_info.assert_status_index_to_key(StatusIndexKey::new(0, None), Some(key));
}

#[test]
fn enum_map_get() {
    let mut test_info = TestInfo::init();
    let (key, req) = create_test_dr(1);
    test_info.insert(&key, req.clone());
    assert_eq!(test_info.get_by_index(0), Some(req.clone()));
    assert_eq!(test_info.get_by_key(&key), Some(req));
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
    let (key, req) = create_test_dr(1);
    test_info.insert(&key, req.clone());
    test_info.insert(&key, req);
}

#[test]
fn enum_map_update() {
    let mut test_info = TestInfo::init();
    let (key1, dr1) = create_test_dr(1);
    let (_, dr2) = create_test_dr(2);

    test_info.insert(&key1, dr1.clone());
    test_info.assert_len(1);
    test_info.assert_status_len(1, &DataRequestStatus::Committing);
    assert_eq!(test_info.get_by_index(0), Some(dr1));
    test_info.assert_status_index_to_key(StatusIndexKey::new(0, None), Some(key1));

    test_info.update(&key1, dr2.clone(), None);
    test_info.assert_len(1);
    test_info.assert_status_len(1, &DataRequestStatus::Committing);
    assert_eq!(test_info.get_by_index(0), Some(dr2));
    test_info.assert_status_index_to_key(StatusIndexKey::new(0, None), Some(key1));
}

#[test]
#[should_panic(expected = "Key does not exist")]
fn enum_map_update_non_existing() {
    let mut test_info = TestInfo::init();
    let (key, req) = create_test_dr(1);
    test_info.update(&key, req, None);
}

#[test]
fn enum_map_remove_first() {
    let mut test_info = TestInfo::init();
    let (key1, req1) = create_test_dr(1);
    let (key2, req2) = create_test_dr(2);
    let (key3, req3) = create_test_dr(3);

    test_info.insert(&key1, req1.clone()); // 0
    test_info.insert(&key2, req2.clone()); // 1
    test_info.insert(&key3, req3.clone()); // 2

    test_info.swap_remove(&key1);
    test_info.assert_len(2);
    test_info.assert_status_len(2, &DataRequestStatus::Committing);
    test_info.assert_index_to_key(0, Some(key3));
    test_info.assert_key_to_index(&key3, Some(0));
    test_info.assert_status_index_to_key(StatusIndexKey::new(0, None), Some(key3));

    // test that the index is updated
    assert_eq!(test_info.get_by_index(0), Some(req3.clone()));
    assert_eq!(test_info.get_by_index(1), Some(req2.clone()));

    // test that the req is removed
    assert_eq!(test_info.get_by_key(&key1), None);
    assert_eq!(test_info.get_by_index(2), None);
    test_info.assert_status_index_to_key(StatusIndexKey::new(2, None), None);
}

#[test]
fn enum_map_remove_last() {
    let mut test_info = TestInfo::init();
    let (key1, req1) = create_test_dr(1);
    let (key2, req2) = create_test_dr(2);
    let (key3, req3) = create_test_dr(3);

    test_info.insert(&key1, req1.clone()); // 0
    test_info.insert(&key2, req2.clone()); // 1
    test_info.insert(&key3, req3.clone()); // 2
    test_info.assert_len(3);
    test_info.assert_status_len(3, &DataRequestStatus::Committing);

    test_info.swap_remove(&key3);
    test_info.assert_len(2);
    test_info.assert_status_len(2, &DataRequestStatus::Committing);
    test_info.assert_index_to_key(2, None);
    test_info.assert_key_to_index(&key3, None);
    test_info.assert_status_index_to_key(StatusIndexKey::new(2, None), None);

    // test that the index is updated
    assert_eq!(test_info.get_by_index(2), None);
    // test that the key is removed
    assert_eq!(test_info.get_by_key(&key3), None);

    // check that the other keys are still there
    assert_eq!(test_info.get_by_key(&key1), Some(req1.clone()));
    assert_eq!(test_info.get_by_key(&key2), Some(req2.clone()));

    // check that the other indexes are still there
    assert_eq!(test_info.get_by_index(0), Some(req1));
    assert_eq!(test_info.get_by_index(1), Some(req2));

    // test that the status indexes are still there
    test_info.assert_status_index_to_key(StatusIndexKey::new(0, None), Some(key1));
    test_info.assert_status_index_to_key(StatusIndexKey::new(1, None), Some(key2));
}

#[test]
fn enum_map_remove() {
    let mut test_info = TestInfo::init();
    let (key1, req1) = create_test_dr(1);
    let (key2, req2) = create_test_dr(2);
    let (key3, req3) = create_test_dr(3);
    let (key4, req4) = create_test_dr(4);

    test_info.insert(&key1, req1.clone()); // 0
    test_info.insert(&key2, req2.clone()); // 1
    test_info.insert(&key3, req3.clone()); // 2
    test_info.insert(&key4, req4.clone()); // 3
    test_info.assert_len(4);
    test_info.assert_status_len(4, &DataRequestStatus::Committing);

    test_info.swap_remove(&key2);
    test_info.assert_len(3);
    test_info.assert_status_len(3, &DataRequestStatus::Committing);
    test_info.assert_index_to_key(1, Some(key4));
    test_info.assert_key_to_index(&key4, Some(1));

    // test that the index is updated
    assert_eq!(test_info.get_by_index(0), Some(req1.clone()));
    assert_eq!(test_info.get_by_index(1), Some(req4.clone()));
    assert_eq!(test_info.get_by_index(2), Some(req3.clone()));
    assert_eq!(test_info.get_by_index(3), None);

    // test that the key is removed
    assert_eq!(test_info.get_by_key(&key2), None);

    // check that the other keys are still there
    assert_eq!(test_info.get_by_key(&key1), Some(req1));
    assert_eq!(test_info.get_by_key(&key3), Some(req3));
    assert_eq!(test_info.get_by_key(&key4), Some(req4));

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
fn get_requests_by_status() {
    let mut test_info = TestInfo::init();

    let (key1, req1) = create_test_dr(1);
    test_info.insert(&key1, req1.clone());

    let (key2, req2) = create_test_dr(2);
    test_info.insert(&key2, req2.clone());
    test_info.update(&key2, req2.clone(), Some(DataRequestStatus::Revealing));

    let committing = test_info.get_requests_by_status(DataRequestStatus::Committing, 0, 10);
    assert_eq!(committing.len(), 1);
    assert!(committing.contains(&req1));

    let revealing = test_info.get_requests_by_status(DataRequestStatus::Revealing, 0, 10);
    assert_eq!(revealing.len(), 1);
    assert!(revealing.contains(&req2));
}

#[test]
fn get_requests_by_status_pagination() {
    let mut test_info = TestInfo::init();

    let mut reqs = Vec::with_capacity(10);

    // indexes 0 - 9
    for i in 0..10 {
        let (key, req) = create_test_dr(i);
        test_info.insert(&key, req.clone());
        reqs.push(req);
    }

    // [3, 4]
    let three_four = test_info.get_requests_by_status(DataRequestStatus::Committing, 3, 2);
    assert_eq!(three_four.len(), 2);
    assert!(three_four.contains(&reqs[3]));
    assert!(three_four.contains(&reqs[4]));

    // [5, 9]
    let five_nine = test_info.get_requests_by_status(DataRequestStatus::Committing, 5, 5);
    assert_eq!(five_nine.len(), 5);
    assert!(five_nine.contains(&reqs[5]));
    assert!(five_nine.contains(&reqs[6]));
    assert!(five_nine.contains(&reqs[7]));
    assert!(five_nine.contains(&reqs[8]));
    assert!(five_nine.contains(&reqs[9]));
}

#[test]
#[should_panic(expected = "Key does not exist")]
fn remove_from_empty() {
    let mut test_info = TestInfo::init();
    test_info.swap_remove(&1.to_string().hash());
}

#[test]
fn remove_only_item() {
    let mut test_info = TestInfo::init();
    let (key, req) = create_test_dr(1);
    test_info.insert(&key, req.clone());
    test_info.swap_remove(&key);
    test_info.assert_len(0);
    test_info.assert_status_len(0, &DataRequestStatus::Committing);
    test_info.assert_index_to_key(0, None);
    test_info.assert_key_to_index(&key, None);
    test_info.assert_status_index_to_key(StatusIndexKey::new(0, None), None);
}
