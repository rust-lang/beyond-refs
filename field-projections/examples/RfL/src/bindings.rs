#![expect(nonstandard_style)]

pub struct mutex;

pub unsafe fn mutex_lock(_: *mut mutex) {}
pub unsafe fn mutex_unlock(_: *mut mutex) {}

pub unsafe fn synchronize_rcu() {}
