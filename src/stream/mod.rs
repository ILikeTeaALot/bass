pub mod device;
pub mod dummy;
pub mod maybe;
pub mod push;

use std::{
	fmt::Debug, hash::Hash, ops::DerefMut, os::raw::{c_char, c_void}, ptr::null_mut, slice, sync::{Arc, Mutex, MutexGuard, Weak}
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
enum MemoryDataOrDownloadProc<T: Send + Sync> {
	#[allow(dead_code)]
	MemoryStream(Vec<u8>),
	#[allow(dead_code)]
	DownloadProc(Arc<Mutex<DownloadProc<T>>>),
}

#[derive(Debug)]
pub struct Stream<T: Send + Sync>(
	HSTREAM,
	/// It is required for a "memory stream" to hold onto the data it is streaming.
	#[allow(dead_code)]
	Option<MemoryDataOrDownloadProc<T>>,
);

#[repr(C)]
pub struct DownloadProc<T: Send + Sync> {
	callback: Box<dyn FnMut(&[u8], &mut T) + Send + Sync + 'static>,
	user: Box<T>,
}

impl<T: Send + Sync> Debug for DownloadProc<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("DownloadProc<T>")
			.field("callback", &"Box<dyn Fn(&[u8], &mut T)>")
			.field("user", &"Box<T>")
			.finish()
	}
}

extern "C" fn download_proc<T: Send + Sync>(buffer: *const c_void, length: DWORD, user: *mut c_void) {
	// let mut user_box = unsafe { Box::from_raw(user as *mut DownloadProc<T>) };
	// let data = unsafe { slice::from_raw_parts(buffer as *const u8, (length.0 / 4) as usize) };
	// (user_box.callback)(data, user_box.user.as_mut());
	// Box::into_raw(user_box);
	let f = |mut user_box: MutexGuard<'_, DownloadProc<T>>| {
		#[cfg(debug_assertions)]
		println!("deref_mut'ing DownloadProc...");
		let user_box = user_box.deref_mut();
		#[cfg(debug_assertions)]
		println!("Running user DownloadProc...");
		let data = unsafe { slice::from_raw_parts(buffer as *const u8, (length.0 / 4) as usize) };
		(user_box.callback)(data, user_box.user.as_mut());
		#[cfg(debug_assertions)]
		println!("Success!");
	};
	// Attempt to upgrade the weak pointer, failing gracefully if it has already been dropped.
	match unsafe { Weak::from_raw(user as *const Mutex<DownloadProc<T>>) }.upgrade() {
		Some(arc) => {
			#[cfg(debug_assertions)]
			println!("Strong: {}; Weak: {}", Arc::strong_count(&arc), Arc::weak_count(&arc));
			// Handle mutex locking.
			#[cfg(debug_assertions)]
			println!("Locking Mutex...");
			match arc.lock() {
				Ok(user_box) => f(user_box),
				Err(e) => f(e.into_inner()),
			}
			#[cfg(debug_assertions)]
			println!("Resetting weak count...");
			// Reset the weak count.
			let weak = Arc::downgrade(&arc);
			// Equivalent to std::mem::forget in a way...
			let _ = weak.into_raw();
			#[cfg(debug_assertions)]
			println!("Hopefully success?");
		}
		None => {
			#[cfg(debug_assertions)]
			println!("User data freed")
		}
	}
	#[cfg(debug_assertions)]
	println!("Everything should drop now.");
}

impl Stream<()> {
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
			Ok(Self(handle, Some(MemoryDataOrDownloadProc::MemoryStream(data))))
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

impl<T: Send + Sync> Stream<T> {
	pub fn create_url_download_proc(
		path: impl AsRef<str>,
		offset: impl Into<DWORD>,
		flags: DWORD,
		callback: impl FnMut(&[u8], &mut T) + Send + Sync + 'static,
		user: T,
	) -> BassResult<Self> {
		// let callback = Box::new(callback) as Box<dyn Fn(&[u8], &mut T) + Send + Sync>;
		let callback = Box::new(callback);
		let url: Vec<u16> = path.as_ref().encode_utf16().collect();
		let url = U16CString::from_vec_truncate(url);
		let user = Arc::new(Mutex::new(DownloadProc { callback, user: Box::new(user) }));
		let weak = Arc::downgrade(&user);
		let handle = unsafe {
			BASS_StreamCreateURL(
				url.as_ptr() as *const c_char,
				offset,
				flags | BASS_UNICODE,
				Some(download_proc::<T>),
				Weak::into_raw(weak) as *mut Weak<Mutex<DownloadProc<T>>>,
			)
		};
		if handle != 0 {
			Ok(Self(handle, Some(MemoryDataOrDownloadProc::DownloadProc(user))))
		} else {
			Err(BassError::get())
		}
	}
}

impl<T: Send + Sync> HasHandle for Stream<T> {
	fn handle(&self) -> DWORD {
		self.0 .0
	}
}

impl<T: Send + Sync> Channel for Stream<T> {}

#[cfg(feature = "mixer")]
impl<T: Send + Sync> crate::channel::mixer::MixableChannel for Stream<T> {}
#[cfg(feature = "mixer")]
impl<T: Send + Sync> crate::channel::MixerSource for Stream<T> {}

impl<T: Send + Sync> Drop for Stream<T> {
	fn drop(&mut self) {
		#[cfg(debug_assertions)]
		println!("Freeing Stream {:?}", self.0);
		BASS_StreamFree(self.0); // Only reason it can fail is if the stream has already been freed
	}
}

impl<T: Send + Sync> Hash for Stream<T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.0.hash(state)
	}
}

impl<T: Send + Sync> PartialEq for Stream<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}
impl<T: Send + Sync> Eq for Stream<T> {}
