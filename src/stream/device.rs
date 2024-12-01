use std::{ffi::c_void, ptr::null_mut};

use bass_sys::{BASS_StreamCreate, HSTREAM, STREAMPROC_DEVICE};

use crate::channel::{handle::HasHandle, Channel};

#[derive(Debug)]
pub struct DeviceStream(HSTREAM);

impl DeviceStream {
	pub fn get() -> Self {
		DeviceStream(BASS_StreamCreate(0, 0, 0, *STREAMPROC_DEVICE, null_mut() as *mut c_void))
	}
}

impl HasHandle for DeviceStream {
	fn handle(&self) -> bass_sys::DWORD {
		self.0 .0
	}
}

impl Channel for DeviceStream {}
