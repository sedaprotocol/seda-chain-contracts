use testing::MockStorage;

use super::*;

struct TestInfo<'a> {
    pub store:    MockStorage,
    pub timeouts: Timeouts<'a>,
}

impl TestInfo<'_> {
    fn init() -> Self {
        let store = MockStorage::new();
        let timeouts: Timeouts = Timeouts {
            timeouts:        Map::new("timeouts"),
            hash_to_timeout: Map::new("hash_to_timeout"),
        };
        Self { store, timeouts }
    }

    #[track_caller]
    fn insert(&mut self, timeout: u64, dr_id: Hash) {
        self.timeouts.insert(&mut self.store, timeout, &dr_id).unwrap();
    }

    #[track_caller]
    fn remove_by_dr_id(&mut self, dr_id: Hash) {
        self.timeouts.remove_by_dr_id(&mut self.store, &dr_id).unwrap();
    }

    #[track_caller]
    fn remove_by_timeout_height(&mut self, timeout_height: u64) {
        self.timeouts
            ._remove_by_timeout_height(&mut self.store, timeout_height)
            .unwrap();
    }

    #[track_caller]
    fn get_timeout_by_dr_id(&self, dr_id: Hash) -> u64 {
        self.timeouts.get_timeout_by_dr_id(&self.store, &dr_id).unwrap()
    }

    #[track_caller]
    fn get_all_by_timeout_height(&self, timeout_height: u64) -> Vec<Hash> {
        self.timeouts
            ._get_all_by_timeout_height(&self.store, timeout_height)
            .unwrap()
    }
}

#[test]
fn insert() {
    let mut info = TestInfo::init();
    let timeout = 1000;
    let dr_id = [1; 32];

    info.insert(timeout, dr_id);

    assert_eq!(info.get_timeout_by_dr_id(dr_id), timeout);
}

#[test]
fn get_by_dr_id() {
    let mut info = TestInfo::init();
    let timeout = 1000;
    let dr_id = [1; 32];

    info.insert(timeout, dr_id);

    assert_eq!(info.get_timeout_by_dr_id(dr_id), timeout);
}

#[test]
fn get_all_by_timeout_height() {
    let mut info = TestInfo::init();
    let timeout = 1000;
    let num = 10;
    let mut ids = vec![];
    (0..num).for_each(|i| {
        ids.push([i; 32]);
        info.insert(timeout, [i; 32]);
    });

    let retrieved = info.get_all_by_timeout_height(timeout);
    assert_eq!(retrieved.len(), num as usize);
    ids.iter().for_each(|id| {
        assert!(retrieved.contains(id));
    });
}

#[test]
fn remove_by_dr_id() {
    let mut info = TestInfo::init();
    let timeout = 1000;
    let dr_id = [1; 32];

    info.insert(timeout, dr_id);
    info.remove_by_dr_id(dr_id);

    assert_eq!(info.get_all_by_timeout_height(timeout).len(), 0);
}

#[test]
fn remove_by_timeout_height() {
    let mut info = TestInfo::init();
    let timeout = 1000;
    let num = 10;
    (0..num).for_each(|i| {
        info.insert(timeout, [i; 32]);
    });

    let retrieved = info.get_all_by_timeout_height(timeout);
    assert_eq!(retrieved.len(), num as usize);

    info.remove_by_timeout_height(timeout);
    assert_eq!(info.get_all_by_timeout_height(timeout).len(), 0);
}

#[test]
fn remove_one_others_stay() {
    let mut info = TestInfo::init();
    let timeout = 1000;
    let num = 10;
    let mut ids = vec![];
    (0..num).for_each(|i| {
        ids.push([i; 32]);
        info.insert(timeout, [i; 32]);
    });

    let retrieved = info.get_all_by_timeout_height(timeout);
    assert_eq!(retrieved.len(), num as usize);

    let to_remove = ids[0];
    info.remove_by_dr_id(to_remove);

    let retrieved = info.get_all_by_timeout_height(timeout);
    assert_eq!(retrieved.len(), num as usize - 1);
    assert!(!retrieved.contains(&to_remove));
}

#[test]
fn remove_one_timeout_from_two() {
    let mut info = TestInfo::init();
    let timeout1 = 1000;
    let timeout2 = 2000;
    let num = 10;
    let mut ids = vec![];
    (0..num).for_each(|i| {
        ids.push([i; 32]);
        info.insert(timeout1, [i; 32]);
        info.insert(timeout2, [i; 32]);
    });

    let retrieved1 = info.get_all_by_timeout_height(timeout1);
    assert_eq!(retrieved1.len(), num as usize);

    let retrieved2 = info.get_all_by_timeout_height(timeout2);
    assert_eq!(retrieved2.len(), num as usize);

    info.remove_by_timeout_height(timeout1);

    let retrieved1 = info.get_all_by_timeout_height(timeout1);
    assert_eq!(retrieved1.len(), 0);

    let retrieved2 = info.get_all_by_timeout_height(timeout2);
    assert_eq!(retrieved2.len(), num as usize);
}
