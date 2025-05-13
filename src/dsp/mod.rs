use std::{fmt::Debug, sync::{Arc, Mutex}};

use bass_sys::{BASS_ChannelRemoveDSP, DWORD, HDSP};

#[derive(Debug)]
pub struct BassDsp<T: Send + Sync> {
	pub(crate) dsp: HDSP,
	pub(crate) channel: DWORD,
	/// Must be held for DSPs to function.
	#[allow(unused)]
	pub(crate) user: Arc<Mutex<DspUserData<T>>>,
}

impl<T: Send + Sync> Drop for BassDsp<T> {
	fn drop(&mut self) {
		#[cfg(debug_assertions)]
		println!("Freeing DSP {:?}", self.dsp);
		BASS_ChannelRemoveDSP(self.channel, self.dsp);
		// let _ = unsafe { Box::from_raw(self.user) };
	}
}

pub(crate) type DspCallback<T> = dyn FnMut(&mut T, &mut [f32], HDSP, DWORD) + Send + Sync + 'static;

#[repr(C)]
pub(crate) struct DspUserData<T: Send + Sync>(pub Box<DspCallback<T>>, pub Box<T>);

impl<T: Send + Sync> Debug for DspUserData<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("DspUserData").field(&"SyncCallback").field(&"Box<{unknown}>").finish()
	}
}

// TODO :: Figure out
// pub fn dsp_data_as_u8<'a>(data: &'a mut [f32]) -> &'a mut [u8] {
// 	unsafe { std::slice::from_raw_parts_mut(data.as_ptr() as *mut u8, data.len() * std::mem::size_of::<f32>()) }
// }

// pub fn dsp_data_as_i16<'a>(data: &'a mut [f32]) -> &'a mut [i16] {
// 	unsafe { std::slice::from_raw_parts_mut(data.as_ptr() as *mut i16, data.len() * std::mem::size_of::<f32>()) }
// }
