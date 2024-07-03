use std::{
	collections::HashMap,
	ffi::c_void,
	fmt::Debug,
	sync::{Arc, Mutex, Weak},
};
use std::ops::Deref;
use std::ptr::null_mut;

use crate::{
	enums::{BassErrorCode, ChannelState},
	error::BassError,
	null::NULL,
	result::{result, BassResult},
	syncproc::*,
};

// use bass_mixer_sys::*;
use bass_sys::*;
use util::safe_lock::SafeLock;
use crate::channel::Channel;

type SyncProcInner = unsafe extern "C" fn(handle: HSYNC, channel: DWORD, data: DWORD, user: *mut c_void);

extern "C" fn channel_was_dequeued(_handle: HSYNC, _channel: DWORD, _data: DWORD, _user_data: *mut Mixer) {
	// println!("channel {} dequeued", data);
	// use_weak_pointer(user_data, |mixer| {
	//     {
	//         let mut mixer = mixer.safe_lock();
	//         // TODO :: mixer.sync("sync", &mixer, handle, channel, data);
	//         // println!("Mixer: {:#?}", mixer);
	//         // println!("Init options: {:#?}", mixer.init_options);
	//         // println!("Channels: {:#?}", mixer.channels());
	//         // println!("Current: {:#?}", mixer.current());
	//         let volume = mixer.init_options.default_volume;
	//         mixer.set_volume(volume);
	//     }
	// })
}

// type SyncTable<T> = HashMap<HSYNC, Arc<SyncData<T>>>;

#[derive(Clone, Copy, Debug)]
pub struct MixerOptions {
	pub freq: DWORD,
	pub channels: DWORD,
	pub default_volume: f32,
}

// impl Default for MixerOptions {
// 	fn default() -> Self {
// 		Self { freq: 44100, channels: 2, default_volume: 1. }
// 	}
// }

pub struct Mixer {
	// _marker: PhantomData<PhantomPinned>,
	// _pin: PhantomPinned,
	weak_self: Weak<Mixer>,
	// sync_functions: HashMap<HSYNC, *const (dyn Fn(DWORD, DWORD, DWORD, *const c_void) -> () + 'a)>,
	// sync_functions: HashMap<HSYNC, *const c_void>,
	// sync_functions: HashMap<HSYNC, Arc<dyn Fn(DWORD, DWORD, DWORD) -> () + 'a>>,
	// sync_data: HashMap<HSYNC, *const c_void>,
	// sync_data: Mutex<SyncTable<dyn Fn(HSYNC, DWORD, DWORD, &c_void), c_void>>,
	sync_data: Mutex<HashMap<HSYNC, Arc<SyncData<
		// dyn Fn(HSYNC, DWORD, DWORD, &CallbackUserData),
		CallbackUserData
	>>>>,
	handle: Channel<HSTREAM>,
	// queue_sync: HSYNC,
	// device_format_sync: HSYNC,
	volume: Mutex<f32>,
	// device: Mutex<DWORD>,
	init_options: MixerOptions,
}

impl Debug for Mixer {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Mixer")
			// .field("_marker", &self._marker)
			// .field("_pin", &self._pin)
			.field("weak_self", &self.weak_self)
			// .field("syncs", &self.sync_functions.keys())
			.field("handle", &self.handle)
			// .field("queue_sync", &self.queue_sync)
			// .field("device_format_sync", &self.device_format_sync)
			.field("volume", &self.volume)
			// .field("device", &self.device)
			.field("init_options", &self.init_options)
			.finish()
	}
}

