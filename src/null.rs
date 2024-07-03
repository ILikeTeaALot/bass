use std::{ffi::c_void, ptr::{null, null_mut}};

pub type Null = *mut c_void;
pub const NULL: *mut c_void = null_mut::<c_void>();
pub const CONST_NULL: *const c_void = null::<c_void>();
