use test_helpers::{calculate_dr_id_and_args, construct_dr};
use testing::MockStorage;

use super::*;
use crate::consts::*;

struct TestInfo<'a> {
    pub store: MockStorage,
    pub map:   DataRequestsMap<'a>,
}

impl TestInfo<'_> {
    fn init() -> Self {
        let mut store = MockStorage::new();
        let map: DataRequestsMap = new_enumerable_status_map!("test");
        map.initialize(&mut store).unwrap();

        DR_CONFIG.save(&mut store, &INITIAL_DR_CONFIG).unwrap();

        Self { store, map }
    }

    #[track_caller]
    pub fn assert_dr_reveals_len(&self, expected: usize, dr_id: &Hash) {
        let reveals = self.map.get_reveals(&self.store, dr_id).unwrap();
        assert_eq!(expected, reveals.len());
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
    fn status_index_key_exists(&self, status: &DataRequestStatus, dr_id: &Hash) -> bool {
        let status_map = match status {
            DataRequestStatus::Committing => &self.map.committing,
            DataRequestStatus::Revealing => &self.map.revealing,
            DataRequestStatus::Tallying => &self.map.tallying,
        };
        let Ok(index) = status_map.dr_id_to_index.load(&self.store, dr_id) else {
            return false;
        };
        status_map.has_index(&self.store, index)
    }

    #[track_caller]
    fn status_dr_id_exists(&self, status: &DataRequestStatus, dr_id: &Hash) -> bool {
        let status_map = match status {
            DataRequestStatus::Committing => &self.map.committing,
            DataRequestStatus::Revealing => &self.map.revealing,
            DataRequestStatus::Tallying => &self.map.tallying,
        };
        status_map.has(&self.store, dr_id)
    }

    #[track_caller]
    fn insert(&mut self, current_height: u64, key: &Hash, value: DataRequestContract) {
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
    fn insert_removable(&mut self, current_height: u64, key: &Hash, value: DataRequestContract) {
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
    fn update(&mut self, key: &Hash, dr: DataRequestContract, status: Option<DataRequestStatus>, current_height: u64) {
        self.map
            .update(&mut self.store, key, dr, status, current_height, false)
            .unwrap();
    }

    #[track_caller]
    fn remove(&mut self, key: &Hash) {
        self.map.remove(&mut self.store, key).unwrap();
    }

    #[track_caller]
    fn get(&self, key: &Hash) -> Option<DataRequestContract> {
        self.map.may_get(&self.store, key).unwrap()
    }

    #[track_caller]
    fn assert_request(&self, key: &Hash, expected: Option<DataRequestContract>) {
        assert_eq!(expected, self.get(key));
    }

    #[track_caller]
    fn get_requests_by_status(
        &self,
        status: DataRequestStatus,
        last_seen_index: Option<IndexKey>,
        limit: u32,
    ) -> (Vec<DataRequestResponse>, Option<IndexKey>, u32) {
        self.map
            .get_requests_by_status(&self.store, &status, last_seen_index, limit)
            .unwrap()
    }
}

fn create_test_dr(height: u64) -> (Hash, DataRequestContract) {
    let args = calculate_dr_id_and_args(height as u128, 2);
    let dr = construct_dr(args, vec![], height);

    (Hash::from_hex_str(&dr.base.id).unwrap(), dr)
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
    test_info.insert(1, &key, val);
    test_info.assert_dr_reveals_len(0, &key);
    test_info.assert_status_len(1, TEST_STATUS);
    assert!(test_info.status_dr_id_exists(TEST_STATUS, &key));
    assert!(test_info.status_index_key_exists(TEST_STATUS, &key));
}

#[test]
fn enum_map_get() {
    let mut test_info = TestInfo::init();

    let (key, req) = create_test_dr(1);
    test_info.insert(1, &key, req.clone());
    test_info.assert_request(&key, Some(req))
}

#[test]
fn enum_map_get_non_existing() {
    let test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Committing;

    test_info.assert_request(&"1".hash(), None);
    test_info.assert_status_len(0, TEST_STATUS);
    assert!(!test_info.status_dr_id_exists(TEST_STATUS, &"1".hash()));
    assert!(!test_info.status_index_key_exists(TEST_STATUS, &"1".hash()));
}

#[test]
#[should_panic(expected = "Key already exists")]
fn enum_map_insert_duplicate() {
    let mut test_info = TestInfo::init();
    let (key, req) = create_test_dr(1);
    test_info.insert(1, &key, req.clone());
    test_info.insert(1, &key, req);
}

#[test]
fn enum_map_update() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Committing;

    let (key1, dr1) = create_test_dr(1);
    let (_, dr2) = create_test_dr(2);
    let current_height = 1;

    test_info.insert(current_height, &key1, dr1.clone());
    test_info.assert_status_len(1, TEST_STATUS);
    assert!(test_info.status_dr_id_exists(TEST_STATUS, &key1));
    assert!(test_info.status_index_key_exists(TEST_STATUS, &key1));

    test_info.update(&key1, dr2.clone(), None, current_height);
    test_info.assert_status_len(1, TEST_STATUS);
    assert!(test_info.status_dr_id_exists(TEST_STATUS, &key1));
    assert!(test_info.status_index_key_exists(TEST_STATUS, &key1));
}

#[test]
#[should_panic(expected = "Key does not exist")]
fn enum_map_update_non_existing() {
    let mut test_info = TestInfo::init();
    let (key, req) = create_test_dr(1);
    let current_height = 1;
    test_info.update(&key, req, None, current_height);
}

#[test]
fn enum_map_remove_first() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Tallying;

    let (key1, req1) = create_test_dr(1);
    let (key2, req2) = create_test_dr(2);
    let (key3, req3) = create_test_dr(3);

    test_info.insert_removable(1, &key1, req1.clone()); // 0
    test_info.insert_removable(1, &key2, req2.clone()); // 1
    test_info.insert_removable(1, &key3, req3.clone()); // 2

    test_info.remove(&key1);
    test_info.assert_status_len(2, TEST_STATUS);
    assert!(test_info.status_dr_id_exists(TEST_STATUS, &key3));
    assert!(test_info.status_index_key_exists(TEST_STATUS, &key3));

    // test that we can still get the other keys
    test_info.assert_request(&key2, Some(req2.clone()));
    test_info.assert_request(&key3, Some(req3.clone()));

    // test that the req is removed
    test_info.assert_request(&key1, None);
    assert!(!test_info.status_dr_id_exists(TEST_STATUS, &key1));
    assert!(!test_info.status_index_key_exists(TEST_STATUS, &key1));
}

