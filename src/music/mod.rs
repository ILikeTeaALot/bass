use std::os::raw::c_void;

use bass_sys::{BASS_MusicLoad, BASS_UNICODE, DWORD, HMUSIC, QWORD};
use widestring::U16CString;

use crate::{
	bass::error::BassError,
	channel::{handle::HasHandle, Channel},
	BassResult,
};

#[derive(Debug)]
pub struct Music(HMUSIC);

impl Music {
	pub fn load(
		path: impl AsRef<str>,
		offset: impl Into<QWORD>,
		length: impl Into<DWORD>,
		flags: DWORD,
		frequency: impl Into<DWORD>,
	) -> BassResult<Self> {
		let file: Vec<u16> = path.as_ref().encode_utf16().collect();
		let file = U16CString::from_vec_truncate(file);
		let handle = unsafe {
			BASS_MusicLoad(false, file.as_ptr() as *const c_void, offset, length, flags | BASS_UNICODE, frequency)
		};
		if handle != 0 {
			Ok(Self(handle))
		} else {
			Err(BassError::get())
		}
	}

	pub fn load_memory(data: &[u8], flags: DWORD, frequency: impl Into<DWORD>) -> BassResult<Self> {
		let handle =
			unsafe { BASS_MusicLoad(true, data.as_ptr() as *const c_void, 0, data.len(), flags | BASS_UNICODE, frequency) };
		if handle != 0 {
			Ok(Self(handle))
		} else {
			Err(BassError::get())
		}
	}
}

impl HasHandle for Music {
	fn handle(&self) -> DWORD {
		self.0 .0
	}
}

impl Channel for Music {}

#[cfg(feature = "mixer")]
impl crate::channel::mixer::MixableChannel for Music {}
#[cfg(feature = "mixer")]
impl crate::channel::MixerSource for Music {}