impl Mixer {
	pub fn new(freq: impl Into<DWORD>, channels: impl Into<DWORD>, default_volume: Option<f32>, flags: impl Into<DWORD>) -> Arc<Self> {
		let freq = freq.into();
		let channels = channels.into();
		let default_volume = default_volume.unwrap_or(1.0);
		let handle = Channel(BASS_Mixer_StreamCreate(
			freq,
			channels,
			flags,
		));
		BASS_ChannelSetAttribute(handle.handle(), BASS_ATTRIB_BUFFER, 0.);
		Arc::new_cyclic(|mixer| {
			Mixer {
				handle,
				// queue_sync: HSYNC(0),
				// device_format_sync: HSYNC(0),
				volume: Mutex::new(default_volume.clone()),
				init_options: MixerOptions { freq, channels, default_volume },
				// device: Mutex::new(BASS_ChannelGetDevice(handle)),
				weak_self: mixer.clone(),
				// sync_functions: HashMap::new(),
				sync_data: Mutex::new(HashMap::new()),
				// _marker: PhantomData,
				// _pin: PhantomPinned,
			}
		})
	}

	fn make_ptr(&self) -> *const Mixer {
		self.weak_self.clone().into_raw()
	}

	/// In many respects this is an awful, horrible, no-good function.
	///
	/// From another point of view however, it is worth mentioning that it is
	/// hard-coded to only use the types it's supplied with and to use them properly
	/// after a brief trip into C-land.
	pub fn set_sync<T: Send + Sync, F: Fn(HSYNC, DWORD, DWORD, &T) + Send + Sync>(
		&self,
		function: F,
		sync_type: impl Into<DWORD>,
		parameter: impl Into<QWORD>,
		user_data: T,
	) {
		let data = make_sync_data(function, user_data);
		// let data_ptr = Arc::downgrade(&self.sync_data.clone()).into_raw();
		let weak_ptr = Arc::downgrade(&data.clone()).into_raw();
		// let weak_ptr = Box::new(weak_ptr);
		// let weak_ptr = Box::into_raw(weak_ptr);
		let sync = BASS_ChannelSetSync(
			self.handle,
			sync_type,
			parameter,
			Some(sync_proc::<T, F>),
			// weak_ptr as *mut SyncData<c_void, dyn Fn(HSYNC, DWORD, DWORD, c_void)>,
			weak_ptr as *mut c_void,
		);
		println!("Sync handle: {:?}", sync);
		// let data_ptr = Arc::into_raw(data);
		self.sync_data.safe_lock().insert(sync, data);
	}

	pub fn remove_sync(&self, sync: HSYNC) {
		let mut sync_data = self.sync_data.safe_lock();
		if let Some(data) = sync_data.get_mut(&sync) {
			let ok = BASS_ChannelRemoveSync(self.handle, sync);
			if ok {
				// Removal handled separately incase the call to BASS_ChannelRemoveSync fails.
				let data = sync_data.remove(&sync);
				drop(data);
				// unsafe {
				// 	// Allow the Arcs to be dropped. Because the Arcs are stored locally,
				// 	// there is guaranteed to be at least 1 existing refernce.
				// 	// Whether the underlying data is dropped is irrelevant.
				// 	// let _ = Arc::from_raw(*data.function);
				// 	// let _ = Arc::from_raw(*data.user_data);
				// };
			}
		}
	}

	/// Unlike `self.set_sync()`, this function doesn't store the data separately,
	/// the Arcs are created as usual and converted to a pointer before being passed
	/// to the sync_proc as usual, where they are then consumed rather then retained.
	pub fn set_sync_once<T: Send + Sync, F: Fn(HSYNC, DWORD, DWORD, &T) + Send + Sync>(
		&self,
		function: F,
		sync_type: impl Into<DWORD>,
		parameter: impl Into<QWORD>,
		user_data: T,
	) -> HSYNC {
		let data = make_sync_data(function, user_data);
		// let data_ptr = Arc::downgrade(&self.sync_data.clone()).into_raw();
		let data_ptr = Arc::into_raw(data);
		let sync: HSYNC = BASS_ChannelSetSync(
			self.handle,
			sync_type.into() | BASS_SYNC_ONETIME,
			parameter,
			Some(sync_onetime_proc::<T, F>),
			// data_ptr as *mut SyncData<T, F>,
			data_ptr as *mut c_void,
		);
		sync
	}

