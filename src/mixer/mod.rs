use std::ptr::null_mut;

use bass_sys::*;

use crate::{bass::error::{BassError, BassErrorCode}, channel::{handle::HasHandle, mixer::MixableChannel, Channel, MixerSource}, BassResult};

#[derive(Debug)]
pub struct Mixer(HSTREAM);

impl Mixer {
	pub fn create(frequency: impl Into<DWORD>, channels: impl Into<DWORD>, flags: Option<DWORD>) -> BassResult<Self> {
		let handle = BASS_Mixer_StreamCreate(frequency, channels, flags.unwrap_or_default());
		if handle != 0 {
			Ok(Self(handle))
		} else {
			Err(BassError::get())
		}
	}

	pub fn add_channel(&self, channel: &impl Channel, flags: Option<DWORD>) -> BassResult<()> {
		let ok = BASS_Mixer_StreamAddChannel(self.0, channel.handle(), flags.unwrap_or_default());
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	pub fn add_channel_ex(&self, channel: &impl Channel, flags: impl Into<DWORD>, delay: impl Into<QWORD>, length: impl Into<QWORD>) -> BassResult<()> {
		let ok = BASS_Mixer_StreamAddChannelEx(self.0, channel.handle(), flags, delay, length);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	pub fn current(&self) -> BassResult<Option<DWORD>> {
		let mut channels: Vec<DWORD> = Vec::with_capacity(1);
		let inserted = unsafe { BASS_Mixer_StreamGetChannels(self.0, channels.as_mut_ptr(), 1) };
		if *inserted as i32 == -1 {
			return Err(BassError::get()); // An error occurred; pass the error up the call chain.
		} else if *inserted == 0 {
			return Ok(None); // No channels currently in Mixer.
		}
		unsafe {
			channels.set_len(*inserted as usize); // Set len
		}
		match channels.get(0) {
			Some(channel) => Ok(Some(*channel)),
			None => Err(BassErrorCode::BassErrorUnknown), // This shouldn't be able to happen, but if it does, a horrible error occurred somewhere
		}
	}

	pub fn clear(&self) -> BassResult<()> {
		if let Ok(channels) = self.channels() {
			for handle in channels {
				let ok = BASS_ChannelSetPosition(handle, 0, BASS_POS_BYTE | BASS_POS_FLUSH);
				if !ok {
					BassError::consume();
				}
				let ok = BASS_Mixer_ChannelRemove(handle);
				if !ok {
					BassError::consume();
				}
			}
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	pub fn flush(&self) -> BassResult<()> {
		let ok = BASS_ChannelSetPosition(self.0, 0, BASS_POS_BYTE | BASS_POS_FLUSH);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	pub fn has_channel(&self, channel: impl Into<DWORD>) -> bool {
		let mixer = BASS_Mixer_ChannelGetMixer(channel);
		if mixer == 0 {
			BassError::consume()
		}
		return mixer == self.0;
	}

	pub fn channels_count(&self) -> u32 {
		unsafe { BASS_Mixer_StreamGetChannels(self.0, null_mut::<DWORD>(), 0).0 }
	}

	/// If the returned `Vec` has `.len()` `0`, then there are no channels.
	pub fn channels(&self) -> BassResult<Vec<DWORD>> {
		// let count =
		//     BASS_Mixer_StreamGetChannels(self.handle, Vec::with_capacity(0).as_mut_ptr(), 0);
		// if count as i32 == -1 {
		//     return BassError::result();
		// }
		let count = self.channels_count();
		let mut channels: Vec<DWORD> = Vec::with_capacity(count as usize);
		let inserted = unsafe { BASS_Mixer_StreamGetChannels(self.0, channels.as_mut_ptr(), count) };
		if *inserted as i32 == -1 {
			return Err(BassError::get());
		}
		unsafe {
			channels.set_len(*inserted as usize);
		}
		Ok(channels)
	}
}

impl HasHandle for Mixer {
	fn handle(&self) -> bass_sys::DWORD {
		self.0 .0
	}
}

impl Channel for Mixer {}

impl MixableChannel for Mixer {}
impl MixerSource for Mixer {}

impl Drop for Mixer {
	fn drop(&mut self) {
		#[cfg(debug_assertions)]
		println!("Freeing Mixer {:?}", self.0);
		BASS_StreamFree(self.0); // Only reason it can fail is if the stream has already been freed
	}
}
