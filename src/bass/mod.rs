use std::{
	ffi::{CStr, CString},
	os::raw::c_void,
	ptr::{null, null_mut},
	slice,
};

pub mod device;
pub mod error;
pub mod plugin;

use bass_sys::*;
use device::BassDeviceInfo;
use error::{BassError, BassErrorCode};
use plugin::{Plugin, PluginFormat, PluginInfo};

use crate::BassResult;

#[derive(Debug)]
pub struct Bass;

impl Bass {
	pub fn init(device: i32, frequency: u32, flags: Option<DWORD>) -> BassResult<Self> {
		let ok = unsafe { BASS_Init(device, frequency, flags.unwrap_or(DWORD(0)), null_mut(), null_mut()) };
		if ok {
			BASS_SetConfig(BASS_CONFIG_UNICODE, TRUE);
			BASS_SetConfig(BASS_CONFIG_FLOATDSP, TRUE);
			Ok(Bass)
		} else {
			Err(BassError::get())
		}
	}

	pub unsafe fn init_window(
		device: i32,
		frequency: u32,
		flags: Option<DWORD>,
		window: *mut c_void,
	) -> BassResult<Self> {
		let ok = BASS_Init(device, frequency, flags.unwrap_or(DWORD(0)), window, null_mut());
		if ok {
			Ok(Bass)
		} else {
			Err(BassError::get())
		}
	}

	pub fn cpu(&self) -> f32 {
		BASS_GetCPU()
	}

	pub fn device(&self) -> u32 {
		BASS_GetDevice().0
	}

	pub fn set_device(&self, device: i32) -> BassResult<()> {
		if BASS_SetDevice(device) {
			Ok(())
		} else {
			Err(BassError::get())
		}
	}

	fn devices_internal(print: bool) -> Vec<BassDeviceInfo> {
		let mut devices: Vec<BassDeviceInfo> = Vec::new();
		for i in 0..*BASS_NODEVICE {
			// let mut info = BASS_DEVICEINFO::new(NULL as *const c_char, NULL as *const c_char, 0);
			let mut info = BASS_DEVICEINFO::new(null(), null(), 0);
			// let infoptr = Box::into_raw(Box::new(info));
			let ok = BASS_GetDeviceInfo(i, &mut info);
			if !ok {
				BassError::consume();
				break;
			}
			// This is **soooooo** fun...
			// Aaaahhh the intricacies of interacting with C...
			unsafe {
				// let info = *Box::from_raw(infoptr);
				let mut better_info = BassDeviceInfo { name: String::new(), driver: String::new(), flags: info.flags };
				if !info.name.is_null() {
					let name = CStr::from_ptr(info.name);
					if print {
						println!("Device: {:#?}", name)
					}
					if let Ok(name) = name.to_str() {
						better_info.name = String::from(name);
					}
				}
				if !info.driver.is_null() {
					let driver = CStr::from_ptr(info.driver);
					if print {
						println!("Driver: {:#?}", driver)
					}
					if let Ok(driver) = driver.to_str() {
						better_info.driver = String::from(driver);
					}
				}
				devices.push(better_info);
			}
		}
		if print {
			println!("Devices: {:#?}", devices)
		}
		return devices;
	}

	pub fn devices() -> Vec<BassDeviceInfo> {
		Self::devices_internal(false)
	}

	pub fn get_error(&self) -> BassErrorCode {
		BassError::get()
	}

	fn free(&self) {
		for i in 1..*BASS_NODEVICE {
			println!("Freeing device: {}", i);
			let ok = BASS_SetDevice(i);
			if !ok {
				let code = BassError::get();
				match code {
					// Device is invalid -> We've reached the end of the dvice list and can stop
					BassErrorCode::BassErrorDevice => {
						break;
					}
					// BASS_Init was never called on this device -> continue.
					BassErrorCode::BassErrorInit => (),
					_ => (),
				}
			}
			let ok = BASS_Free();
			if !ok {
				let code = BassError::get();
				match code {
					BassErrorCode::BassErrorBusy => {
						panic!("BASS attempted to free a device that was in the process of being reinitialised.");
					}
					BassErrorCode::BassErrorInit => (),
					_ => (),
				}
				break;
			}
		}
	}

	pub fn load_plugin(&self, plugin: &str) -> BassResult<Plugin> {
		let file = CString::new(plugin);
		match file {
			Ok(file) => {
				let ok = unsafe { BASS_PluginLoad(file.as_ptr(), 0) };
				if ok == 0 {
					Err(BassError::get())
				} else {
					Ok(plugin::plugin(ok))
				}
			}
			Err(_) => Err(BassError::get()),
		}
	}

	pub fn unload_plugin(&self, plugin: Plugin) {
		BASS_PluginFree(plugin.0);
	}

	/// This doesn't require any checking because any `Plugin` that exists has a valid handle.
	pub fn enable_plugin(&self, plugin: &Plugin) {
		BASS_PluginEnable(plugin.0, true);
	}

	/// This doesn't require any checking because any `Plugin` that exists has a valid handle.
	pub fn disable_plugin(&self, plugin: &Plugin) {
		BASS_PluginEnable(plugin.0, false);
	}

	pub fn plugin_info(&self, plugin: &Plugin) -> BassResult<PluginInfo> {
		let mut info = PluginInfo::default();
		let ok = BASS_PluginGetInfo(plugin.0);
		if ok == null() {
			Err(BassError::get())
		} else {
			let bass = unsafe { *ok };
			info.version = bass.version.0;
			// let formats = Vec::<PluginFormat>::with_capacity(bass.formatc.0 as usize);
			let slice = unsafe { slice::from_raw_parts(bass.formats, bass.formatc.0 as usize) };
			let vec: Vec<PluginFormat> = slice
				.iter()
				.map(|plugin| {
					let name = String::from(unsafe { CStr::from_ptr(plugin.name) }.to_string_lossy());
					let exts = String::from(unsafe { CStr::from_ptr(plugin.exts) }.to_string_lossy());
					PluginFormat { channel_type: plugin.ctype, name, exts }
				})
				.collect();
			info.formats = vec;
			Ok(info)
		}
	}
}

impl Drop for Bass {
	fn drop(&mut self) {
		#[cfg(debug_assertions)]
		println!("Dropping BASS");
		self.free();
	}
}
