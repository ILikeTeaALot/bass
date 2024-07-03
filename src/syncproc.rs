use std::{
	os::raw::c_void,
	sync::{Arc, Weak},
};
use std::sync::Mutex;

use bass_sys::{DWORD, HSYNC};

pub(crate) fn use_weak_pointer<T, F: Fn(&T) -> ()>(ptr: *const T, function: F) {
	let ptrcpy = ptr;
	let weak = unsafe {
		// **DO NOT** use Box::from_raw on this... it will cause issues... (see docs)
		if ptr.is_null() {
			println!("user_data is null");
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
/// <strike>This will also have to be reimplemented again, and again, and again,
/// for every Managed Bass component that can register procs...</strike>
pub(crate) unsafe extern "C" fn sync_proc<T: Send + Sync, F: Fn(HSYNC, DWORD, DWORD, &T) -> ()>(
	handle: HSYNC,
	channel: DWORD,
	data: DWORD,
	// user: *mut Sync<F, T>,
	user: *mut c_void,
) {
	use_weak_pointer(user as *const SyncData<T>, |proc_data| {
		// let guard = proc_data.safe_lock();
		// let sync = guard.get(&handle);
		let sync = proc_data;
		println!("Running sync proc");
		// let sync = *sync;
		(sync.function)(handle, channel, data, &sync.user_data);
		// use_arc_pointer(*sync.function as *mut F, |function| {
		// 	let user_data = *sync.user_data;
		// 	use_arc_pointer(user_data as *mut T, |user_data| {
		// 	});
		// });
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
	consume_arc_pointer(user as *mut SyncData<T>, |sync_data| {
		// consume_arc_pointer(*sync_data.function as *mut F, |function| {
		// 	let user_data = *sync_data.user_data;
		// 	consume_arc_pointer(user_data as *mut T, |user_data| {
		// 		function(handle, channel, data, user_data);
		// 	});
		// });
		(sync_data.function)(handle, channel, data, &sync_data.user_data);
	});
}

// extern "C" fn dev_format_sync(handle: HSYNC, channel: DWORD, data: DWORD, user: Null) {}

pub(crate) struct SyncData<T: Send + Sync> {
	pub function: Arc<dyn Fn(HSYNC, DWORD, DWORD, &T) + Send + Sync>,
	pub user_data: Arc<T>,
}

// /// TODO: Verify this.
// unsafe impl<T: Send> Send for SyncData<T> {}
// /// TODO: Verify this.
// unsafe impl<T: Sync> Sync for SyncData<T> {}

/// # Safety
//
// This exists to store a send+sync UserData Arc in non-generic structs (i.e. [`Mixer`](crate::mixer::Mixer))
///
/// This is a wrapper around a raw `Arc` pointer to a verified thread-safe T type.
///
/// **THIS IS STRICTLY FOR INTERNAL, CAREFULLY WRITTEN USE.**
///
/// **IF, SOMEHOW, YOU HAVE THIS OUTSIDE OF [`bass`], PLEASE TAKE THE TIME TO EXAMINE ITS USE.**
#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct CallbackUserData(*const c_void);

// impl<T: Send + Sync> From<*const T> for CallbackUserData {
//     fn from(value: *const T) -> Self {
//         Self(value as *const c_void)
//     }
// }

unsafe impl Send for CallbackUserData {}
unsafe impl Sync for CallbackUserData {}

// #[repr(transparent)]
// pub(crate) struct SyncFn(*const c_void);

/// # Safety
///
/// All efforts have been made to make this function fool-proof.
///
/// Thread safety is enforced by the `T: Send + Sync` and `F: Fn(...) + Send + Sync` requirements.
///
/// However, incorrect usage of the returned data when directly interacting with BASS functions could cause UB; as such, take care!
///
/// ## Return
///
/// The returned SyncData from this function can be passed as the `user: *mut c_void` parameter in BASS functions.
pub(crate) fn make_sync_data<T: Send + Sync, F: Fn(HSYNC, DWORD, DWORD, &T) + Send + Sync>(function: F, user_data: T) -> Arc<SyncData<CallbackUserData>> {
	let function = Arc::new(function);
	let function: *const (dyn Fn(HSYNC, DWORD, DWORD, &T) + Send + Sync) = Arc::into_raw(function);
	let function = function as *const (dyn Fn(HSYNC, DWORD, DWORD, &CallbackUserData) + Send + Sync);
	let function = unsafe { Arc::from_raw(function) };
	// let function = Arc::new(Arc::into_raw(function) as *const SyncPtr);
	// let function = Arc::new(Arc::into_raw(function));
	// self.sync_functions.insert(sync, ptr);
	let user_data = Arc::into_raw(Arc::new(user_data));
	let user_data = unsafe { Arc::from_raw(user_data as *const CallbackUserData) };
	// let user_data = Arc::new(Arc::into_raw(user_data) as *const SyncPtr);
	// let user_data = Arc::new(Arc::into_raw(user_data) as *const c_void);
	// self.sync_data.insert(sync, user_data as *const c_void);
	Arc::new(SyncData { function, user_data })
}