	#[allow(unreachable_code)]
	/// Doesn't actually do anything...
	pub fn register_syncs(&self) {
		return;
		// let ptr = self.make_ptr();
		// self.queue_sync = BASS_ChannelSetSync(
		//     self.handle,
		//     BASS_SYNC_MIXER_QUEUE,
		//     0,
		//     channel_was_dequeued as SYNCPROC,
		//     ptr as *mut c_void,
		// );

		// let ptr = self.make_ptr();
		// self.device_format_sync = BASS_ChannelSetSync(
		//     self.handle,
		//     BASS_SYNC_DEV_FORMAT,
		//     0,
		//     dev_format_sync as *mut SYNCPROC,
		//     ptr as *mut c_void,
		// );
		println!("Mixer: {:#?}", self);
	}

	pub fn device(&self) -> DWORD {
		self.device()
	}

	pub fn set_device(&self, device: impl Into<DWORD>) -> BassResult<()> {
		let device = device.into();
		let ok = unsafe { BASS_Init(*device as i32, self.init_options.freq, 0, NULL, NULL) };
		if !ok {
			let code = BassError::get();
			return match code {
				BassErrorCode::BassErrorNotAvailable => {
					println!("Device already initialised");
					Err(BassError::from(code))
				}
				BassErrorCode::BassErrorAlready => {
					println!("Device already initialised");
					Err(BassError::from(code))
				}
				BassErrorCode::BassErrorDevice
				| BassErrorCode::BassErrorFormat
				| BassErrorCode::BassErrorMem
				| BassErrorCode::BassErrorUnknown
				| BassErrorCode::BassErrorDriver => Err(BassError::from(code)),
				// Never happens, here to keep rust happy
				// Returns "Unknown" because it should never happen.
				_ => Err(BassError::unknown()),
			};
		}
		let ok = BASS_ChannelSetDevice(self.handle, device);
		result(ok)
	}

	pub fn add(&self, channel: impl Into<DWORD>, flags: impl Into<DWORD>) -> BassResult<()> {
		result(BASS_Mixer_StreamAddChannelEx(*self.handle, channel, flags, 0, 0))
	}

	pub fn add_ex(&self, channel: impl Into<DWORD>, flags: impl Into<DWORD>, start: impl Into<QWORD>, length: impl Into<QWORD>) -> BassResult<()> {
		result(BASS_Mixer_StreamAddChannelEx(*self.handle, channel, flags, start, length))
	}

