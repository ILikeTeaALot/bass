use std::{
	ffi::{c_char, c_double, c_void},
	fmt::Debug,
	sync::Mutex,
};
use std::ops::Deref;

use bass_sys::*;
use util::safe_lock::SafeLock;
use widestring::U16CString;

use crate::{
	error::BassError,
	null::NULL,
	result::{result, BassResult},
	syncproc::CallbackUserData,
	types::proc::{BassFileProcs, FileProcHandlers},
};
use crate::channel::Channel;

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
#[repr(u32)]
/// This is only marked as Non-exhaustive because BASS is an external library.
pub enum StreamSystem {
	NoBuffer = STREAMFILE_NOBUFFER.0,
	Buffer = STREAMFILE_BUFFER.0,
	BufferPush = STREAMFILE_BUFFERPUSH.0,
}

#[derive(Clone, Copy, Debug)]
pub enum StreamType {
	Sample,
	File,
	FileMem,
	FileUser,
	URL,
}

/// The inner members of these are technically unused, but need to be owned for memory safety reasons.
enum StreamData {
	Memory(Vec<u8>),
	FileUser { user: *mut FileProcHandlers<CallbackUserData>, dropper: Box<dyn Fn(Box<FileProcHandlers<CallbackUserData>>) + Send + Sync> },
}

/// TODO: Verify - It should be (StreamData is private and only ever used with `T: Send + Sync` but I am an ameteur)
unsafe impl Send for StreamData {}
/// TODO: Verify - It should be (StreamData is private and only ever used with `T: Send + Sync` but I am an ameteur)
unsafe impl Sync for StreamData {}

impl Debug for StreamData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			StreamData::Memory(data) => f.debug_tuple("StreamData").field(data).finish(),
			StreamData::FileUser { user: data, dropper } => {
				f.debug_struct("StreamData").field("data", data).field("dropper", &"DropperFn").finish()
			}
		}
	}
}

#[derive(Debug)]
pub struct Stream {
	device: DWORD,
	handle: HSTREAM,
	init_flags: DWORD,
	stream_type: StreamType,
	/// This has to be kept around for memory safety reasons.
	///
	/// I'd like to find a way to be able to destroy the stream and return the memory for re-use.
	#[allow(unused)]
	data: Option<StreamData>,
}

pub type SuperPosition = (QWORD, f64);

impl Stream {
	fn new_internal(
		stream_type: StreamType,
		handle: HSTREAM,
		flags: DWORD,
		device: Option<impl Into<DWORD>>,
		data: Option<StreamData>,
	) -> BassResult<Self> {
		// let device = BASS_ChannelGetDevice(handle);
		let device = match device {
			Some(device) => {
				let device = device.into();
				let ok = BASS_ChannelSetDevice(handle, device);
				if !ok {
					eprintln!("BASS Error: {:#?}", BassError::default());
					// panic!();
				}
				device
			}
			None => {
				let device = BASS_ChannelGetDevice(handle);
				if *device as i32 == -1 {
					// throw new BassError();
					eprintln!("BASS Error: {:#?}", BassError::default());
					// panic!();
				}
				device
			}
		};
		Ok(Self { data, device, init_flags: flags, handle, stream_type })
	}

	fn new_file_internal(path: impl AsRef<str>, flags: impl Into<DWORD>, offset: impl Into<QWORD>, length: impl Into<QWORD>, device: Option<impl Into<DWORD>>) -> BassResult<Self> {
		let flags = flags.into();
		let path = U16CString::from_str(path).unwrap();
		// let seconds = Seconds(1.);
		// let bytes = Bytes(800402);
		let handle =
			unsafe { BASS_StreamCreateFile(false, path.as_ptr() as *const c_void, offset, length, flags | BASS_UNICODE) };
		if handle == 0 {
			let error = BassError::default();
			eprintln!("{}", error);
			eprintln!("Path: {}", path.to_string_lossy());
			eprintln!("Flags: {}", flags);
			eprintln!("Stream type: {:?}", StreamType::File);
			eprintln!("Stream failed to be created!");
			return Err(error);
		}
		Self::new_internal(StreamType::File, handle, flags, device, None)
	}

	/// The default stream type. A file stream not from memory.
	///
	/// The BASS library handles opening and managing the file.
	pub fn new(path: impl AsRef<str>, flags: impl Into<DWORD>, device: Option<impl Into<DWORD>>) -> BassResult<Self> {
		Self::new_file_internal(path, flags, 0, 0, device)
	}

