use test_helpers::{calculate_dr_id_and_args, construct_dr};
use testing::MockStorage;

use super::*;
use crate::consts::{INITIAL_COMMIT_TIMEOUT_IN_BLOCKS, INITIAL_REVEAL_TIMEOUT_IN_BLOCKS};

struct TestInfo<'a> {
    pub store: MockStorage,
    pub map:   DataRequestsMap<'a>,
}

impl TestInfo<'_> {
    fn init() -> Self {
        let mut store = MockStorage::new();
        let map: DataRequestsMap = new_enumerable_status_map!("test");
        map.initialize(&mut store).unwrap();

        let init_timeout_config = TimeoutConfig {
            commit_timeout_in_blocks: INITIAL_COMMIT_TIMEOUT_IN_BLOCKS,
            reveal_timeout_in_blocks: INITIAL_REVEAL_TIMEOUT_IN_BLOCKS,
        };
        TIMEOUT_CONFIG.save(&mut store, &init_timeout_config).unwrap();

        Self { store, map }
    }

    #[track_caller]
    pub fn assert_status_len(&self, expected: u32, status: &DataRequestStatus) {
        let len = match status {
            DataRequestStatus::Committing => self.map.committing.len(&self.store),
            DataRequestStatus::Revealing => self.map.revealing.len(&self.store),
            DataRequestStatus::Tallying => self.map.tallying.len(&self.store),
        }
        .unwrap();
        assert_eq!(expected, len);
    }

    #[track_caller]
    fn assert_status_key_to_index(&self, status: &DataRequestStatus, key: Hash, index: Option<u32>) {
        let status_map = match status {
            DataRequestStatus::Committing => &self.map.committing,
            DataRequestStatus::Revealing => &self.map.revealing,
            DataRequestStatus::Tallying => &self.map.tallying,
        };
        assert_eq!(index, status_map.key_to_index.may_load(&self.store, key).unwrap());
    }

    #[track_caller]
    fn assert_status_index_to_key(&self, status: &DataRequestStatus, status_index: u32, key: Option<Hash>) {
        let status_map = match status {
            DataRequestStatus::Committing => &self.map.committing,
            DataRequestStatus::Revealing => &self.map.revealing,
            DataRequestStatus::Tallying => &self.map.tallying,
        };
        assert_eq!(
            key,
            status_map.index_to_key.may_load(&self.store, status_index).unwrap()
        );
    }

    #[track_caller]
    fn insert(&mut self, current_height: u64, key: Hash, value: DataRequest) {
        self.map
            .insert(
                &mut self.store,
                current_height,
                key,
                value,
                &DataRequestStatus::Committing,
            )
            .unwrap();
    }

    #[track_caller]
    fn insert_removable(&mut self, current_height: u64, key: Hash, value: DataRequest) {
        self.map
            .insert(
                &mut self.store,
                current_height,
                key,
                value,
                &DataRequestStatus::Tallying,
            )
            .unwrap();
    }

    #[track_caller]
    fn update(&mut self, key: Hash, dr: DataRequest, status: Option<DataRequestStatus>, current_height: u64) {
        self.map
            .update(&mut self.store, key, dr, status, current_height, false)
            .unwrap();
    }

    #[track_caller]
    fn remove(&mut self, key: Hash) {
        self.map.remove(&mut self.store, key).unwrap();
    }

    #[track_caller]
    fn get(&self, key: &Hash) -> Option<DataRequest> {
        self.map.may_get(&self.store, key).unwrap()
    }

    #[track_caller]
    fn assert_request(&self, key: &Hash, expected: Option<DataRequest>) {
        assert_eq!(expected, self.get(key));
    }

    #[track_caller]
    fn get_requests_by_status(&self, status: DataRequestStatus, offset: u32, limit: u32) -> Vec<DataRequest> {
        self.map
            .get_requests_by_status(&self.store, &status, offset, limit)
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
    test_info.assert_status_len(0, &DataRequestStatus::Committing);
    test_info.assert_status_len(0, &DataRequestStatus::Revealing);
    test_info.assert_status_len(0, &DataRequestStatus::Tallying);
}

#[test]
fn enum_map_insert() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Committing;

    let (key, val) = create_test_dr(1);
    test_info.insert(1, key, val);
    test_info.assert_status_len(1, TEST_STATUS);
    test_info.assert_status_key_to_index(TEST_STATUS, key, Some(0));
    test_info.assert_status_index_to_key(TEST_STATUS, 0, Some(key));
}

#[test]
fn enum_map_get() {
    let mut test_info = TestInfo::init();

    let (key, req) = create_test_dr(1);
    test_info.insert(1, key, req.clone());
    test_info.assert_request(&key, Some(req))
}

#[test]
fn enum_map_get_non_existing() {
    let test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Committing;

    test_info.assert_request(&"1".hash(), None);
    test_info.assert_status_len(0, TEST_STATUS);
    test_info.assert_status_key_to_index(TEST_STATUS, "1".hash(), None);
    test_info.assert_status_index_to_key(TEST_STATUS, 0, None);
}

#[test]
#[should_panic(expected = "Key already exists")]
fn enum_map_insert_duplicate() {
    let mut test_info = TestInfo::init();
    let (key, req) = create_test_dr(1);
    test_info.insert(1, key, req.clone());
    test_info.insert(1, key, req);
}

