use bass_sys::DWORD;

#[derive(Debug)]
pub struct BassDeviceInfo {
	pub name: String,
	pub driver: String,
	#[allow(unused)]
	pub flags: DWORD,
}