use Tree;
use rand::{self, Rng};

quickcheck! {
    fn insert_then_iterate(keys: Vec<u32>) -> bool {
        let mut keys = keys;
        keys.sort();
        keys.dedup();
        let reference = keys.clone();

        rand::thread_rng().shuffle(&mut keys);
        let mut tree = Tree::new();
        for key in keys {
            tree = tree.insert(key, key)
        }
        let results: Vec<u32> = tree.iter().map(|(k, _)| *k).collect();
        reference == results
    }
}

