use std::{
	ops::DerefMut,
	os::raw::c_void,
	ptr::null_mut,
	slice,
	sync::{Arc, Mutex, MutexGuard, Weak},
};

use bass_sys::*;
use handle::HasHandle;

use crate::{
	bass::error::BassError,
	dsp::{BassDsp, DspUserData},
	functions::make_word,
	fx::BassFx,
	sync::{BassSync, SyncUserData},
	BassResult,
};

/// In many respects this is an absolute nightmare...
extern "C" fn sync_handler<T: Send + Sync>(handle: HSYNC, channel: DWORD, data: DWORD, user: *mut c_void) {
	// let mut user_box = unsafe { Arc::from_raw(user as *mut SyncUserData<T>) };
	let f = |mut user_box: MutexGuard<'_, SyncUserData<T>>| {
		#[cfg(debug_assertions)]
		println!("deref_mut'ing SyncUserData...");
		let user_box = user_box.deref_mut();
		#[cfg(debug_assertions)]
		println!("Running user SyncProc...");
		(user_box.0)(user_box.1.as_mut(), handle, channel, data);
		#[cfg(debug_assertions)]
		println!("Success!");
	};
	// Attempt to upgrade the weak pointer, failing gracefully if it has already been dropped.
	match unsafe { Weak::from_raw(user as *const Mutex<SyncUserData<T>>) }.upgrade() {
		Some(arc) => {
			#[cfg(debug_assertions)]
			println!("Strong: {}; Weak: {}", Arc::strong_count(&arc), Arc::weak_count(&arc));
			// Handle mutex locking.
			#[cfg(debug_assertions)]
			println!("Locking Mutex...");
			match arc.lock() {
				Ok(user_box) => f(user_box),
				Err(e) => f(e.into_inner()),
			}
			#[cfg(debug_assertions)]
			println!("Resetting weak count...");
			// Reset the weak count.
			let weak = Arc::downgrade(&arc);
			// Equivalent to std::mem::forget in a way...
			let _ = weak.into_raw();
			#[cfg(debug_assertions)]
			println!("Hopefully success?");
		}
		None => {
			#[cfg(debug_assertions)]
			println!("User data freed")
		}
	}
	#[cfg(debug_assertions)]
	println!("Everything should drop now.");
}

extern "C" fn dsp_handler<T: Send + Sync>(
	handle: HDSP,
	channel: DWORD,
	buffer: *mut c_void,
	length: DWORD,
	user: *mut c_void,
) {
	// let mut user_box = unsafe { Box::from_raw(user as *mut DspUserData<T>) };
	// let mut data = unsafe { slice::from_raw_parts_mut(buffer as *mut f32, (length.0 / 4) as usize) };
	// (user_box.0)(user_box.1.as_mut(), &mut data, handle, channel); // FIXME!
	// Box::into_raw(user_box);
	let f = |mut user_box: MutexGuard<'_, DspUserData<T>>| {
		#[cfg(debug_assertions)]
		println!("deref_mut'ing DspUserData...");
		let user_box = user_box.deref_mut();
		#[cfg(debug_assertions)]
		println!("Running user DspProc...");
		let mut data = unsafe { slice::from_raw_parts_mut(buffer as *mut f32, (length.0 / 4) as usize) };
		(user_box.0)(user_box.1.as_mut(), &mut data, handle, channel);
		#[cfg(debug_assertions)]
		println!("Success!");
	};
	// Attempt to upgrade the weak pointer, failing gracefully if it has already been dropped.
	match unsafe { Weak::from_raw(user as *const Mutex<DspUserData<T>>) }.upgrade() {
		Some(arc) => {
			#[cfg(debug_assertions)]
			println!("Strong: {}; Weak: {}", Arc::strong_count(&arc), Arc::weak_count(&arc));
			// Handle mutex locking.
			#[cfg(debug_assertions)]
			println!("Locking Mutex...");
			match arc.lock() {
				Ok(user_box) => f(user_box),
				Err(e) => f(e.into_inner()),
			}
			#[cfg(debug_assertions)]
			println!("Resetting weak count...");
			// Reset the weak count.
			let weak = Arc::downgrade(&arc);
			// Equivalent to std::mem::forget in a way...
			let _ = weak.into_raw();
			#[cfg(debug_assertions)]
			println!("Hopefully success?");
		}
		None => {
			#[cfg(debug_assertions)]
			println!("User data freed")
		}
	}
	#[cfg(debug_assertions)]
	println!("Everything should drop now.");
}

pub(crate) mod handle {
	use bass_sys::DWORD;

