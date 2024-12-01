use bass_sys::{BASS_ChannelRemoveDSP, DWORD, HDSP};

#[derive(Debug)]
pub struct BassDsp<T: Send + Sync> {
	pub(crate) dsp: HDSP,
	pub(crate) channel: DWORD,
	pub(crate) user: *mut DspUserData<T>,
}

impl<T: Send + Sync> Drop for BassDsp<T> {
	fn drop(&mut self) {
		#[cfg(debug_assertions)]
		println!("Freeing DSP {:?}", self.dsp);
		BASS_ChannelRemoveDSP(self.channel, self.dsp);
		let _ = unsafe { Box::from_raw(self.user) };
	}
}

pub(crate) type DspCallback<T> = dyn FnMut(&mut T, &mut [f32], HDSP, DWORD);

#[repr(C)]
pub(crate) struct DspUserData<T: Send + Sync>(pub Box<DspCallback<T>>, pub Box<T>);

pub fn dsp_data_as_u8<'a>(data: &'a mut [f32]) -> &'a mut [u8] {
	unsafe { std::slice::from_raw_parts_mut(data.as_ptr() as *mut u8, data.len() * std::mem::size_of::<f32>()) }
}

pub fn dsp_data_as_i16<'a>(data: &'a mut [f32]) -> &'a mut [i16] {
	unsafe { std::slice::from_raw_parts_mut(data.as_ptr() as *mut i16, data.len() * std::mem::size_of::<f32>()) }
}
