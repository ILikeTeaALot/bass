use std::ptr::{null, null_mut};

use bass_sys::{BASS_Split_StreamCreate, BASS_Split_StreamGetAvailable, BASS_Split_StreamGetSource, BASS_Split_StreamGetSplits, BASS_Split_StreamReset, BASS_Split_StreamResetEx, BASS_StreamFree, DWORD, HSTREAM};

use crate::{
	bass::error::BassError,
	channel::{handle::HasHandle, mixer::MixableChannel, Channel, MixerSource},
	stream::Stream,
	BassResult,
};

#[derive(Debug)]
pub struct Splitter(HSTREAM);

impl Splitter {
	pub fn create(stream: &Stream, flags: DWORD) -> BassResult<Self> {
		let stream = unsafe { BASS_Split_StreamCreate(stream.handle(), flags, null()) };
		if stream != 0 {
			Ok(Splitter(stream))
		} else {
			Err(BassError::get())
		}
	}

	pub fn available(&self, handle: Option<HSTREAM>) -> BassResult<u32> {
		let value = BASS_Split_StreamGetAvailable(handle.unwrap_or(self.0));
		if value.0 as i32 != -1 {
			Ok(value.0)
		} else {
			Err(BassError::get())
		}
	}

	pub fn source(&self) -> BassResult<DWORD> {
		let source = BASS_Split_StreamGetSource(self.0);
		if source != 0 {
			Ok(source)
		} else {
			Err(BassError::get())
		}
	}

	pub fn split_count(&self) -> DWORD {
		unsafe { BASS_Split_StreamGetSplits(self.0, null_mut::<HSTREAM>(), 0) }
	}

	/// If the returned `Vec` has `.len()` `0`, then there are no channels.
	pub fn splits(&self) -> BassResult<Vec<HSTREAM>> {
		// let count =
		//     BASS_Mixer_StreamGetChannels(self.handle, Vec::with_capacity(0).as_mut_ptr(), 0);
		// if count as i32 == -1 {
		//     return BassError::result();
		// }
		let count = self.split_count();
		let mut channels: Vec<HSTREAM> = Vec::with_capacity(*count as usize);
		let inserted = unsafe { BASS_Split_StreamGetSplits(self.0, channels.as_mut_ptr(), count) };
		if *inserted as i32 == -1 {
			return Err(BassError::get());
		}
		unsafe {
			channels.set_len(*inserted as usize);
		}
		Ok(channels)
	}

	pub fn reset(&self, offset: Option<u32>) -> BassResult<()> {
		let ok = if let Some(offset) = offset {
			BASS_Split_StreamResetEx(self.0, offset)
		} else {
			BASS_Split_StreamReset(self.0)
		};
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}
}

impl HasHandle for Splitter {
	fn handle(&self) -> DWORD {
		self.0 .0
	}
}

impl Channel for Splitter {}

impl MixableChannel for Splitter {}
impl MixerSource for Splitter {}

impl Drop for Splitter {
	fn drop(&mut self) {
		#[cfg(debug_assertions)]
		println!("Freeing Mixer {:?}", self.0);
		BASS_StreamFree(self.0); // Only reason it can fail is if the stream has already been freed
	}
}