#[test]
fn enum_map_update() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Committing;

    let (key1, dr1) = create_test_dr(1);
    let (_, dr2) = create_test_dr(2);
    let current_height = 1;

    test_info.insert(current_height, key1, dr1.clone());
    test_info.assert_status_len(1, TEST_STATUS);
    test_info.assert_status_key_to_index(TEST_STATUS, key1, Some(0));
    test_info.assert_status_index_to_key(TEST_STATUS, 0, Some(key1));

    test_info.update(key1, dr2.clone(), None, current_height);
    test_info.assert_status_len(1, TEST_STATUS);
    test_info.assert_status_key_to_index(TEST_STATUS, key1, Some(0));
    test_info.assert_status_index_to_key(TEST_STATUS, 0, Some(key1));
}

#[test]
#[should_panic(expected = "Key does not exist")]
fn enum_map_update_non_existing() {
    let mut test_info = TestInfo::init();
    let (key, req) = create_test_dr(1);
    let current_height = 1;
    test_info.update(key, req, None, current_height);
}

#[test]
fn enum_map_remove_first() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Tallying;

    let (key1, req1) = create_test_dr(1);
    let (key2, req2) = create_test_dr(2);
    let (key3, req3) = create_test_dr(3);

    test_info.insert_removable(1, key1, req1.clone()); // 0
    test_info.insert_removable(1, key2, req2.clone()); // 1
    test_info.insert_removable(1, key3, req3.clone()); // 2

    test_info.remove(key1);
    test_info.assert_status_len(2, TEST_STATUS);
    test_info.assert_status_key_to_index(TEST_STATUS, key3, Some(0));
    test_info.assert_status_index_to_key(TEST_STATUS, 0, Some(key3));

    // test that we can still get the other keys
    test_info.assert_request(&key2, Some(req2.clone()));
    test_info.assert_request(&key3, Some(req3.clone()));

    // test that the req is removed
    test_info.assert_request(&key1, None);
    test_info.assert_status_key_to_index(TEST_STATUS, key1, None);
}

#[test]
fn enum_map_remove_last() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Tallying;

    let (key1, req1) = create_test_dr(1);
    let (key2, req2) = create_test_dr(2);
    let (key3, req3) = create_test_dr(3);

    test_info.insert_removable(1, key1, req1.clone()); // 0
    test_info.insert_removable(1, key2, req2.clone()); // 1
    test_info.insert_removable(1, key3, req3.clone()); // 2
    test_info.assert_status_len(3, TEST_STATUS);

    test_info.remove(key3);
    test_info.assert_status_len(2, TEST_STATUS);
    test_info.assert_status_index_to_key(TEST_STATUS, 2, None);
    test_info.assert_status_key_to_index(TEST_STATUS, key3, None);

    // check that the other keys are still there
    assert_eq!(test_info.get(&key1), Some(req1.clone()));
    assert_eq!(test_info.get(&key2), Some(req2.clone()));

    // test that the status indexes are still there
    test_info.assert_status_index_to_key(TEST_STATUS, 0, Some(key1));
    test_info.assert_status_index_to_key(TEST_STATUS, 1, Some(key2));
}

#[test]
fn enum_map_remove() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Tallying;

    let (key1, req1) = create_test_dr(1);
    let (key2, req2) = create_test_dr(2);
    let (key3, req3) = create_test_dr(3);
    let (key4, req4) = create_test_dr(4);

    test_info.insert_removable(1, key1, req1.clone()); // 0
    test_info.insert_removable(1, key2, req2.clone()); // 1
    test_info.insert_removable(1, key3, req3.clone()); // 2
    test_info.insert_removable(1, key4, req4.clone()); // 3
    test_info.assert_status_len(4, &DataRequestStatus::Tallying);

    test_info.remove(key2);

    // test that the key is removed
    test_info.assert_status_len(3, &DataRequestStatus::Tallying);
    test_info.assert_request(&key2, None);

    // check that the other keys are still there
    test_info.assert_request(&key1, Some(req1));
    test_info.assert_request(&key3, Some(req3));
    test_info.assert_request(&key4, Some(req4));

    // check that the status is updated
    test_info.assert_status_key_to_index(TEST_STATUS, key1, Some(0));
    test_info.assert_status_key_to_index(TEST_STATUS, key2, None);
    test_info.assert_status_key_to_index(TEST_STATUS, key3, Some(2));
    test_info.assert_status_key_to_index(TEST_STATUS, key4, Some(1));

    // check the status indexes
    test_info.assert_status_index_to_key(TEST_STATUS, 0, Some(key1));
    test_info.assert_status_index_to_key(TEST_STATUS, 1, Some(key4));
    test_info.assert_status_index_to_key(TEST_STATUS, 2, Some(key3));
    test_info.assert_status_index_to_key(TEST_STATUS, 3, None);
}

#[test]
#[should_panic(expected = "Key does not exist")]
fn enum_map_remove_non_existing() {
    let mut test_info = TestInfo::init();
    test_info.remove(2.to_string().hash());
}

#[test]
fn get_requests_by_status() {
    let mut test_info = TestInfo::init();
    let current_height = 1;

    let (key1, req1) = create_test_dr(1);
    test_info.insert(current_height, key1, req1.clone());

    let (key2, req2) = create_test_dr(2);
    test_info.insert(current_height, key2, req2.clone());
    test_info.update(key2, req2.clone(), Some(DataRequestStatus::Revealing), current_height);

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
        test_info.insert(1, key, req.clone());
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
    test_info.remove(1.to_string().hash());
}

#[test]
fn remove_only_item() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Tallying;

    let (key, req) = create_test_dr(1);
    test_info.insert_removable(1, key, req.clone());
    test_info.remove(key);

    test_info.assert_status_len(0, &DataRequestStatus::Tallying);
    test_info.assert_status_index_to_key(TEST_STATUS, 0, None);
    test_info.assert_status_key_to_index(TEST_STATUS, key, None);
}