	/// Like the BASS docs (which you should read), offset and length can be set to 0 to be left default.
	pub fn new_file_ex(path: impl AsRef<str>, flags: impl Into<DWORD>, offset: impl Into<QWORD>, length: impl Into<QWORD>, device: Option<impl Into<DWORD>>) -> BassResult<Self> {
		Self::new_file_internal(path, flags, offset, length, device)
	}

	pub fn new_url(
		path: impl AsRef<str>,
		flags: impl Into<DWORD>,
		device: Option<impl Into<DWORD>>,
	) -> BassResult<Self> {
		let flags = flags.into();
		let path = U16CString::from_str(path).unwrap();
		let handle =
			unsafe { BASS_StreamCreateURL(path.as_ptr() as *const c_char, 0, flags | BASS_UNICODE, None, NULL) };
		if handle == 0 {
			let error = BassError::default();
			eprintln!("{}", error);
			eprintln!("Path: {}", path.to_string_lossy());
			eprintln!("Flags: {}", flags);
			eprintln!("Stream type: {:?}", StreamType::URL);
			eprintln!("Stream failed to be created!");
			return Err(error);
		}
		Self::new_internal(StreamType::URL, handle, flags, device, None)
	}

	/// Create a new stream from file/audio data in memory.
	///
	/// We take the memory as owned because it's passed over an FFI boundary by this function.
	pub fn new_mem(memory: Vec<u8>, flags: impl Into<DWORD>, device: Option<impl Into<DWORD>>) -> BassResult<Self> {
		let flags = flags.into();
		let handle =
			unsafe { BASS_StreamCreateFile(true, memory.as_ptr() as *const c_void, 0, memory.len(), flags | BASS_UNICODE) };
		if handle == 0 {
			let error = BassError::default();
			eprintln!("{}", error);
			eprintln!("Flags: {}", flags);
			eprintln!("Stream type: {:?}", StreamType::FileMem);
			eprintln!("Stream failed to be created!");
			return Err(error);
		}
		// We then store the `Vec<u8>` stream data with the `Stream`
		Self::new_internal(StreamType::FileMem, handle, flags, device, Some(StreamData::Memory(memory)))
	}

	pub fn new_mem_static(memory: &'static [u8], flags: impl Into<DWORD>, device: Option<impl Into<DWORD>>) -> BassResult<Self> {
		let flags = flags.into();
		let handle =
			unsafe { BASS_StreamCreateFile(true, memory.as_ptr() as *const c_void, 0, memory.len(), flags | BASS_UNICODE) };
		if handle == 0 {
			let error = BassError::default();
			eprintln!("{}", error);
			eprintln!("Flags: {}", flags);
			eprintln!("Stream type: {:?}", StreamType::FileMem);
			eprintln!("Stream failed to be created!");
			return Err(error);
		}
		// We then DO NOT store the data becuase it's a reference to static data.
		Self::new_internal(StreamType::FileMem, handle, flags, device, None)
	}

	/// This was bad anyway
	// pub fn new_user<T: Send + Sync>(
	// 	system: StreamSystem,
	// 	flags: impl Into<DWORD>,
	// 	device: Option<impl Into<DWORD>>,
	// 	// user: T,
	// 	// procs: BassFileProcs<T>,
	// 	// proc_handler: Box<dyn FileProc<UserData = T>>,
	// 	user: FileProcHandlers<T>,
	// ) -> BassResult<Self> {
	// 	let flags = flags.into();
	// 	let procs = BassFileProcs::from(&user);
	// 	// This cruft is necessary in order to run the correct `mem::drop` for the data stored in the `Box`.
	// 	// There is some seriously hideous, but afaik sound, code here...
	// 	let user = Box::into_raw(Box::new(user));
	// 	let user = unsafe {
	// 		Box::from_raw(user as *mut FileProcHandlers<CallbackUserData>)
	// 	};
	// 	let user = Box::into_raw(user);
	// 	let dropper = Box::into_raw(Box::new(|to_drop: Box<FileProcHandlers<T>>| {
	// 		drop(to_drop)
	// 	}) as Box<dyn Fn(Box<FileProcHandlers<T>>) + Send + Sync + 'static>);
	// 	let dropper = unsafe {
	// 		Box::from_raw(dropper as *mut (dyn Fn(Box<FileProcHandlers<CallbackUserData>>) + Send + Sync + 'static))
	// 	};
	// 	// let user = Box::into_raw(Box::new(Arc::new(user)));
	// 	let handle = BASS_StreamCreateFileUser(DWORD(system as u32), flags, &procs, user);
	// 	// let handle = HSTREAM(DWORD(0)); // TODO
	// 	Self::new_internal(StreamType::FileUser, handle, flags, device, Some(StreamData::FileUser{user, dropper}))
	// }

