pub mod device;
pub mod dummy;
pub mod maybe;
pub mod push;

use std::{
	hash::Hash,
	os::raw::{c_char, c_void},
	ptr::null_mut,
	slice,
};

use bass_sys::{
	BASS_SampleGetChannel, BASS_StreamCreateFile, BASS_StreamCreateURL, BASS_StreamFree, BASS_StreamGetFilePosition,
	BASS_StreamPutData, BASS_SAMCHAN_STREAM, BASS_UNICODE, DWORD, HSAMPLE, HSTREAM, QWORD,
};
use widestring::U16CString;

use crate::{
	bass::error::BassError,
	channel::{handle::HasHandle, Channel},
	BassResult,
};

#[derive(Debug)]
pub struct Stream(
	HSTREAM,
	#[allow(dead_code)]
	/// It is required for a "memory stream" to hold onto the data it is streaming.
	Option<Vec<u8>>,
);

#[repr(C)]
pub struct DownloadProc<T> {
	callback: Box<dyn Fn(&[u8], &mut T)>,
	user: Box<T>,
}

extern "C" fn download_proc<T>(buffer: *const c_void, length: DWORD, user: *mut c_void) {
	let mut user_box = unsafe { Box::from_raw(user as *mut DownloadProc<T>) };
	let data = unsafe { slice::from_raw_parts(buffer as *const u8, (length.0 / 4) as usize) };
	(user_box.callback)(data, user_box.user.as_mut());
	Box::into_raw(user_box);
}

impl Stream {
	pub fn create() {}

	pub fn create_file(
		path: impl AsRef<str>,
		offset: impl Into<QWORD>,
		length: impl Into<QWORD>,
		flags: DWORD,
	) -> BassResult<Self> {
		let file: Vec<u16> = path.as_ref().encode_utf16().collect();
		let file = U16CString::from_vec_truncate(file);
		let handle = unsafe {
			BASS_StreamCreateFile(false, file.as_ptr() as *const c_void, offset, length, flags | BASS_UNICODE)
		};
		if handle != 0 {
			Ok(Self(handle, None))
		} else {
			Err(BassError::get())
		}
	}

	pub fn create_file_mem(
		data: impl Into<Vec<u8>>,
		offset: impl Into<QWORD>,
		length: impl Into<QWORD>,
		flags: DWORD,
	) -> BassResult<Self> {
		let data: Vec<u8> = data.into();
		let handle = unsafe {
			BASS_StreamCreateFile(true, data.as_ptr() as *const c_void, offset, length, flags | BASS_UNICODE)
		};
		if handle != 0 {
			Ok(Self(handle, Some(data)))
		} else {
			Err(BassError::get())
		}
	}

	pub fn create_file_user() {}

	pub fn create_url(path: impl AsRef<str>, offset: impl Into<DWORD>, flags: DWORD) -> BassResult<Self> {
		let url: Vec<u16> = path.as_ref().encode_utf16().collect();
		let url = U16CString::from_vec_truncate(url);
		let handle = unsafe {
			BASS_StreamCreateURL(
				url.as_ptr() as *const c_char,
				offset,
				flags | BASS_UNICODE,
				None,
				null_mut::<c_void>(),
			)
		};
		if handle != 0 {
			Ok(Self(handle, None))
		} else {
			Err(BassError::get())
		}
	}

	pub fn create_url_download_proc<T: Send + Sync>(
		path: impl AsRef<str>,
		offset: impl Into<DWORD>,
		flags: DWORD,
		callback: impl Fn(&[u8], &mut T) + 'static,
		user: T,
	) -> BassResult<Self> {
		let callback = Box::new(callback) as Box<dyn Fn(&[u8], &mut T)>;
		let url: Vec<u16> = path.as_ref().encode_utf16().collect();
		let url = U16CString::from_vec_truncate(url);
		let handle = unsafe {
			BASS_StreamCreateURL(
				url.as_ptr() as *const c_char,
				offset,
				flags | BASS_UNICODE,
				Some(download_proc::<T>),
				Box::into_raw(Box::new(DownloadProc { callback, user: Box::new(user) })),
			)
		};
		if handle != 0 {
			Ok(Self(handle, None))
		} else {
			Err(BassError::get())
		}
	}

	pub(crate) fn from_sample(handle: HSAMPLE, flags: DWORD) -> BassResult<Self> {
		let ok = BASS_SampleGetChannel(handle, flags | BASS_SAMCHAN_STREAM);
		if let Some(handle) = ok {
			Ok(Self(HSTREAM(handle), None))
		} else {
			Err(BassError::get())
		}
	}

	pub fn file_position(&self, mode: DWORD) -> u64 {
		BASS_StreamGetFilePosition(self.0, mode).0
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

impl HasHandle for Stream {
	fn handle(&self) -> DWORD {
		self.0 .0
	}
}

impl Channel for Stream {}

#[cfg(feature = "mixer")]
impl crate::channel::mixer::MixableChannel for Stream {}
#[cfg(feature = "mixer")]
impl crate::channel::MixerSource for Stream {}

impl Drop for Stream {
	fn drop(&mut self) {
		#[cfg(debug_assertions)]
		println!("Freeing Stream {:?}", self.0);
		BASS_StreamFree(self.0); // Only reason it can fail is if the stream has already been freed
	}
}

impl Hash for Stream {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.0.hash(state)
	}
}

impl PartialEq for Stream {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}
impl Eq for Stream {}
