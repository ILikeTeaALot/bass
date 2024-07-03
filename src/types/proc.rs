use std::{ffi::c_void, marker::PhantomData, mem, ops::Deref, pin::Pin};

use bass_sys::{BASS_FILEPROCS, BOOL, DWORD, FILECLOSEPROC, FILELENPROC, FILEREADPROC, FILESEEKPROC, QWORD};

use crate::syncproc::CallbackUserData;

// type FileCloseProc<T> = extern "C" fn (user: *mut T);
// type FileLenProc<T> = extern "C" fn (user: *mut T) -> QWORD;
// type FileReadProc<T> = extern "C" fn (buffer: *mut c_void, length: DWORD, user: *mut T) -> DWORD;
// type FileSeekProc<T> = extern "C" fn (offset: QWORD, user: *mut T) -> BOOL;

#[derive(Debug)]
pub(crate) struct BassFileProcs<T: Send + Sync> {
	pub procs: BASS_FILEPROCS,
	p: PhantomData<T>,
	// handlers: Box<dyn FileProc<UserData = T>>,
	// handlers: FileProcHandlers<T>,
}

impl<T: Send + Sync> BassFileProcs<T> {
	// pub fn new_raw(close: FILECLOSEPROC, length: FILELENPROC, read: FILEREADPROC, seek: FILESEEKPROC) -> Self {
	// 	BassFileProcs {
	// 		procs: BASS_FILEPROCS { close, length, read, seek },
	// 	}
	// }

	// pub fn new_trait(handler: Box<dyn FileProc<UserData = T>>) {}
}

impl<T: Send + Sync> Deref for BassFileProcs<T> {
	type Target = BASS_FILEPROCS;

	fn deref(&self) -> &Self::Target {
		&self.procs
	}
}

fn use_box<T: ?Sized + Send + Sync, R>(raw: *mut T, function: impl Fn(&mut Box<T>) -> R) -> Result<R, ()> {
	let mut data = match raw.is_null() {
		true => return Err(()),
		false => unsafe { Box::from_raw(raw) },
	};
	let r = function(&mut data);
	Box::into_raw(data);
	Ok(r)
}

unsafe extern "C" fn file_close<T: Send + Sync>(user: *mut c_void) {
	// let user = user as *mut CallbackUserData as *mut dyn FileProc<UserData = T>;
	// let user = unsafe { mem::transmute(user) };
	let user = user as *mut FileProcHandlers<T>;
	// It doesn't matter if this errors
	let _ = use_box(user, |data| (data.close)(&mut data.user_data));
}

unsafe extern "C" fn file_len<T: Send + Sync>(user: *mut c_void) -> QWORD {
	let user = user as *mut FileProcHandlers<T>;
	match use_box(user, |data| (data.len)(&mut data.user_data)) {
		Ok(len) => len,
		Err(_) => QWORD(0),
	}
}

unsafe extern "C" fn file_read<T: Send + Sync>(buffer: *mut c_void, length: DWORD, user: *mut c_void) -> DWORD {
	let user = user as *mut FileProcHandlers<T>;
	match use_box(user, |data| (data.read)(buffer, length.0 as usize, &mut data.user_data)) {
		Ok(idk) => idk,
		Err(_) => DWORD(0),
	}
}

unsafe extern "C" fn file_seek<T: Send + Sync>(offset: QWORD, user: *mut c_void) -> BOOL {
	let user = user as *mut FileProcHandlers<T>;
	match use_box(user, |data| (data.seek)(offset, &mut data.user_data)) {
		Ok(ok) => ok.into(),
		Err(_) => true.into()
	}
}

// impl<T: Send + Sync> From<Box<dyn FileProc<UserData = T>>> for BassFileProcs<T> {
// 	fn from(handlers: Box<dyn FileProc<UserData = T>>) -> Self {
// 		BassFileProcs {
// 			procs: BASS_FILEPROCS {
// 				close: Some(file_close::<T>),
// 				length: Some(file_len::<T>),
// 				read: Some(file_read::<T>),
// 				seek: Some(file_seek::<T>),
// 			},
// 			handlers,
// 		}
// 	}
// }

