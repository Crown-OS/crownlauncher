use std::sync::{Mutex, MutexGuard, PoisonError};

/// Locks `mutex`, and takes its value over from a thread that panicked while it held the lock.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