#[test]
fn enum_map_remove_last() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Tallying;

    let (key1, req1) = create_test_dr(1);
    let (key2, req2) = create_test_dr(2);
    let (key3, req3) = create_test_dr(3);

    test_info.insert_removable(1, &key1, req1.clone()); // 0
    test_info.insert_removable(1, &key2, req2.clone()); // 1
    test_info.insert_removable(1, &key3, req3.clone()); // 2
    test_info.assert_status_len(3, TEST_STATUS);

    test_info.remove(&key3);
    test_info.assert_status_len(2, TEST_STATUS);
    assert!(!test_info.status_dr_id_exists(TEST_STATUS, &key3));
    assert!(!test_info.status_index_key_exists(TEST_STATUS, &key3));

    // check that the other keys are still there
    assert_eq!(test_info.get(&key1), Some(req1.clone()));
    assert_eq!(test_info.get(&key2), Some(req2.clone()));

    // test that the status indexes are still there
    assert!(test_info.status_dr_id_exists(TEST_STATUS, &key1));
    assert!(test_info.status_index_key_exists(TEST_STATUS, &key1));

    assert!(test_info.status_dr_id_exists(TEST_STATUS, &key2));
    assert!(test_info.status_index_key_exists(TEST_STATUS, &key2));
}

#[test]
fn enum_map_remove() {
    let mut test_info = TestInfo::init();
    const TEST_STATUS: &DataRequestStatus = &DataRequestStatus::Tallying;

    let (key1, req1) = create_test_dr(1);
    let (key2, req2) = create_test_dr(2);
    let (key3, req3) = create_test_dr(3);
    let (key4, req4) = create_test_dr(4);

    test_info.insert_removable(1, &key1, req1.clone()); // 0
    test_info.insert_removable(1, &key2, req2.clone()); // 1
    test_info.insert_removable(1, &key3, req3.clone()); // 2
    test_info.insert_removable(1, &key4, req4.clone()); // 3
    test_info.assert_status_len(4, TEST_STATUS);

    test_info.remove(&key2);

    // test that the key is removed
    test_info.assert_status_len(3, TEST_STATUS);
    test_info.assert_request(&key2, None);

    // check that the other keys are still there
    test_info.assert_request(&key1, Some(req1));
    test_info.assert_request(&key3, Some(req3));
    test_info.assert_request(&key4, Some(req4));

    // check that the status is updated
    assert!(test_info.status_dr_id_exists(TEST_STATUS, &key1));
    assert!(!test_info.status_dr_id_exists(TEST_STATUS, &key2));
    assert!(test_info.status_dr_id_exists(TEST_STATUS, &key3));
    assert!(test_info.status_dr_id_exists(TEST_STATUS, &key4));

    // check the status indexes
    assert!(test_info.status_index_key_exists(TEST_STATUS, &key1));
    assert!(!test_info.status_index_key_exists(TEST_STATUS, &key2));
    assert!(test_info.status_index_key_exists(TEST_STATUS, &key3));
    assert!(test_info.status_index_key_exists(TEST_STATUS, &key4));
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

    let (key1, req1) = create_test_dr(1);
    test_info.insert(current_height, &key1, req1.clone());

    let (key2, req2) = create_test_dr(2);
    test_info.insert(current_height, &key2, req2.clone());
    test_info.update(&key2, req2.clone(), Some(DataRequestStatus::Revealing), current_height);

    let (committing, _, total) = test_info.get_requests_by_status(DataRequestStatus::Committing, None, 10);
    assert_eq!(committing.len(), 1);
    assert!(committing.iter().any(|r| r.base.id == req1.base.id));
    assert_eq!(total, 1);

    let (revealing, _, total) = test_info.get_requests_by_status(DataRequestStatus::Revealing, None, 10);
    assert_eq!(revealing.len(), 1);
    assert!(revealing.iter().any(|r| r.base.id == req2.base.id));
    assert_eq!(total, 1);
}

