use std::{os::raw::c_void, ptr::null_mut};

use bass_sys::{BASS_StreamCreate, HSTREAM, STREAMPROC_DUMMY};

use crate::{bass::error::BassError, BassResult};

#[derive(Debug)]
#[allow(unused)]
pub struct DummyStream(HSTREAM);

impl DummyStream {
	pub fn create() -> BassResult<Self> {
		let handle = BASS_StreamCreate(0, 0, 0, STREAMPROC_DUMMY, null_mut::<c_void>());
		if handle != 0 {
			Ok(Self(handle))
		} else {
			Err(BassError::get())
		}
	}
}
