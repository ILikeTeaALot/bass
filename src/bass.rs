//! The primary system for handling BASS.
//! 
//! The recommended way to use this is to only initialise 1 "instance" of it.
//! 
//! This may be enforced later by silently wrapping some variety of a static `Mutex<Bass>`.

use std::ffi::OsStr;
use std::{
	collections::HashMap,
	ffi::{c_char, c_int, c_void, CStr, OsString},
};

use bass_sys::BASS_MIXER_BUFFER;
use bass_sys::BOOL::TRUE;
use bass_sys::*;
use widestring::U16CString;

use crate::{enums::BassErrorCode, error::BassError, null::NULL, result::BassResult};

pub struct BassOptions {
	pub device: Option<c_int>,
	pub frequency: u32,
}

impl Default for BassOptions {
	fn default() -> Self {
		Self { device: None, frequency: 44100 }
	}
}

#[derive(Debug)]
pub struct BassDeviceInfo {
	name: String,
	driver: String,
	#[allow(unused)]
	flags: DWORD,
}

pub struct Bass {
	device: DWORD,
	plugins: HashMap<OsString, HPLUGIN>,
}

impl Bass {
	/// Create a new "managed" BASS instance.
	///
	/// NOTE: BASS_CONFIG_UNICODE is always enabled.
	pub fn new(
		device: Option<i32>,
		frequency: u32,
		config: Option<&[(DWORD, DWORD)]>,
		plugin_names: Option<&[&str]>, // plugin_names: Option<&[impl AsRef<OsStr>]>
	) -> Result<Self, BassError> {
		// Explictly enable the default device.
		BASS_SetConfig(BASS_CONFIG_DEV_DEFAULT, TRUE);
		// Buffer and update period
		BASS_SetConfig(BASS_CONFIG_UPDATEPERIOD, 20);
		BASS_SetConfig(BASS_CONFIG_BUFFER, 500);
		BASS_SetConfig(BASS_MIXER_BUFFER, 10);
		if let Some(config) = config {
			for (setting, value) in config.into_iter() {
				BASS_SetConfig(*setting, *value);
			}
		}
		// Force UTF-8 for BASS_*INFO structs on all platforms.
		BASS_SetConfig(BASS_CONFIG_UNICODE, TRUE);
		let initialised = unsafe { BASS_Init(device.unwrap_or(-1), frequency, 0, NULL, NULL) };
		if initialised {
			let device = BASS_GetDevice();
			let mut plugins: HashMap<OsString, HPLUGIN> = HashMap::new();
			if let Some(plugins_to_load) = plugin_names {
				for plugin in plugins_to_load {
					let identifier = OsString::from(plugin);
					if let Ok(name) = U16CString::from_os_str(plugin) {
						let handle = unsafe { BASS_PluginLoad(name.as_ptr() as *const c_char, BASS_UNICODE) };
						if handle == 0 {
							return BassError::result();
						}
						plugins.insert(identifier, handle);
					}
				}
			}
			Ok(Bass { device, plugins })
		} else {
			BassError::result()
		}
	}

	pub fn device(&self) -> DWORD {
		self.device
	}

	fn devices_internal(print: bool) -> Vec<BassDeviceInfo> {
		let mut devices: Vec<BassDeviceInfo> = Vec::new();
		for i in 0..*BASS_NODEVICE {
			let mut info = bass_sys::BassDeviceInfo::new(NULL as *const c_char, NULL as *const c_char, 0);
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

	pub fn devices_debug() -> Vec<BassDeviceInfo> {
		Self::devices_internal(true)
	}

	fn free(&self) {
		for (_, handle) in &self.plugins {
			let ok = BASS_PluginFree(*handle);
			if !ok {
				// Only 1 type of error: BASS_ERROR_HANDLE
				// which means that the plugin handle doesn't exist
				// and we can just consume it and carry on
				BassError::consume();
			}
		}
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

	pub fn info() -> BassResult<BassInfo> {
		let mut info = BassInfo::default();
		// let infoptr = Box::into_raw(Box::new(info));
		let ok = BASS_GetInfo(&mut info);
		if !ok {
			BassError::result()
		} else {
			// unsafe {
			// 	// let info = *Box::from_raw(infoptr);
			// }
			Ok(info)
		}
	}
}

impl Drop for Bass {
	fn drop(&mut self) {
		self.free()
	}
}
