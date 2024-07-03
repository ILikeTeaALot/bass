use std::{ops::Deref, os::raw::c_void, sync::Mutex};

use bass_sys::{BASS_ChannelSetFX, BASS_ChannelSetSync, BASS_FXSetParameters, BASS_StreamCreate, BASS_DX8_PARAMEQ, BASS_FX_DX8_PARAMEQ, BASS_SYNC_FREE, DWORD, HFX, HSYNC, STREAMPROC_DEVICE, BASS_ChannelRemoveSync};
use util::safe_lock::SafeLock;

use crate::{error::BassError, null::NULL, result::BassResult};

static GLOBAL_EQ: GlobalEqSettings = GlobalEqSettings::new();

pub struct GlobalEqSettings(EqSettings, Mutex<Option<HSYNC>>);

impl GlobalEqSettings {
	pub(crate) const fn new() -> Self {
		Self(EqSettings::new_global(), Mutex::new(None))
	}

	pub fn set_eq_bands(&self, bands: impl IntoIterator<Item = EqParam>) -> &'static Self {
		*GLOBAL_EQ.bands.safe_lock() = bands.into_iter().map(|b| (None, b.into())).collect();
		&GLOBAL_EQ
	}

	// pub fn set_eq_settings(settings: EqSettings) -> &'static Self {
	// 	*GLOBAL_EQ.0 = settings;
	// 	&GLOBAL_EQ
	// }

	pub fn apply(&self) -> BassResult<&'static Self> {
		let output_handle = BASS_StreamCreate(0, 0, 0, *STREAMPROC_DEVICE, NULL);
		// -- //
		let mut sync_handle = self.1.safe_lock();
		match *sync_handle {
			None => (),
			Some(sync) => { BASS_ChannelRemoveSync(output_handle, sync); }
		}
		let sync = BASS_ChannelSetSync(output_handle, BASS_SYNC_FREE, 0, Some(update_global_eq), NULL);
		if sync != 0 {
			*sync_handle = Some(sync)
		}
		drop(sync_handle);
		// -- //
		// let mut fx_handles_guard = self.fx_handles.safe_lock();
		// match &mut *fx_handles_guard {
		// 	Some(handle_list) => {
		// 	}
		// 	None => {
		// 	}
		// }
		// *fx_handles_guard = Some(Vec::with_capacity(self.bands.safe_lock().len()));
		// let fx_handles = fx_handles_guard.as_mut().unwrap();
		let bands = &mut self.bands.safe_lock();
		for (fx, band) in bands.iter_mut() {
			let handle = match fx {
				Some(handle) => *handle,
				None => {
					let fx_handle = BASS_ChannelSetFX(output_handle, BASS_FX_DX8_PARAMEQ, 0);
					if fx_handle == 0 {
						return BassError::result();
					} else {
						*fx = Some(fx_handle);
						fx_handle
					}
				}
			};
			let ok = unsafe { BASS_FXSetParameters(handle, band as *mut _ as *const c_void) };
			if !ok {
				return BassError::result();
			}
		}
		Ok(&GLOBAL_EQ)
	}
}

impl Deref for GlobalEqSettings {
	type Target = EqSettings;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// # Notes
///
/// On Windows, `frequency` must be in the range of 80 to 16000,
/// and not exceed one-third of the channel's sample rate.
///
/// On other platforms, the range is above 0 and below half the channel's sample rate.
#[derive(Clone, Debug)]
pub struct EqParam {
	/// Centre frequency in hertz
	frequency: f32,
	/// Band Width in semitones (default 12)
	bandwidth: f32,
	/// Gain from -15 to 15
	gain: f32,
}

impl EqParam {
	pub fn new(frequency: f32, bandwidth: Option<f32>, gain: Option<f32>) -> Self {
		Self { frequency, bandwidth: bandwidth.unwrap_or(12.), gain: gain.unwrap_or(0.) }
	}
}

impl Into<BASS_DX8_PARAMEQ> for EqParam {
	fn into(self) -> BASS_DX8_PARAMEQ {
		BASS_DX8_PARAMEQ { fCenter: self.frequency, fBandwidth: self.bandwidth, fGain: self.gain }
	}
}

impl Into<EqParam> for BASS_DX8_PARAMEQ {
	fn into(self) -> EqParam {
		EqParam { frequency: self.fCenter, gain: self.fGain, bandwidth: self.fBandwidth }
	}
}

pub struct EqSettings {
	bands: Mutex<Vec<(Option<HFX>, BASS_DX8_PARAMEQ)>>,
	// fx_handles: Mutex<Option<Vec<HFX>>>,
}

impl EqSettings {
	pub fn new(bands: impl IntoIterator<Item = EqParam>) -> Self {
		Self { bands: Mutex::new(bands.into_iter().map(|b| (None, b.into())).collect()) }
	}

	pub(crate) const fn new_global() -> Self {
		Self { bands: Mutex::new(vec![]) }
	}

	// fn unwrap_apply_eq_to_channel(handle: HSTREAM, band: &EqParam) -> BassResult<()> {
	// 	let mut fx_handles_guard = self.fx_handles.safe_lock();
	// 	let fx_handles = fx_handles_guard.as_mut().unwrap();
	// 	for (index, mut band) in self.bands.safe_lock().iter_mut().enumerate() {
	// 		let ok = BASS_FXSetParameters(fx_handles[index], &mut band as *mut _ as *const c_void);
	// 		if !ok {
	// 			return BassError::result();
	// 		}
	// 	}
	// 	Ok(())
	// }

	/* pub fn apply_to_output(&self) -> BassResult<()> {
		let output_handle = BASS_StreamCreate(0, 0, 0, *STREAMPROC_DEVICE, NULL);
		// let mut fx_handles_guard = self.fx_handles.safe_lock();
		// match &mut *fx_handles_guard {
		// 	Some(handle_list) => {
		// 	}
		// 	None => {
		// 	}
		// }
		// *fx_handles_guard = Some(Vec::with_capacity(self.bands.safe_lock().len()));
		// let fx_handles = fx_handles_guard.as_mut().unwrap();
		let bands = &mut self.bands.safe_lock();
		for (handle, band) in bands.iter_mut() {
			let fx_handle = match handle {
				Some(handle) => *handle,
				None => {
					let fx_handle = BASS_ChannelSetFX(output_handle, BASS_FX_DX8_PARAMEQ, 0);
					if *fx_handle == 0 {
						return BassError::result();
					} else {
						*handle = Some(fx_handle);
						fx_handle
					}
				}
			};
			let ok = BASS_FXSetParameters(fx_handle, band as *mut _ as *const c_void);
			if !ok {
				return BassError::result();
			}
		}
		Ok(())
	} */

	pub fn apply_to_output(&self) -> BassResult<()> {
		GLOBAL_EQ.set_eq_bands(self.bands.safe_lock().iter().map(|(_, eq)| eq.clone().into())).apply()?;
		Ok(())
	}
}

// impl Drop for EqSettings {
// 	fn drop(&mut self) {
// 		for (fx, _) in self.bands.safe_lock().iter() {
// 			if let Some(fx) = fx {
// 				let _ = BASS_ChannelRemoveFX(0, *fx);
// 			}
// 		}
// 	}
// }

extern "C" fn update_global_eq(_: HSYNC, _: DWORD, _: DWORD, _: *mut c_void) {
	let output_handle = BASS_StreamCreate(0, 0, 0, *STREAMPROC_DEVICE, NULL);
	BASS_ChannelSetSync(output_handle, BASS_SYNC_FREE, 0, Some(update_global_eq), NULL);
	GLOBAL_EQ.apply();
}