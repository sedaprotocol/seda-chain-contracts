use test_helpers::{calculate_dr_id_and_args, construct_dr};
use testing::MockStorage;

use super::*;
use crate::{
    consts::{INITIAL_COMMIT_TIMEOUT_IN_BLOCKS, INITIAL_REVEAL_TIMEOUT_IN_BLOCKS},
    msgs::data_requests::consts::min_post_dr_cost,
};

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
    fn assert_status_key_to_index(&self, status: &DataRequestStatus, key: &Hash, index: Option<u32>) {
        let status_map = match status {
            DataRequestStatus::Committing => &self.map.committing,
            DataRequestStatus::Revealing => &self.map.revealing,
            DataRequestStatus::Tallying => &self.map.tallying,
        };
        assert_eq!(index, status_map.key_to_index.may_load(&self.store, key).unwrap());
    }

    #[track_caller]
    fn assert_status_index_to_key(&self, status: &DataRequestStatus, status_index: u32, entry: Option<Entry<'_>>) {
        let status_map = match status {
            DataRequestStatus::Committing => &self.map.committing,
            DataRequestStatus::Revealing => &self.map.revealing,
            DataRequestStatus::Tallying => &self.map.tallying,
        };
        assert_eq!(
            entry,
            status_map.index_to_value.may_load(&self.store, status_index).unwrap()
        );
    }

    #[track_caller]
    fn insert(&mut self, current_height: u64, entry: Entry<'_>, value: DataRequest) {
        self.map
            .insert(
                &mut self.store,
                current_height,
                entry,
                value,
                &DataRequestStatus::Committing,
            )
            .unwrap();
    }

    #[track_caller]
    fn insert_removable(&mut self, current_height: u64, entry: Entry<'_>, value: DataRequest) {
        self.map
            .insert(
                &mut self.store,
                current_height,
                entry,
                value,
                &DataRequestStatus::Tallying,
            )
            .unwrap();
    }

    #[track_caller]
    fn update(&mut self, key: &Hash, dr: DataRequest, status: Option<DataRequestStatus>, current_height: u64) {
        self.map
            .update(&mut self.store, key, dr, status, current_height, false)
            .unwrap();
    }

    #[track_caller]
    fn remove(&mut self, key: &Hash) {
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

fn create_test_dr<'a>(nonce: u128, cost: Option<u128>) -> (Entry<'a>, DataRequest) {
    let args = calculate_dr_id_and_args(nonce, 2);
    let id = nonce.to_string().hash();
    let dr = construct_dr(args, vec![], nonce as u64);
    let cost = Uint128::new(cost.unwrap_or(min_post_dr_cost()));
    let entry = Entry::new(cost, id);

    (entry, dr)
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

    let (entry, val) = create_test_dr(1, None);
    test_info.insert(1, entry.clone(), val);
    test_info.assert_status_len(1, TEST_STATUS);
    test_info.assert_status_key_to_index(TEST_STATUS, &entry.key, Some(0));
    test_info.assert_status_index_to_key(TEST_STATUS, 0, Some(entry));
}

#[test]
fn enum_map_get() {
    let mut test_info = TestInfo::init();

    let (entry, req) = create_test_dr(1, None);
    test_info.insert(1, entry.clone(), req.clone());
    test_info.assert_request(&entry.key, Some(req))
}

#[test]
fn enum_map_get_non_existing() {
    let test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Committing;

    test_info.assert_request(&"1".hash(), None);
    test_info.assert_status_len(0, TEST_STATUS);
    test_info.assert_status_key_to_index(TEST_STATUS, &"1".hash(), None);
    test_info.assert_status_index_to_key(TEST_STATUS, 0, None);
}

#[test]
#[should_panic(expected = "Key already exists")]
fn enum_map_insert_duplicate() {
    let mut test_info = TestInfo::init();
    let (entry, req) = create_test_dr(1, None);
    test_info.insert(1, entry.clone(), req.clone());
    test_info.insert(1, entry, req);
}

#[test]
fn enum_map_update() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Committing;

    let (entry1, dr1) = create_test_dr(1, None);
    let (_, dr2) = create_test_dr(2, None);
    let current_height = 1;

    test_info.insert(current_height, entry1.clone(), dr1.clone());
    test_info.assert_status_len(1, TEST_STATUS);
    test_info.assert_status_key_to_index(TEST_STATUS, &entry1.key, Some(0));
    test_info.assert_status_index_to_key(TEST_STATUS, 0, Some(entry1.clone()));

    test_info.update(&entry1.key, dr2.clone(), None, current_height);
    test_info.assert_status_len(1, TEST_STATUS);
    test_info.assert_status_key_to_index(TEST_STATUS, &entry1.key, Some(0));
    test_info.assert_status_index_to_key(TEST_STATUS, 0, Some(entry1));
}

#[test]
#[should_panic(expected = "Key does not exist")]
fn enum_map_update_non_existing() {
    let mut test_info = TestInfo::init();
    let (entry1, req) = create_test_dr(1, None);
    let current_height = 1;
    test_info.update(&entry1.key, req, None, current_height);
}

