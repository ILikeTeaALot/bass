use bass_sys::{BASS_ChannelBytes2Seconds, DWORD, HSTREAM};

use crate::{
	bass::error::{BassError, BassErrorCode},
	channel::{handle::HasHandle, Channel},
};

pub struct MaybeStream(HSTREAM);

impl TryFrom<DWORD> for MaybeStream {
	type Error = BassErrorCode;

	fn try_from(value: DWORD) -> Result<Self, Self::Error> {
		let ok = BASS_ChannelBytes2Seconds(value, 64);
		if ok.is_sign_negative() {
			Err(BassError::get())
		} else {
			Ok(MaybeStream(HSTREAM(value)))
		}
	}
}

impl HasHandle for MaybeStream {
	fn handle(&self) -> DWORD {
		self.0 .0
	}
}

impl Channel for MaybeStream {}

#[cfg(feature = "mixer")]
impl crate::channel::mixer::MixableChannel for MaybeStream {}
#[cfg(feature = "mixer")]
impl crate::channel::MixerSource for MaybeStream {}
