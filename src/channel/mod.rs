use std::fmt::Debug;
use std::ops::Deref;

use bass_sys::{BASS_ChannelFree, BASS_ChannelGet3DAttributes, BASS_ChannelGet3DPosition, BASS_ChannelGetAttribute, BASS_ChannelSet3DPosition, BASS_3DVECTOR, DWORD, BASS_ChannelSetAttribute, BASS_ChannelGetInfo, BASS_ChannelFlags};

use crate::error::BassError;
use crate::result::{BassResult, result};
use crate::types::channel::{ChannelGetAttribute, ChannelSetAttribute, Bass3DAttributes, Bass3DPosition, BassChannelInfo};

#[derive(Copy, Clone, Debug)]
pub enum FlagUpdateError {
	BassError(BassError),
	FlagsNotChanged(DWORD),
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Channel<T: Clone + Copy + Debug + Deref<Target = DWORD> + Into<DWORD>>(pub(crate) T);

// /// In theory this should auto-drop all Channel<H_> types...
// /// 
// /// I really need to open source this to get some more experienced eyeballs on it...
// impl<T: Clone + Copy + Debug + Deref<Target = DWORD> + Into<DWORD>> Drop for Channel<T> {
// 	fn drop(&mut self) {
// 		BASS_ChannelFree(self.handle());
// 	}
// }

impl<T: Clone + Copy + Debug + Deref<Target = DWORD> + Into<DWORD>> Channel<T> {
	pub fn handle(&self) -> DWORD {
		(&self.0).clone().into()
	}

	pub fn get_3d_position(&self) -> BassResult<Bass3DPosition> {
		let mut position = Some(Default::default());
		let mut orientation = Some(Default::default());
		let mut velocity = Some(Default::default());
		match BASS_ChannelGet3DPosition(self.handle(), &mut position, &mut orientation, &mut velocity) {
			true => Ok(Bass3DPosition { position, orientation, velocity }),
			false => Err(BassError::default()),
		}
	}

	pub fn set_3d_position(
		&self,
		position: &Option<BASS_3DVECTOR>,
		orientation: &Option<BASS_3DVECTOR>,
		velocity: &Option<BASS_3DVECTOR>,
	) -> BassResult<()> {
		result(BASS_ChannelSet3DPosition(self.handle(), position, orientation, velocity))
	}

	pub fn get_3d_attributes(&self) -> BassResult<Bass3DAttributes> {
		let mut mode: Option<DWORD> = Some(DWORD(0));
		let mut minimum: Option<f32> = Some(0.);
		let mut maximum: Option<f32> = Some(0.);
		let mut i_angle: Option<DWORD> = Some(DWORD(0));
		let mut o_angle: Option<DWORD> = Some(DWORD(0));
		let mut output_volume: Option<f32> = Some(0.);
		match BASS_ChannelGet3DAttributes(
			self.handle(),
			&mut mode,
			&mut minimum,
			&mut maximum,
			&mut i_angle,
			&mut o_angle,
			&mut output_volume,
		) {
			true => Ok(Bass3DAttributes {
				mode: mode.unwrap(),
				minimum: minimum.unwrap(),
				maximum: maximum.unwrap(),
				angle_of_inside_projection_cone: i_angle.unwrap(),
				angle_of_outside_projection_cone: o_angle.unwrap(),
				output_volume: output_volume.unwrap(),
			}),
			false => Err(BassError::default()),
		}
	}

	pub fn free(self: Self) -> BassResult<()> {
		result(BASS_ChannelFree(self.handle()))
	}

	pub fn get_attribute(&self, attrib: ChannelGetAttribute) -> BassResult<f32> {
		let mut value = 0.;
		match BASS_ChannelGetAttribute(self.handle(), attrib as u32, &mut value) {
			true => Ok(value),
			false => Err(BassError::default()),
		}
	}

