use std::{
	ffi::c_void,
	fmt::Debug,
};
use std::ops::Deref;

use bass_sys::*;
use util::safe_lock::SafeLock;
use widestring::U16CString;

use crate::{
	error::BassError,
	result::BassResult,
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
pub enum SampleType {
	File,
	Memory,
}

// /// The inner members of these are technically unused, but need to be owned for memory safety reasons.
// #[derive(Debug)]
// enum SampleData {
// 	Memory(Vec<u8>),
// }
//
// /// TODO: Verify - It should be (StreamData is private and only ever used with `T: Send + Sync` but I am an ameteur)
// unsafe impl Send for SampleData {}
// /// TODO: Verify - It should be (StreamData is private and only ever used with `T: Send + Sync` but I am an ameteur)
// unsafe impl Sync for SampleData {}

#[derive(Debug)]
/// By default, when a Sample is created from data in-memory, the memory is retained; however,
/// BASS creates a copy of the data itself, so this is not strictly necessary.
pub struct Sample {
	device: DWORD,
	handle: Channel<HSAMPLE>,
	init_flags: DWORD,
	stream_type: SampleType,
	// /// This has to be kept around for memory safety reasons.
	// ///
	// /// I'd like to find a way to be able to destroy the stream and return the memory for re-use.
	// #[allow(unused)]
	// data: Option<StreamData>,
}

pub type SuperPosition = (QWORD, f64);

impl Sample {
	fn new_internal(
		stream_type: SampleType,
		handle: HSAMPLE,
		flags: DWORD,
		device: Option<impl Into<DWORD>>,
		// data: Option<StreamData>,
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
		Ok(Self { /*data,*/ device, init_flags: flags, handle: Channel(handle), stream_type })
	}

	pub fn new_file(
		path: impl AsRef<str>,
		flags: impl Into<DWORD>,
		offset: impl Into<QWORD>,
		length: Option<DWORD>,
		maximum: impl Into<DWORD>,
		device: Option<impl Into<DWORD>>,
	) -> BassResult<Self> {
		let flags = flags.into();
		let path = U16CString::from_str(path).unwrap();
		let handle =
			unsafe { BASS_SampleLoad(false, path.as_ptr() as *const c_void, offset, length.unwrap_or(DWORD(0)), maximum, flags | BASS_UNICODE) };
		if handle == 0 {
			let error = BassError::default();
			eprintln!("{}", error);
			eprintln!("Path: {}", path.to_string_lossy());
			eprintln!("Flags: {}", flags);
			eprintln!("Stream type: {:?}", SampleType::File);
			eprintln!("Stream failed to be created!");
			return Err(error);
		}
		Self::new_internal(SampleType::File, handle, flags, device)
	}

	/// Create a new stream from file/audio data in memory.
	///
	/// Memory is copied by BASS, so anything able to be passed by reference as &[u8] is acceptable.
	pub fn new_memory(
		memory: &[u8],
		flags: impl Into<DWORD>,
		offset: impl Into<QWORD>,
		length: Option<impl Into<DWORD>>,
		maximum: impl Into<DWORD>,
		device: Option<impl Into<DWORD>>,
	) -> BassResult<Self> {
		let flags = flags.into();
		let handle =
			unsafe {
				BASS_SampleLoad(
					true,
					memory.as_ptr() as *const c_void,
					offset,
					length.map(|v| v.into()).map(|v| if v > memory.len() as u32 {
						memory.len().into()
					} else {
						v
					})
						.unwrap_or(memory.len().into()),
					maximum,
					flags | BASS_UNICODE
				)
			};
		if handle == 0 {
			let error = BassError::default();
			eprintln!("{}", error);
			eprintln!("Flags: {}", flags);
			eprintln!("Stream type: {:?}", SampleType::Memory);
			eprintln!("Stream failed to be created!");
			return Err(error);
		}
		// We then store the `Vec<u8>` stream data with the `Stream`
		Self::new_internal(SampleType::Memory, handle, flags, device)
	}

	pub fn get_channel(&self, flags: impl Into<DWORD>) -> Option<DWORD> {
		BASS_SampleGetChannel(self.handle.0, flags)
	}

	// pub fn handle(&self) -> HSAMPLE {
	// 	*self.handle
	// }

	// pub fn flags(&self) -> DWORD {
	// 	self.flags.safe_lock().clone()
	// }
	//
	// pub fn set_flags(&self, flags: DWORD) -> BassResult<()> {
	// 	Ok(())
	// }

	// pub fn device(&self) -> DWORD {
	// 	self.device
	// }
	//
	// pub fn set_device(&mut self, device: impl Into<DWORD>) -> BassResult<()> {
	// 	let device = device.into();
	// 	let ok = BASS_ChannelSetDevice(*self.handle, device);
	// 	if ok {
	// 		self.device = device;
	// 		Ok(())
	// 	} else {
	// 		Err(BassError::default())
	// 	}
	// }

	pub fn stream_type(&self) -> SampleType {
		self.stream_type
	}

	// /// Consumes [`self`] and returns the sample data if applicable.
	// pub fn free(self) -> Option<Vec<u8>> {
	// 	self.data.map(|data| match data {
	// 		SampleData::Memory(data) => data
	// 	})
	// }

	// pub fn duration(&self) -> Result<QWORD, BassError> {
	// 	let bytes = BASS_ChannelGetLength(*self.handle, BASS_POS_BYTE);
	// 	if *bytes as i64 == -1 {
	// 		return Err(BassError::default());
	// 	}
	// 	Ok(bytes)
	// }
	//
	// pub fn duration_seconds(&self) -> Result<f64, BassError> {
	// 	if let Ok(bytes) = self.duration() {
	// 		let length_seconds = BASS_ChannelBytes2Seconds(*self.handle, bytes);
	// 		Ok(length_seconds)
	// 	} else {
	// 		Err(BassError::default())
	// 	}
	// }
	//
	// pub fn position(&self) -> Result<f64, BassError> {
	// 	let bytes = BASS_ChannelGetPosition(*self.handle, BASS_POS_BYTE);
	// 	if *bytes as i64 == -1 {
	// 		return Err(BassError::default());
	// 	}
	// 	let seconds = BASS_ChannelBytes2Seconds(*self.handle, bytes);
	// 	if seconds < 0. {
	// 		return Err(BassError::default());
	// 	}
	// 	Ok(seconds)
	// }
	//
	// pub fn set_position(&self, seconds: c_double) -> BassResult<()> {
	// 	let bytes = BASS_ChannelSeconds2Bytes(*self.handle, seconds);
	// 	if *bytes as i64 == -1 {
	// 		return Err(BassError::default());
	// 	}
	// 	self.set_position_bytes(bytes)
	// }
	//
	// pub fn set_position_bytes(&self, bytes: impl Into<QWORD>) -> BassResult<()> {
	// 	let ok = BASS_ChannelSetPosition(*self.handle, bytes, BASS_POS_BYTE | BASS_POS_SCAN);
	// 	result(ok)
	// }
	//
	// pub fn set_mixer_position(&self, seconds: c_double) -> BassResult<()> {
	// 	let bytes = BASS_ChannelSeconds2Bytes(*self.handle, seconds);
	// 	if *bytes as i64 == -1 {
	// 		return Err(BassError::default());
	// 	}
	// 	self.set_mixer_position_bytes(bytes)
	// }
	//
	// pub fn set_mixer_position_bytes(&self, bytes: impl Into<QWORD>) -> BassResult<()> {
	// 	let ok = BASS_Mixer_ChannelSetPosition(*self.handle, bytes, BASS_POS_BYTE | BASS_POS_SCAN);
	// 	result(ok)
	// }
}

// Handled automatically by `impl Drop for Channel<T>`

// impl Drop for Sample {
// 	fn drop(&mut self) {
// 		println!("Freeing Stream: {}", **self.handle);
// 		BASS_ChannelFree(*self.handle);
// 		match self.data.as_mut() {
// 			Some(data) => match data {
// 				// Doesn't matter, it's a Vec, it drops itself!... right?
// 				StreamData::Memory(_) => (),
// 			},
// 			// Must have been created as a Network or File stream, nothing else to manually clean up.
// 			None => (),
// 		}
// 	}
// }

impl Deref for Sample {
	type Target = Channel<HSAMPLE>;

	fn deref(&self) -> &Self::Target {
		&self.handle
	}
}