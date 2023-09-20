use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub trait EasyAtomic {
    type Item;

    fn get(&self) -> Self::Item;
    fn set(&self, value: Self::Item);
}

impl EasyAtomic for AtomicBool {
    type Item = bool;

    fn get(&self) -> Self::Item {
        self.load(Ordering::Relaxed)
    }

    fn set(&self, value: Self::Item) {
        self.store(value, Ordering::Relaxed)
    }
}

impl EasyAtomic for AtomicU32 {
    type Item = u32;

    fn get(&self) -> Self::Item {
        self.load(Ordering::Relaxed)
    }

    fn set(&self, value: Self::Item) {
        self.store(value, Ordering::Relaxed)
    }
}
