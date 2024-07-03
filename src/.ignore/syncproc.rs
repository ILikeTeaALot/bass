use std::{
	os::raw::c_void,
	sync::{Arc, Weak},
};

use bass_sys::{DWORD, HSYNC};

pub(crate) fn use_weak_pointer<T, F: Fn(&T) -> ()>(ptr: *mut T, function: F) {
	let ptrcpy = ptr;
	let weak = unsafe {
		// **DO NOT** use Box::from_raw on this... it will cause issues... (see docs)
		if ptr.is_null() {
			// println!("user_data is null");
			return;
		}
		Weak::from_raw(ptr)
	};
	{
		let option = weak.upgrade();
		let arc = match option {
			Some(mixer) => mixer,
			None => {
				// println!("Pointer gone.");
				return;
			}
		};
		function(&arc);
	}
	let ptr = weak.into_raw();
	// println!("Eq: {}", ptr == ptrcpy);
	debug_assert_eq!(ptr, ptrcpy);
}

pub(crate) fn use_arc_pointer<T, F: Fn(&T) -> ()>(ptr: *mut T, function: F) {
	let ptrcpy = ptr;
	let arc = unsafe {
		// **DO NOT** use Box::from_raw on this... it will cause issues... (see docs)
		if ptr.is_null() {
			// println!("Arc is null");
			return;
		}
		Arc::from_raw(ptr)
	};
	{
		function(&arc);
	}
	let ptr = Arc::into_raw(arc);
	// println!("Eq: {}", ptr == ptrcpy);
	debug_assert_eq!(ptr, ptrcpy);
}

pub(crate) fn consume_arc_pointer<T, F: Fn(&T) -> ()>(ptr: *mut T, function: F) {
	let arc = unsafe {
		if ptr.is_null() {
			return;
		}
		Arc::from_raw(ptr)
	};
	{
		function(&arc);
	}
	drop(arc);
}

/// ***This*** is truly evil...
///
/// <strikethrough>This will also have to be reimplemented again, and again, and again,
/// for every Managed Bass component that can register procs...</strikethrough>
pub(crate) unsafe extern "C" fn sync_proc<T: Send + Sync, F: Fn(HSYNC, DWORD, DWORD, &T) -> ()>(
	handle: HSYNC,
	channel: DWORD,
	data: DWORD,
	// user: *mut Sync<F, T>,
	user: *mut c_void,
) {
	use_weak_pointer(user as *mut SyncData<F, T>, |proc_data| {
		// let guard = proc_data.safe_lock();
		// let sync = guard.get(&handle);
		let sync = proc_data;
		// let sync = *sync;
		use_arc_pointer(*sync.function as *mut F, |function| {
			let user_data = *sync.user_data;
			use_arc_pointer(user_data as *mut T, |user_data| {
				function(handle, channel, data, user_data);
			});
		});
	});
}

/// This is *relatively* less evil than the many-time
/// sync proc, but it's still pretty evil.
///
/// This function consumes all the ARCs rather than maintaining them.
pub(crate) extern "C" fn sync_onetime_proc<T: Send + Sync, F: Fn(HSYNC, DWORD, DWORD, &T) -> ()>(
	handle: HSYNC,
	channel: DWORD,
	data: DWORD,
	// user: *mut Sync<F, T>,
	user: *mut c_void,
) {
	consume_arc_pointer(user as *mut SyncData<F, T>, |sync_data| {
		consume_arc_pointer(*sync_data.function as *mut F, |function| {
			let user_data = *sync_data.user_data;
			consume_arc_pointer(user_data as *mut T, |user_data| {
				function(handle, channel, data, user_data);
			});
		});
	});
}

// extern "C" fn dev_format_sync(handle: HSYNC, channel: DWORD, data: DWORD, user: Null) {}

pub(crate) struct SyncData<F, T: Send + Sync> {
	pub function: Arc<*const F>,
	pub user_data: Arc<*const T>,
}

// unsafe impl<F: Send + Sync, T: Send + Sync> Send for SyncData<F, T> {}
// unsafe impl<F: Send + Sync, T: Send + Sync> Sync for SyncData<F, T> {}

// #[repr(transparent)]
// pub(crate) struct SyncFn(*const c_void);

#[repr(transparent)]
pub(crate) struct SyncPtr(*const c_void);

unsafe impl Send for SyncPtr {}
unsafe impl Sync for SyncPtr {}

pub(crate) fn make_sync_data<T, F: Fn(HSYNC, DWORD, DWORD, &T)>(function: F, user_data: T) -> Arc<SyncData<SyncPtr, SyncPtr>> {
	let function = Arc::new(function);
	// let function = Arc::new(Arc::into_raw(function) as *const SyncPtr);
	let function = Arc::new(SyncPtr(Arc::into_raw(function) as *const c_void));
	// self.sync_functions.insert(sync, ptr);
	let user_data = Arc::new(user_data);
	// let user_data = Arc::new(Arc::into_raw(user_data) as *const SyncPtr);
	let user_data = Arc::new(SyncPtr(Arc::into_raw(user_data) as *const c_void));
	// self.sync_data.insert(sync, user_data as *const c_void);
	Arc::new(SyncData { function, user_data })
}
