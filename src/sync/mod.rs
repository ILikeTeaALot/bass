use bass_sys::{BASS_ChannelRemoveSync, DWORD, HSYNC};

#[derive(Debug)]
pub struct BassSync<T: Send + Sync> {
	pub(crate) sync: HSYNC,
	pub(crate) channel: DWORD,
	pub(crate) user: *mut SyncUserData<T>,
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
		let _ = unsafe { Box::from_raw(self.user) };
		#[cfg(debug_assertions)]
		println!("User data freed");
	}
}

pub(crate) type SyncCallback<T> = dyn FnMut(&mut T, HSYNC, DWORD, DWORD) + Send + Sync;

#[repr(C)]
pub(crate) struct SyncUserData<T: Send + Sync>(pub Box<SyncCallback<T>>, pub Box<T>);