	pub fn dump(&self) -> BassResult<()> {
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
			BassError::result()
		}
	}

	pub fn flush(&self) -> BassResult<()> {
		let ok = BASS_ChannelSetPosition(self.handle, 0, BASS_POS_BYTE | BASS_POS_FLUSH);
		result(ok)
	}

	pub fn has_channel(&self, channel: impl Into<DWORD>) -> bool {
		let mixer = BASS_Mixer_ChannelGetMixer(channel);
		if mixer == 0 {
			BassError::consume()
		}
		return mixer == *self.handle;
	}

	pub fn channels_count(&self) -> DWORD {
		unsafe { BASS_Mixer_StreamGetChannels(*self.handle, null_mut::<DWORD>(), 0) }
	}

	/// If the returned `Vec` has `.len()` `0`, then there are no channels.
	pub fn channels(&self) -> BassResult<Vec<DWORD>> {
		// let count =
		//     BASS_Mixer_StreamGetChannels(self.handle, Vec::with_capacity(0).as_mut_ptr(), 0);
		// if count as i32 == -1 {
		//     return BassError::result();
		// }
		let count = self.channels_count();
		let mut channels: Vec<DWORD> = Vec::with_capacity(*count as usize);
		let inserted = unsafe { BASS_Mixer_StreamGetChannels(*self.handle, channels.as_mut_ptr(), count) };
		if *inserted as i32 == -1 {
			return BassError::result();
		}
		unsafe {
			channels.set_len(*inserted as usize);
		}
		Ok(channels)
	}

	pub fn current(&self) -> BassResult<Option<DWORD>> {
		let mut channels: Vec<DWORD> = Vec::with_capacity(1);
		let inserted = unsafe { BASS_Mixer_StreamGetChannels(*self.handle, channels.as_mut_ptr(), 1) };
		if *inserted as i32 == -1 {
			return BassError::result(); // An error occurred; pass the error up the call chain.
		} else if *inserted == 0 {
			return Ok(None); // No channels currently in Mixer.
		}
		unsafe {
			channels.set_len(*inserted as usize); // Set len
		}
		match channels.get(0) {
			Some(channel) => Ok(Some(*channel)),
			None => Err(BassError { code: -1 }), // This shouldn't be able to happen, but if it does, a horrible error occurred somewhere
		}
	}

	pub fn remove_channel(&self, channel: impl Into<DWORD>) -> BassResult<()> {
		let ok = BASS_Mixer_ChannelRemove(channel);
		result(ok)
	}

	pub fn state(&self) -> ChannelState {
		let state = BASS_ChannelIsActive(self.handle);
		ChannelState::from(state)
	}

	pub fn playing(&self) -> bool {
		match self.state() {
			ChannelState::BassActiveStopped
			| ChannelState::BassActivePaused
			| ChannelState::BassActivePausedDevice => false,
			ChannelState::BassActivePlaying
			| ChannelState::BassActiveStalled
			| ChannelState::BassActiveWaiting
			| ChannelState::BassActiveQueued => true,
		}
	}

	pub fn play(&self, restart: impl Into<BOOL>) -> BassResult<()> {
		if self.playing() {
			return result(true);
		}
		let ok = BASS_ChannelPlay(self.handle, restart.into());
		result(ok)
	}

	pub fn paused(&self) -> bool {
		self.state() == ChannelState::BassActivePaused
	}

	pub fn pause(&self) -> BassResult<()> {
		if self.paused() {
			return result(true);
		}
		let ok = BASS_ChannelPause(self.handle);
		result(ok)
	}

	pub fn volume(&self) -> f32 {
		self.volume.safe_lock().clone()
	}

	pub fn set_volume(&self, value: f32) {
		let current_volume = self.volume.safe_lock().clone();
		if f32::max(current_volume, value) - f32::min(current_volume, value) > 0.15 {
			self.slide_volume(value);
		} else {
			// *current_volume = value;
			*self.volume.safe_lock() = value;
			let ok = BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_VOL, value);
			if !ok {
				BassError::consume();
			} else {
				println!("Volume set successfully! New value: {}", current_volume.clone());
			}
		}
	}

	fn slide_volume(&self, value: f32) {
		*self.volume.safe_lock() = value;
		let ok = BASS_ChannelSlideAttribute(self.handle, BASS_ATTRIB_VOL, value, 400);
		if !ok {
			BassError::consume();
		}
	}
}

impl Drop for Mixer {
	fn drop(&mut self) {
		// BASS_ChannelRemoveSync(self.handle, self.queue_sync);
		// BASS_ChannelRemoveSync(self.handle, self.device_format_sync);
		// let syncs = self.sync_data.safe_lock();
		for (sync, data) in self.sync_data.safe_lock().iter() {
			BASS_ChannelRemoveSync(self.handle, *sync);
			unsafe {
				// Allow the Arcs to be dereferenced
				// let _ = Arc::from_raw(*data.function);
				// let _ = Arc::from_raw(*data.user_data);
			};
		}
		BASS_ChannelFree(self.handle);
		// for (_, ptr) in &self.sync_data {
		//     unsafe {
		//         // Allow the Arcs to be dereferenced
		//         let _ = Arc::from_raw(ptr);
		//     };
		// }
	}
}

impl Deref for Mixer {
	type Target = Channel<HSTREAM>;

	fn deref(&self) -> &Self::Target {
		&self.handle
	}
}