	pub trait HasHandle {
		fn handle(&self) -> DWORD;
	}
}

pub trait Channel: handle::HasHandle {
	#[inline]
	fn raw_handle(&self) -> DWORD {
		HasHandle::handle(self)
	}

	/// Equivalent to `self.handle() == channel`.
	#[inline]
	fn represents_handle(&self, channel: impl Into<DWORD>) -> bool {
		self.handle() == channel.into()
	}

	#[inline]
	fn bytes_to_seconds(&self, position: impl Into<QWORD>) -> BassResult<f64> {
		let value = BASS_ChannelBytes2Seconds(self.handle(), position);
		if value >= 0. {
			Ok(value)
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	fn flag_remove(&self, flag: DWORD) -> BassResult<DWORD> {
		let ok = BASS_ChannelFlags(self.handle(), 0, flag);
		if ok != -1 {
			Ok(ok)
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	fn flag_set(&self, flag: DWORD) -> BassResult<DWORD> {
		let ok = BASS_ChannelFlags(self.handle(), flag, flag);
		if ok != -1 {
			Ok(ok)
		} else {
			Err(BassError::get())
		}
	}

	// 3D Attributes TODO

	#[inline]
	fn get_attribute(&self, attribute: DWORD) -> BassResult<f32> {
		let mut value: f32 = 0.;
		let ok = BASS_ChannelGetAttribute(self.handle(), attribute, &mut value);
		if ok {
			Ok(value)
		} else {
			Err(BassError::get())
		}
	}

	// get_attribute_ex TODO MAYBE

	#[inline]
	fn get_device(&self) -> BassResult<u32> {
		let device = BASS_ChannelGetDevice(self.handle()).0;
		if device as i32 != -1 {
			Ok(device)
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	fn get_info(&self) -> BassResult<BASS_CHANNELINFO> {
		let mut info = BASS_CHANNELINFO::default();
		let ok = BASS_ChannelGetInfo(self.handle(), &mut info); // This should always succeed.
		if ok {
			Ok(info)
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	fn get_length(&self, mode: DWORD) -> BassResult<u64> {
		let ok = BASS_ChannelGetLength(self.handle(), mode);
		if ok.0 as i64 != -1 {
			Ok(ok.0)
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	fn get_level(&self) -> BassResult<(u16, u16)> {
		let ok = BASS_ChannelGetLevel(self.handle());
		if ok.0 as i32 != -1 {
			Ok(make_word(ok))
		} else {
			Err(BassError::get())
		}
	}

	fn get_level_ex(&self, length: f32, flags: Option<DWORD>) -> BassResult<Vec<f32>> {
		let mono = (flags.unwrap_or_default() & BASS_LEVEL_MONO) == BASS_LEVEL_MONO;
		let stereo = (flags.unwrap_or_default() & BASS_LEVEL_STEREO) == BASS_LEVEL_STEREO;
		if mono || stereo {
			let mut levels = Vec::with_capacity(2);
			#[cfg(debug_assertions)]
			println!("Getting levels in mono or stereo mode.");
			let ok = unsafe {
				BASS_ChannelGetLevelEx(self.handle(), levels.as_mut_ptr(), length, flags.unwrap_or_default())
			};
			if ok {
				unsafe {
					levels.set_len(if mono { 1 } else { 2 });
				}
				Ok(levels)
			} else {
				Err(BassError::get())
			}
		} else {
			let mut info = BASS_CHANNELINFO::default();
			let ok = BASS_ChannelGetInfo(self.handle(), &mut info); // This should always succeed.
			if ok {
				let chan_count = info.chans.0 as usize;
				let mut levels = Vec::with_capacity(chan_count);
				#[cfg(debug_assertions)]
				println!("Getting levels for channel with {chan_count} channels.");
				let ok = unsafe {
					BASS_ChannelGetLevelEx(self.handle(), levels.as_mut_ptr(), length, flags.unwrap_or_default())
				};
				if ok {
					unsafe {
						levels.set_len(chan_count);
					}
					Ok(levels)
				} else {
					Err(BassError::get())
				}
			} else {
				Err(BassError::get())
			}
		}
	}

	#[inline]
	fn get_position(&self, mode: DWORD) -> BassResult<u64> {
		let value = BASS_ChannelGetPosition(self.handle(), mode);
		if value.0 as i64 != -1 {
			Ok(value.0)
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	/// TODO :: Use an enum?
	fn is_active(&self) -> DWORD {
		BASS_ChannelIsActive(self.handle())
	}

	#[inline]
	fn is_sliding(&self, attrib: DWORD) -> bool {
		BASS_ChannelIsSliding(self.handle(), attrib)
	}

	#[inline]
	fn lock(&self) -> BassResult<()> {
		let ok = BASS_ChannelLock(self.handle(), true);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	fn unlock(&self) -> BassResult<()> {
		let ok = BASS_ChannelLock(self.handle(), false);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	fn pause(&self) -> BassResult<()> {
		let ok = BASS_ChannelPause(self.handle());
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	fn play(&self, restart: bool) -> BassResult<()> {
		let ok = BASS_ChannelPlay(self.handle(), restart);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	// fn remove_dsp(&self, dsp: &BassDsp) -> BassResult<()> {
	// 	let ok = BASS_ChannelRemoveDSP(self.handle(), dsp.0);
	// 	if ok {
	// 		Ok(())
	// 	} else {
	// 		Err(BassError::get())
	// 	}
	// }

	#[inline]
	fn remove_fx(&self, fx: &BassFx) -> BassResult<()> {
		let ok = BASS_ChannelRemoveFX(self.handle(), fx.0);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	fn remove_link(&self, channel: impl Into<DWORD>) -> BassResult<()> {
		let ok = BASS_ChannelRemoveLink(self.handle(), channel);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	// fn remove_sync(&self, sync: BassSync) -> BassResult<()> {
	// 	let ok = BASS_ChannelRemoveSync(self.handle(), sync.sync);
	// 	if ok {
	// 		Ok(())
	// 	} else {
	// 		Err(BassError::get())
	// 	}
	// }

	#[inline]
	fn seconds_to_bytes(&self, seconds: f64) -> BassResult<u64> {
		let value = BASS_ChannelSeconds2Bytes(self.handle(), seconds);
		if value.0 as i64 != -1 {
			Ok(value.0)
		} else {
			Err(BassError::get())
		}
	}

	// Set 3D Attributes...

	#[inline]
	fn set_attribute(&self, attribute: DWORD, value: f32) -> BassResult<()> {
		let ok = BASS_ChannelSetAttribute(self.handle(), attribute, value);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	fn set_device(&self, device: DWORD) -> BassResult<()> {
		let ok = BASS_ChannelSetDevice(self.handle(), device);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	/// To use user data, the recommended way is a static Arc/Mutex
	fn set_dsp<T: Send + Sync>(
		&self,
		priority: i32,
		user_data: T,
		proc: impl FnMut(&mut T, &mut [f32], HDSP, DWORD) + Send + Sync + 'static,
	) -> BassResult<BassDsp<T>> {
		let data = Box::new(user_data);
		let user = Arc::new(Mutex::new(DspUserData(Box::new(proc), data)));
		// let raw = Box::into_raw(user);
		let weak = Arc::downgrade(&user);
		let dsp = BASS_ChannelSetDSP(self.handle(), Some(dsp_handler::<T>), weak.into_raw() as *mut Mutex<DspUserData<T>>, priority);
		println!("HDSP: {:?}", dsp);
		if dsp != 0 {
			Ok(BassDsp { dsp, channel: self.handle(), user })
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	/// To use user data, the recommended way is a static Arc/Mutex
	fn set_link(&self, channel: DWORD) -> BassResult<()> {
		let ok = BASS_ChannelSetLink(self.handle(), channel);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	fn set_position(&self, position: impl Into<QWORD>, mode: DWORD) -> BassResult<()> {
		let ok = BASS_ChannelSetPosition(self.handle(), position, mode);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	/// The `BassSync` type holds the user data.
	/// You must hold the `BassSync` until you no longer wish to have the sync.
	/// The Sync will free itself automatically when dropped.
	fn set_sync<T: Send + Sync>(
		&self,
		sync_type: DWORD,
		parameter: impl Into<QWORD>,
		proc: impl FnMut(&mut T, HSYNC, DWORD, DWORD) + Send + Sync + 'static,
		user_data: T,
	) -> BassResult<BassSync<T>> {
		// let sync = BASS_ChannelSetSync(self.handle(), sync_type & (!BASS_SYNC_ONETIME), parameter, proc, null_mut() as *mut c_void);
		let data = Box::new(user_data);
		let user = Arc::new(Mutex::new(SyncUserData(Box::new(proc), data)));
		let weak = Arc::downgrade(&user);
		let sync = BASS_ChannelSetSync(
			self.handle(),
			sync_type,
			parameter,
			Some(sync_handler::<T>),
			weak.into_raw() as *mut Mutex<SyncUserData<T>>,
		);
		println!("Sync: {:?}", sync);
		if sync != 0 {
			Ok(BassSync { sync, channel: self.handle(), user })
		} else {
			Err(BassError::get())
		}
	}

	// /// This function will only allow you to set a onetime-sync
	// fn set_sync_once(&self, sync_type: DWORD, parameter: u64, proc: SYNCPROC) -> BassResult<BassSyncOnce> {
	// 	let sync = BASS_ChannelSetSync(self.handle(), sync_type | BASS_SYNC_ONETIME, parameter, proc, null_mut() as *mut c_void);
	// 	if sync != 0 {
	// 		Ok(BassSyncOnce { sync, channel: self.handle() })
	// 	} else {
	// 		Err(BassError::get())
	// 	}
	// }

	#[inline]
	fn slide_attribute(&self, attribute: DWORD, value: f32, milliseconds: u32) -> BassResult<()> {
		let ok = BASS_ChannelSlideAttribute(self.handle(), attribute, value, milliseconds);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	fn start(&self) -> BassResult<()> {
		let ok = BASS_ChannelStart(self.handle());
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	fn stop(&self) -> BassResult<()> {
		let ok = BASS_ChannelStop(self.handle());
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	#[inline]
	fn update(&self, length: u32) -> BassResult<()> {
		let ok = BASS_ChannelUpdate(self.handle(), length);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}
}

impl PartialEq for dyn handle::HasHandle {
	fn eq(&self, other: &Self) -> bool {
		self.handle() == other.handle()
	}
}
impl Eq for dyn handle::HasHandle {}

impl PartialEq<DWORD> for dyn handle::HasHandle {
	fn eq(&self, other: &DWORD) -> bool {
		self.handle() == *other
	}
}

#[cfg(feature = "mixer")]
pub(crate) mod mixer {
	pub trait MixableChannel {}
}

#[cfg(feature = "mixer")]
pub trait MixerSource: Channel + mixer::MixableChannel {
	fn mixer_channel_active(&self) -> DWORD {
		BASS_Mixer_ChannelIsActive(self.handle())
	}

	fn mixer_channel_flag_remove(&self, flag: DWORD) -> BassResult<DWORD> {
		let ok = BASS_Mixer_ChannelFlags(self.handle(), 0, flag);
		if ok != -1 {
			Ok(ok)
		} else {
			Err(BassError::get())
		}
	}

	fn mixer_channel_flag_set(&self, flag: DWORD) -> BassResult<DWORD> {
		let ok = BASS_Mixer_ChannelFlags(self.handle(), flag, flag);
		if ok != -1 {
			Ok(ok)
		} else {
			Err(BassError::get())
		}
	}

	fn mixer_channel_get_envelope_pos(&self, env_type: DWORD) -> BassResult<(QWORD, f32)> {
		let mut value = 0.;
		let ok = BASS_Mixer_ChannelGetEnvelopePos(self.handle(), env_type, &mut value);
		if ok.0 as i64 != -1 {
			Ok((ok, value))
		} else {
			Err(BassError::get())
		}
	}

	fn mixer_channel_get_level(&self) -> BassResult<(u16, u16)> {
		let ok = BASS_Mixer_ChannelGetLevel(self.handle());
		if ok.0 as i32 != -1 {
			Ok(make_word(ok))
		} else {
			Err(BassError::get())
		}
	}

	fn mixer_channel_get_level_ex(&self, length: f32, flags: Option<DWORD>) -> BassResult<Vec<f32>> {
		let mono = (flags.unwrap_or_default() & BASS_LEVEL_MONO) == BASS_LEVEL_MONO;
		let stereo = (flags.unwrap_or_default() & BASS_LEVEL_STEREO) == BASS_LEVEL_STEREO;
		if mono || stereo {
			let mut levels = Vec::with_capacity(2);
			#[cfg(debug_assertions)]
			println!("Getting levels in mono or stereo mode.");
			let ok = unsafe {
				BASS_Mixer_ChannelGetLevelEx(self.handle(), levels.as_mut_ptr(), length, flags.unwrap_or_default())
			};
			if ok {
				unsafe {
					levels.set_len(if mono { 1 } else { 2 });
				}
				Ok(levels)
			} else {
				Err(BassError::get())
			}
		} else {
			let mut info = BASS_CHANNELINFO::default();
			let ok = BASS_ChannelGetInfo(self.handle(), &mut info); // This should always succeed.
			if ok {
				let chan_count = info.chans.0 as usize;
				let mut levels = Vec::with_capacity(chan_count);
				#[cfg(debug_assertions)]
				println!("Getting levels for channel with {chan_count} channels.");
				let ok = unsafe {
					BASS_Mixer_ChannelGetLevelEx(self.handle(), levels.as_mut_ptr(), length, flags.unwrap_or_default())
				};
				if ok {
					unsafe {
						levels.set_len(chan_count);
					}
					Ok(levels)
				} else {
					Err(BassError::get())
				}
			} else {
				Err(BassError::get())
			}
		}
	}

	// fn mixer_channel_get_matrix(&self) -> BassResult<Vec<Vec<f32>>> {}

	fn mixer_channel_get_mixer(&self) -> BassResult<HSTREAM> {
		let ok = BASS_Mixer_ChannelGetMixer(self.handle());
		if ok != 0 {
			Ok(ok)
		} else {
			Err(BassError::get())
		}
	}

	fn mixer_channel_get_position(&self, mode: DWORD) -> BassResult<u64> {
		let value = BASS_Mixer_ChannelGetPosition(self.handle(), mode);
		if value.0 as i64 != -1 {
			Ok(value.0)
		} else {
			Err(BassError::get())
		}
	}

	fn mixer_channel_get_position_ex(&self, mode: DWORD, delay: impl Into<DWORD>) -> BassResult<u64> {
		let value = BASS_Mixer_ChannelGetPositionEx(self.handle(), mode, delay);
		if value.0 as i64 != -1 {
			Ok(value.0)
		} else {
			Err(BassError::get())
		}
	}

	fn mixer_channel_remove(&self) -> BassResult<()> {
		let ok = BASS_Mixer_ChannelRemove(self.handle());
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	/// Helper. Equivalent to a call to `mixer_channel_set_envelope` with an empty `nodes` array.
	fn mixer_channel_remove_envelope(&self, env_type: DWORD) -> BassResult<()> {
		let ok = unsafe { BASS_Mixer_ChannelSetEnvelope(self.handle(), env_type, null_mut::<BASS_MIXER_NODE>(), 0) };
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	/// Pass an empty slice to remove the envelope `env_type`
	fn mixer_channel_set_envelope(&self, env_type: DWORD, nodes: &mut [BASS_MIXER_NODE]) -> BassResult<()> {
		let ok = unsafe { BASS_Mixer_ChannelSetEnvelope(self.handle(), env_type, nodes.as_mut_ptr(), nodes.len()) };
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	fn mixer_channel_set_envelope_position(&self, env_type: DWORD, position: impl Into<QWORD>) -> BassResult<()> {
		let ok = BASS_Mixer_ChannelSetEnvelopePos(self.handle(), env_type, position);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	// TODO :: ChannelSetEnvelopePos

	/// I'm uncertain about the soundness of this due to memory and safety and such-like.
	unsafe fn mixer_channel_set_matrix(&self, mut matrix: Vec<Vec<f32>>, time: Option<f32>) -> BassResult<()> {
		let mut matrix = matrix.iter_mut().map(|v| v.as_mut_ptr()).collect::<Vec<_>>();
		let ok = if let Some(time) = time {
			unsafe { BASS_Mixer_ChannelSetMatrixEx(self.handle(), matrix.as_mut_ptr() as *mut c_void, time) }
		} else {
			unsafe { BASS_Mixer_ChannelSetMatrix(self.handle(), matrix.as_mut_ptr() as *mut c_void) }
		};
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	fn mixer_channel_set_position(&self, position: impl Into<QWORD>, mode: DWORD) -> BassResult<()> {
		let ok = BASS_Mixer_ChannelSetPosition(self.handle(), position, mode);
		if ok {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	/// See docs for `Channel::set_sync`
	fn mixer_channel_set_sync<T: Send + Sync>(
		&self,
		sync_type: DWORD,
		parameter: impl Into<QWORD>,
		proc: impl FnMut(&mut T, HSYNC, DWORD, DWORD) + Send + Sync + 'static,
		user_data: T,
	) -> BassResult<BassSync<T>> {
		// let sync = BASS_ChannelSetSync(self.handle(), sync_type & (!BASS_SYNC_ONETIME), parameter, proc, null_mut() as *mut c_void);
		let data = Box::new(user_data);
		let user = Arc::new(Mutex::new(SyncUserData(Box::new(proc), data)));
		let weak = Arc::downgrade(&user);
		let sync = unsafe {
			BASS_Mixer_ChannelSetSync(
				self.handle(),
				sync_type,
				parameter,
				Some(sync_handler::<T>),
				weak.into_raw() as *mut c_void,
			)
		};
		println!("Mixer Sync: {:?}", sync);
		if sync != 0 {
			Ok(BassSync { sync, channel: self.handle(), user })
		} else {
			Err(BassError::get())
		}
	}
}