#[test]
fn get_requests_by_status_pagination() {
    let mut test_info = TestInfo::init();

    let mut reqs = Vec::with_capacity(10);

    // indexes 0 - 9
    for i in 0..10 {
        let (key, req) = create_test_dr(i);
        test_info.insert(i, &key, req.clone());
        reqs.push(req);
    }

    let (_, page_two, total) = test_info.get_requests_by_status(DataRequestStatus::Committing, None, 3);
    assert_eq!(total, 10);

    // [3, 4]
    let (three_four, page_three, total) = test_info.get_requests_by_status(DataRequestStatus::Committing, page_two, 2);
    assert_eq!(three_four.len(), 2);
    assert!(three_four.iter().any(|req| req.base.id == reqs[3].base.id));
    assert!(three_four.iter().any(|req| req.base.id == reqs[4].base.id));
    assert_eq!(total, 10);

    // [5, 9]
    let (five_nine, _, total) = test_info.get_requests_by_status(DataRequestStatus::Committing, page_three, 5);
    assert_eq!(five_nine.len(), 5);
    assert!(five_nine.iter().any(|req| req.base.id == reqs[5].base.id));
    assert!(five_nine.iter().any(|req| req.base.id == reqs[6].base.id));
    assert!(five_nine.iter().any(|req| req.base.id == reqs[7].base.id));
    assert!(five_nine.iter().any(|req| req.base.id == reqs[8].base.id));
    assert!(five_nine.iter().any(|req| req.base.id == reqs[9].base.id));
    assert_eq!(total, 10);
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

    let (key, req) = create_test_dr(1);
    test_info.insert_removable(1, &key, req.clone());
    test_info.remove(&key);

    test_info.assert_status_len(0, TEST_STATUS);
    assert!(!test_info.status_dr_id_exists(TEST_STATUS, &key));
    assert!(!test_info.status_index_key_exists(TEST_STATUS, &key));
}

#[test]
fn reveal() {
    let mut test_info = TestInfo::init();

    let (key, req) = create_test_dr(1);
    test_info.insert_removable(1, &key, req.clone());

    let identity = "identity";
    let reveal_body = RevealBody {
        dr_id:             "dr_id".to_string(),
        dr_block_height:   1,
        exit_code:         0,
        gas_used:          1,
        reveal:            "reveal".as_bytes().into(),
        proxy_public_keys: Default::default(),
    };
    test_info
        .map
        .insert_reveal(&mut test_info.store, &key, identity, reveal_body.clone())
        .unwrap();

    test_info.assert_dr_reveals_len(1, &key);

    let reveal = test_info
        .map
        .get_reveal(&test_info.store, &key, identity)
        .unwrap()
        .unwrap();
    assert_eq!(reveal, reveal_body);

    let reveals = test_info.map.get_reveals(&test_info.store, &key).unwrap();
    assert_eq!(reveals.len(), 1);

    test_info.map.remove(&mut test_info.store, &key).unwrap();
}
