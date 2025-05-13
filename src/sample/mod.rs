use std::{os::raw::c_void, ptr::null_mut};

use bass_sys::{BASS_SampleFree, BASS_SampleGetChannel, BASS_SampleGetChannels, BASS_SampleGetInfo, BASS_SampleLoad, BASS_SampleSetInfo, BASS_SampleStop, BASS_SAMCHAN_STREAM, BASS_SAMPLE, BASS_UNICODE, DWORD, HCHANNEL, HSAMPLE, QWORD};
use widestring::U16CString;

use crate::{
	bass::error::BassError, channel::{handle::HasHandle, Channel}, stream::Stream, BassResult
};

#[derive(Debug)]
pub struct Sample(HSAMPLE);

/// Samples will automatically free themselves and all streams and channels created from them when dropped.
impl Sample {
	pub fn load(
		path: impl AsRef<str>,
		offset: impl Into<QWORD>,
		length: impl Into<DWORD>,
		maximum: impl Into<DWORD>,
		flags: DWORD,
	) -> BassResult<Self> {
		let file: Vec<u16> = path.as_ref().encode_utf16().collect();
		let file = U16CString::from_vec_truncate(file);
		let handle = unsafe {
			BASS_SampleLoad(false, file.as_ptr() as *const c_void, offset, length, maximum, flags | BASS_UNICODE)
		};
		if handle != 0 {
			Ok(Self(handle))
		} else {
			Err(BassError::get())
		}
	}

	pub fn load_memory(data: &[u8], flags: DWORD, maximum: impl Into<DWORD>) -> BassResult<Self> {
		let handle =
			unsafe { BASS_SampleLoad(true, data.as_ptr() as *const c_void, 0, data.len(), maximum, flags | BASS_UNICODE) };
		if handle != 0 {
			Ok(Self(handle))
		} else {
			Err(BassError::get())
		}
	}

	pub fn get_channel(&self, flags: DWORD) -> BassResult<SampleChannel> {
		let ok = BASS_SampleGetChannel(self.0, flags & !BASS_SAMCHAN_STREAM);
		if let Some(handle) = ok {
			Ok(SampleChannel(HCHANNEL(handle)))
		} else {
			Err(BassError::get())
		}
	}

	pub fn get_stream(&self, flags: DWORD) -> BassResult<Stream> {
		Stream::from_sample(self.0, flags)
	}

	pub fn channels_count(&self) -> DWORD {
		unsafe { BASS_SampleGetChannels(self.0, null_mut::<HCHANNEL>()) }
	}

	/// If the returned `Vec` has `.len()` `0`, then there are no channels.
	pub fn channels(&self) -> BassResult<Vec<HCHANNEL>> {
		// let count =
		//     BASS_Mixer_StreamGetChannels(self.handle, Vec::with_capacity(0).as_mut_ptr(), 0);
		// if count as i32 == -1 {
		//     return BassError::result();
		// }
		let count = self.channels_count();
		let mut channels: Vec<HCHANNEL> = Vec::with_capacity(*count as usize);
		let inserted = unsafe { BASS_SampleGetChannels(self.0, channels.as_mut_ptr()) };
		if *inserted as i32 == -1 {
			return Err(BassError::get());
		}
		unsafe {
			channels.set_len(*inserted as usize);
		}
		Ok(channels)
	}

	pub fn info(&self) -> BassResult<BASS_SAMPLE> {
		let mut info = BASS_SAMPLE::default();
		let ok = BASS_SampleGetInfo(self.0, &mut info);
		if ok {
			Ok(info)
		} else {
			Err(BassError::get())
		}
	}

	pub fn set_info(&self, info: BASS_SAMPLE) -> BassResult<()> {
		let ok = BASS_SampleSetInfo(self.0, &info);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	pub fn stop(&self) -> BassResult<()> {
		let ok = BASS_SampleStop(self.0);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}
}

impl HasHandle for Sample {
	fn handle(&self) -> DWORD {
		self.0 .0
	}
}

impl Drop for Sample {
    fn drop(&mut self) {
        BASS_SampleFree(self.0);
    }
}

pub struct SampleChannel(HCHANNEL);

impl HasHandle for SampleChannel {
    fn handle(&self) -> DWORD {
        self.0.0
    }
}

impl Channel for SampleChannel {}