#[test]
fn enum_map_remove_first() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Tallying;

    let (entry1, req1) = create_test_dr(1, Some(3));
    let (entry2, req2) = create_test_dr(2, Some(2));
    let (entry3, req3) = create_test_dr(3, Some(1));

    test_info.insert_removable(1, entry1.clone(), req1.clone()); // 0
    test_info.insert_removable(1, entry2.clone(), req2.clone()); // 1
    test_info.insert_removable(1, entry3.clone(), req3.clone()); // 2

    test_info.remove(&entry1.key);
    test_info.assert_status_len(2, TEST_STATUS);
    test_info.assert_status_key_to_index(TEST_STATUS, &entry3.key, Some(1));
    test_info.assert_status_index_to_key(TEST_STATUS, 1, Some(entry3.clone()));

    // test that we can still get the other keys
    test_info.assert_request(&entry2.key, Some(req2.clone()));
    test_info.assert_request(&entry3.key, Some(req3.clone()));

    // test that the req is removed
    test_info.assert_request(&entry1.key, None);
    test_info.assert_status_key_to_index(TEST_STATUS, &entry1.key, None);
}

#[test]
fn enum_map_remove_last() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Tallying;

    let (entry1, req1) = create_test_dr(1, None);
    let (entry2, req2) = create_test_dr(2, None);
    let (entry3, req3) = create_test_dr(3, None);

    test_info.insert_removable(1, entry1.clone(), req1.clone()); // 0
    test_info.insert_removable(1, entry2.clone(), req2.clone()); // 1
    test_info.insert_removable(1, entry3.clone(), req3.clone()); // 2
    test_info.assert_status_len(3, TEST_STATUS);

    test_info.remove(&entry3.key);
    test_info.assert_status_len(2, TEST_STATUS);
    test_info.assert_status_index_to_key(TEST_STATUS, 2, None);
    test_info.assert_status_key_to_index(TEST_STATUS, &entry3.key, None);

    // check that the other keys are still there
    assert_eq!(test_info.get(&entry1.key), Some(req1.clone()));
    assert_eq!(test_info.get(&entry2.key), Some(req2.clone()));

    // test that the status indexes are still there
    test_info.assert_status_index_to_key(TEST_STATUS, 0, Some(entry1));
    test_info.assert_status_index_to_key(TEST_STATUS, 1, Some(entry2));
}

#[test]
fn enum_map_remove() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Tallying;

    let (entry1, req1) = create_test_dr(1, Some(1));
    let (entry2, req2) = create_test_dr(2, Some(4));
    let (entry3, req3) = create_test_dr(3, Some(3));
    let (entry4, req4) = create_test_dr(4, Some(2));

    test_info.insert_removable(1, entry1.clone(), req1.clone());
    test_info.insert_removable(1, entry2.clone(), req2.clone());
    test_info.insert_removable(1, entry3.clone(), req3.clone());
    test_info.insert_removable(1, entry4.clone(), req4.clone());
    // indexes should be ordered by cost:
    // entry1: 3
    // entry2: 0
    // entry3: 1
    // entry4: 2
    test_info.assert_status_len(4, &DataRequestStatus::Tallying);

    test_info.remove(&entry2.key);

    // test that the key is removed
    test_info.assert_status_len(3, &DataRequestStatus::Tallying);
    test_info.assert_request(&entry2.key, None);

    // check that the other keys are still there
    test_info.assert_request(&entry1.key, Some(req1));
    test_info.assert_request(&entry3.key, Some(req3));
    test_info.assert_request(&entry4.key, Some(req4));

    // check that the status is updated
    test_info.assert_status_key_to_index(TEST_STATUS, &entry1.key, Some(2));
    test_info.assert_status_key_to_index(TEST_STATUS, &entry2.key, None);
    test_info.assert_status_key_to_index(TEST_STATUS, &entry3.key, Some(0));
    test_info.assert_status_key_to_index(TEST_STATUS, &entry4.key, Some(1));

    // check the status indexes
    test_info.assert_status_index_to_key(TEST_STATUS, 2, Some(entry1));
    test_info.assert_status_index_to_key(TEST_STATUS, 1, Some(entry4));
    test_info.assert_status_index_to_key(TEST_STATUS, 0, Some(entry3));
    test_info.assert_status_index_to_key(TEST_STATUS, 3, None);
}

#[test]
#[should_panic(expected = "Key does not exist")]
fn enum_map_remove_non_existing() {
    let mut test_info = TestInfo::init();
    test_info.remove(&2.to_string().hash());
}

#[test]
fn get_requests_by_status() {
    let mut test_info = TestInfo::init();
    let current_height = 1;

    let (entry1, req1) = create_test_dr(1, None);
    test_info.insert(current_height, entry1.clone(), req1.clone());

    let (entry2, req2) = create_test_dr(2, None);
    test_info.insert(current_height, entry2.clone(), req2.clone());
    test_info.update(
        &entry2.key,
        req2.clone(),
        Some(DataRequestStatus::Revealing),
        current_height,
    );

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
        let (entry, req) = create_test_dr(i, None);
        test_info.insert(1, entry, req.clone());
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
    test_info.remove(&1.to_string().hash());
}

#[test]
fn remove_only_item() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Tallying;

    let (entry, req) = create_test_dr(1, None);
    test_info.insert_removable(1, entry.clone(), req.clone());
    test_info.remove(&entry.key);

    test_info.assert_status_len(0, &DataRequestStatus::Tallying);
    test_info.assert_status_index_to_key(TEST_STATUS, 0, None);
    test_info.assert_status_key_to_index(TEST_STATUS, &entry.key, None);
}
