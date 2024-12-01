use std::{ffi::c_void, ptr::null_mut};

use bass_sys::{BASS_StreamCreate, BASS_StreamFree, BASS_StreamPutData, DWORD, HSTREAM, STREAMPROC_PUSH};

use crate::{bass::error::BassError, BassResult};

#[derive(Debug)]
pub struct PushStream(HSTREAM);

impl PushStream {
	pub fn create(frequency: impl Into<DWORD>, channels: impl Into<DWORD>, flags: DWORD) -> BassResult<Self> {
		let handle = BASS_StreamCreate(frequency, channels, flags, *STREAMPROC_PUSH, null_mut::<c_void>());
		if handle != 0 {
			Ok(Self(handle))
		} else {
			Err(BassError::get())
		}
	}

	pub fn put_data(&self, data: &[u8]) -> BassResult<usize> {
		let inserted = unsafe { BASS_StreamPutData(self.0, data.as_ptr() as *const c_void, data.len()) };
		if inserted.0 as i32 != -1 {
			Ok(inserted.0 as usize)
		} else {
			Err(BassError::get())
		}
	}
}

impl Drop for PushStream{
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
		println!("Freeing Stream {:?}", self.0);
		BASS_StreamFree(self.0); // Only reason it can fail is if the stream has already been freed
    }
}