impl<T: Send + Sync> From<&FileProcHandlers<T>> for BassFileProcs<T> {
	fn from(_: &FileProcHandlers<T>) -> Self {
		BassFileProcs {
			procs: BASS_FILEPROCS {
				close: Some(file_close::<T>),
				length: Some(file_len::<T>),
				read: Some(file_read::<T>),
				seek: Some(file_seek::<T>),
			},
			p: PhantomData::<T>
			// handlers,
		}
	}
}

trait FileProc: Send + Sync {
	type UserData;
	fn close(&self, user: &mut Self::UserData);
	fn len(&self, user: &mut Self::UserData) -> QWORD;
	fn read(&self, buffer: *mut c_void, length: usize, user: &mut Self::UserData) -> DWORD;
	fn seek(&self, offset: QWORD, user: &mut Self::UserData) -> bool;
}

impl FileProc for CallbackUserData {
	type UserData = ();

	fn close(&self, user: &mut Self::UserData) {}

	fn len(&self, user: &mut Self::UserData) -> QWORD {
		QWORD(0)
	}

	fn read(&self, buffer: *mut c_void, length: usize, user: &mut Self::UserData) -> DWORD {
		DWORD(0)
	}

	fn seek(&self, offset: QWORD, user: &mut Self::UserData) -> bool {
		false
	}
}

impl<T> std::fmt::Debug for Box<dyn FileProc<UserData = T>> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("FileProc")
	}
}

// fn make_sync_data<T: Send + Sync, F: Fn(HSYNC, DWORD, DWORD, &T) + Send + Sync>(function: F, user_data: T) -> Arc<SyncData<CallbackUserData>> {
// 	let function = Arc::new(function);
// 	let function: *const (dyn Fn(HSYNC, DWORD, DWORD, &T) + Send + Sync) = Arc::into_raw(function);
// 	let function = function as *const (dyn Fn(HSYNC, DWORD, DWORD, &CallbackUserData) + Send + Sync);
// 	let function = unsafe { Arc::from_raw(function) };
// 	// let function = Arc::new(Arc::into_raw(function) as *const SyncPtr);
// 	// let function = Arc::new(Arc::into_raw(function));
// 	// self.sync_functions.insert(sync, ptr);
// 	let user_data = Arc::into_raw(Arc::new(user_data));
// 	let user_data = unsafe { Arc::from_raw(user_data as *const CallbackUserData) };
// 	// let user_data = Arc::new(Arc::into_raw(user_data) as *const SyncPtr);
// 	// let user_data = Arc::new(Arc::into_raw(user_data) as *const c_void);
// 	// self.sync_data.insert(sync, user_data as *const c_void);
// 	Arc::new(SyncData { function, user_data })
// }

pub type Close<T> = fn(user: &mut T);
pub type Length<T> = fn(user: &mut T) -> QWORD;
pub type Read<T> = fn(buffer: *mut c_void, length: usize, user: &mut T) -> DWORD;
pub type Seek<T> = fn(offset: QWORD, user: &mut T) -> bool;

/// This is done as a struct rather than a trait to allow `Box`ing and sending over C-ffi without a fat pointer.
#[derive(Debug)]
pub struct FileProcHandlers<T: Send + Sync> {
	close: Close<T>,
	len: Length<T>,
	read: Read<T>,
	seek: Seek<T>,
	user_data: Box<T>,
}

impl<T: Send + Sync> FileProcHandlers<T> {
	pub fn new(user_data: T, close: Close<T>, len: Length<T>, read: Read<T>, seek: Seek<T>) -> Self {
		Self { close, len, read, seek, user_data: Box::new(user_data) }
	}
}