	pub fn handle(&self) -> HSTREAM {
		self.handle
	}

	// pub fn flags(&self) -> DWORD {
	// 	self.flags.safe_lock().clone()
	// }

	// pub fn set_flags(&self, flags: DWORD) -> BassResult<()> {
	// 	Ok(())
	// }

	pub fn device(&self) -> DWORD {
		self.device
	}

	pub fn set_device(&mut self, device: impl Into<DWORD>) -> BassResult<()> {
		let device = device.into();
		let ok = BASS_ChannelSetDevice(*self.handle, device);
		if ok {
			self.device = device;
			Ok(())
		} else {
			Err(BassError::default())
		}
	}

	pub fn stream_type(&self) -> StreamType {
		self.stream_type
	}

	pub fn duration(&self) -> Result<QWORD, BassError> {
		let bytes = BASS_ChannelGetLength(*self.handle, BASS_POS_BYTE);
		if *bytes as i64 == -1 {
			return Err(BassError::default());
		}
		Ok(bytes)
	}

	pub fn duration_seconds(&self) -> Result<f64, BassError> {
		if let Ok(bytes) = self.duration() {
			let length_seconds = BASS_ChannelBytes2Seconds(*self.handle, bytes);
			Ok(length_seconds)
		} else {
			Err(BassError::default())
		}
	}

	pub fn position(&self) -> Result<f64, BassError> {
		let bytes = BASS_ChannelGetPosition(*self.handle, BASS_POS_BYTE);
		if *bytes as i64 == -1 {
			return Err(BassError::default());
		}
		let seconds = BASS_ChannelBytes2Seconds(*self.handle, bytes);
		if seconds < 0. {
			return Err(BassError::default());
		}
		Ok(seconds)
	}

	pub fn set_position(&self, seconds: c_double) -> BassResult<()> {
		let bytes = BASS_ChannelSeconds2Bytes(*self.handle, seconds);
		if *bytes as i64 == -1 {
			return Err(BassError::default());
		}
		self.set_position_bytes(bytes)
	}

	pub fn set_position_bytes(&self, bytes: impl Into<QWORD>) -> BassResult<()> {
		let ok = BASS_ChannelSetPosition(*self.handle, bytes, BASS_POS_BYTE | BASS_POS_SCAN);
		result(ok)
	}

	#[cfg(feature = "mixer")]
	pub fn set_mixer_position(&self, seconds: c_double) -> BassResult<()> {
		let bytes = BASS_ChannelSeconds2Bytes(*self.handle, seconds);
		if *bytes as i64 == -1 {
			return Err(BassError::default());
		}
		self.set_mixer_position_bytes(bytes)
	}

	#[cfg(feature = "mixer")]
	pub fn set_mixer_position_bytes(&self, bytes: impl Into<QWORD>) -> BassResult<()> {
		let ok = BASS_Mixer_ChannelSetPosition(*self.handle, bytes, BASS_POS_BYTE | BASS_POS_SCAN);
		result(ok)
	}
}

impl Drop for Stream {
	fn drop(&mut self) {
		println!("Freeing Stream: {}", **self.handle);
		BASS_ChannelFree(*self.handle);
		match self.data.as_mut() {
			Some(data) => match data {
				// Doesn't matter, it's a Vec, it drops itself!... right?
				StreamData::Memory(_) => (),
				StreamData::FileUser{user: raw, dropper} => {
					let data = unsafe { Box::from_raw(*raw) };
					dropper(data)
				}
			},
			// Must have been created as a Network or File stream, nothing else to manually clean up.
			None => (),
		}
	}
}

impl Deref for Stream {
	type Target = HSTREAM;

	fn deref(&self) -> &Self::Target {
		&self.handle
	}
}

// impl From<HSTREAM> for Stream {
// 	fn from(value: HSTREAM) -> Self {
// 		let channel = Channel(value);
// 		let info = channel.get_info();
// 		Stream {
// 			device: BASS_ChannelGetDevice(value),
// 			handle: channel,
// 			flags: Mutex::new(info.flags),
// 			stream_type: StreamType::Sample,
// 			data: None,
// 		}
// 	}
// }