	pub fn set_attribute(&self, attrib: ChannelSetAttribute, value: f32) -> BassResult<()> {
		// match match attrib {
		// 	ChannelSetAttribute::Buffer(value)
		// 	| ChannelSetAttribute::Frequency(value)
		// 	| ChannelSetAttribute::Granule(value)
		// 	| ChannelSetAttribute::MusicActive(value)
		// 	| ChannelSetAttribute::MusicAmplify(value)
		// 	| ChannelSetAttribute::MusicBPM(value)
		// 	| ChannelSetAttribute::MusicPanSeparation(value)
		// 	| ChannelSetAttribute::MusicPositionScaler(value)
		// 	| ChannelSetAttribute::MusicSpeed(value)
		// 	| ChannelSetAttribute::MusicChannelVolume(value)
		// 	| ChannelSetAttribute::MusicGlobalVolume(value)
		// 	| ChannelSetAttribute::MusicInstrumentVolume(value)
		// 	| ChannelSetAttribute::NetworkResume(value)
		// 	| ChannelSetAttribute::Ramp(value)
		// 	| ChannelSetAttribute::Pan(value)
		// 	| ChannelSetAttribute::BufferPushLimit(value)
		// 	| ChannelSetAttribute::SRCQuality(value)
		// 	| ChannelSetAttribute::Tail(value)
		// 	| ChannelSetAttribute::Volume(value)
		// 	| ChannelSetAttribute::VolumeDSP(value)
		// 	| ChannelSetAttribute::VolumeDSPPriority(value) => BASS_ChannelSetAttribute(self.handle(), DWORD(attrib as u32), value)
		// } {
		// 	true => Ok(()),
		// 	false => Err(BassError::default()),
		// }
		match BASS_ChannelSetAttribute(self.handle(), attrib as u32, value) {
			true => Ok(()),
			false => Err(BassError::default())
		}
	}

	pub fn get_info(&self) -> BassResult<BassChannelInfo> {
		let mut info = Default::default();
		match BASS_ChannelGetInfo(self.handle(), &mut info) {
			true => Ok(info.into()),
			false => Err(BassError::default())
		}
	}

	/// Only time this should fail is if the handle is invalid which,
	/// if `BASS_AUTOFREE` was set on the Channel, it could be.
	pub fn get_flags(&self) -> BassResult<DWORD> {
		let flags = BASS_ChannelFlags(self.handle(), 0, 0);
		match flags == -1 {
			true => Err(BassError::default()),
			false => Ok(flags),
		}
	}

	/// Most of the time you can just ignore the error on this with `.ok()`, but sometimes you may
	/// want to check if the flags have been correctly updated.
	pub fn set_flags(&self, flags: impl Into<DWORD>, value: bool) -> Result<(), FlagUpdateError> {
		let flags = flags.into();
		let current = BASS_ChannelFlags(self.handle(), 0, 0);
		if current == -1 {
			Err(FlagUpdateError::BassError(BassError::default()))
		} else {
			if (current & flags) != 0 { // Flag is active on channel
				if value { // Flag is already enabled, return Ok()
					Ok(())
				} else {
					let ok = BASS_ChannelFlags(self.handle(), 0, flags);
					if ok == -1 { // Handle invalid, caller should handle dropping
						Err(FlagUpdateError::BassError(BassError::default()))
					} else { // Everything OK
						if ok & flags == flags { // Successfully updated
							Ok(())
						} else { // Unsuccessful
							Err(FlagUpdateError::FlagsNotChanged(ok & flags))
						}
					}
				}
			} else { // Flag is inactive on channel
				if !value { // Flag is already disabled, return Ok()
					return Ok(());
				}
				let ok = BASS_ChannelFlags(self.handle(), flags, flags);
				if ok == -1 { // Handle invalid. caller should handle dropping
					Err(FlagUpdateError::BassError(BassError::default()))
				} else { // Everything OK
					if ok & flags == flags { // Successfully updated
						Ok(())
					} else { // Unsuccessful
						Err(FlagUpdateError::FlagsNotChanged(ok & flags))
					}
				}
			}
		}
	}
}

// impl Channel for HSTREAM {}
// impl Channel for HMUSIC {}

impl<T: Clone + Copy + Debug + Deref<Target = DWORD> + Into<DWORD>> Deref for Channel<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T: Clone + Copy + Debug + Deref<Target = DWORD> + Into<DWORD>> From<Channel<T>> for DWORD {
	fn from(value: Channel<T>) -> Self {
		value.0.into()
	}
}