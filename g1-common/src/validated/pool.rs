use derivative::Derivative;
use std::{collections::HashMap, hash::Hash, ops::AddAssign};

/// A pool for variables, assigning variables with the same name the same index.
#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub struct Pool<K: Eq + Hash, V: AddAssign + Clone + Default + From<u8>> {
    pub pool: HashMap<K, V>,
    pub next_index: V,
}

impl<K: Eq + Hash, V: AddAssign + Clone + Default + From<u8>> Pool<K, V> {
    pub fn intern(&mut self, key: K) -> V {
        let pool = &mut self.pool;
        let next_index = &mut self.next_index;

        pool.entry(key)
            .or_insert_with(|| {
                let n = next_index.clone();
                *next_index += 1.into();
                n
            })
            .clone()
    }

    pub fn intern_dummy(&mut self) -> V {
        let n = self.next_index.clone();
        self.next_index += 1.into();
        n
    }
}
