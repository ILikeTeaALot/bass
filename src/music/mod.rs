use std::{ffi::c_void, sync::Mutex};

use bass_sys::{BASS_ChannelGetDevice, BASS_ChannelSetDevice, BASS_MusicLoad, DWORD, HMUSIC, QWORD};
use widestring::U16CString;

use crate::{error::BassError, result::BassResult};

pub struct ModMusic {
	data: Option<Vec<u8>>,
	handle: HMUSIC,
	device: DWORD,
	flags: Mutex<DWORD>,
}

impl ModMusic {
	fn new_internal(
		handle: HMUSIC,
		flags: DWORD,
		device: Option<impl Into<DWORD>>,
		data: Option<Vec<u8>>,
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
		Ok(Self { device, flags: Mutex::new(flags), handle, data })
	}

	/// By default, the BASS library handles opening and managing the file.
	pub fn new_file(
		path: impl AsRef<str>,
		flags: impl Into<DWORD>,
		frequency: u32,
		offset: impl Into<QWORD>,
		length: impl Into<DWORD>,
		device: Option<impl Into<DWORD>>,
	) -> BassResult<Self> {
		let flags = flags.into();
		let path = U16CString::from_str(path).unwrap();
		// let seconds = Seconds(1.);
		// let bytes = Bytes(800402);
		let handle =
		// 	unsafe { BASS_StreamCreateFile(false, path.as_ptr() as *const c_void, 0, 0, flags | BASS_UNICODE) };
		unsafe { BASS_MusicLoad(false, path.as_ptr() as *const c_void, offset, length, flags, DWORD(frequency)) };
		if handle == 0 {
			let error = BassError::default();
			eprintln!("{}", error);
			eprintln!("Path: {}", path.to_string_lossy());
			eprintln!("Flags: {}", flags);
			eprintln!("Stream type: {:?}", "File");
			eprintln!("Stream failed to be created!");
			return Err(error);
		}
		Self::new_internal(handle, flags, device, None)
	}

	/// By default, the BASS library handles opening and managing the file.
	pub fn new_mem(
		memory: Vec<u8>,
		flags: impl Into<DWORD>,
		frequency: u32,
		offset: impl Into<QWORD>,
		length: impl Into<DWORD>,
		device: Option<impl Into<DWORD>>,
	) -> BassResult<Self> {
		let flags = flags.into();
		// let seconds = Seconds(1.);
		// let bytes = Bytes(800402);
		let handle =
		// 	unsafe { BASS_StreamCreateFile(false, path.as_ptr() as *const c_void, 0, 0, flags | BASS_UNICODE) };
		unsafe { BASS_MusicLoad(true, memory.as_ptr() as *const c_void, offset, length, flags, DWORD(frequency)) };
		if handle == 0 {
			let error = BassError::default();
			eprintln!("{}", error);
			eprintln!("Flags: {}", flags);
			eprintln!("Stream type: {:?}", "Memory");
			eprintln!("Stream failed to be created!");
			return Err(error);
		}
		Self::new_internal(handle, flags, device, None)
	}
}
