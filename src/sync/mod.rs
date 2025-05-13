use std::{
	fmt::Debug,
	sync::{Arc, Mutex},
};

use bass_sys::{BASS_ChannelRemoveSync, DWORD, HSYNC};

#[derive(Debug)]
pub struct BassSync<T: Send + Sync> {
	pub(crate) sync: HSYNC,
	pub(crate) channel: DWORD,
	/// Internal implementation detail... this is an `Arc<Mutex<SyncUserData<T>>>`
	pub(crate) user: Arc<Mutex<SyncUserData<T>>>,
}

impl<T: Send + Sync> BassSync<T> {
	pub fn handle(&self) -> HSYNC {
		self.sync
	}

	pub fn channel(&self) -> DWORD {
		self.channel
	}
}

impl<T: Send + Sync> PartialEq for BassSync<T> {
	fn eq(&self, other: &Self) -> bool {
		self.sync == other.sync
	}
}
impl<T: Send + Sync> Eq for BassSync<T> {}

unsafe impl<T: Send + Sync> Send for BassSync<T> {}
unsafe impl<T: Send + Sync> Sync for BassSync<T> {}

impl<T: Send + Sync> Drop for BassSync<T> {
	fn drop(&mut self) {
		#[cfg(debug_assertions)]
		println!("Freeing Sync {:?}", self.sync);
		let ok = BASS_ChannelRemoveSync(self.channel, self.sync);
		#[cfg(debug_assertions)]
		println!("Sync Freed: {}", ok);
		#[cfg(debug_assertions)]
		println!("Sync Removed... freeing user data...");
		#[cfg(debug_assertions)]
		{
			// let arc = unsafe { Arc::from_raw(self.user) };
			// let arc = self.user;
			println!("Strong: {}; Weak: {}", Arc::strong_count(&self.user), Arc::weak_count(&self.user));
		}
		#[cfg(not(debug_assertions))]
		{
			// let _ = unsafe { Arc::from_raw(self.user) };
			// let _ = self.user;
		}
		#[cfg(debug_assertions)]
		println!("User data freed");
	}
}

pub(crate) type SyncCallback<T> = dyn FnMut(&mut T, HSYNC, DWORD, DWORD) + Send + Sync + 'static;

#[repr(C)]
pub(crate) struct SyncUserData<T: Send + Sync>(pub(crate) Box<SyncCallback<T>>, pub(crate) Box<T>);

impl<T: Send + Sync> Debug for SyncUserData<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("SyncUserData").field(&"SyncCallback").field(&"Box<{unknown}>").finish()
	}
}
