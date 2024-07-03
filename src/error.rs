use std::{error::Error, fmt::Display};

use bass_sys::BASS_ErrorGetCode;

use crate::enums::BassErrorCode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BassError {
	pub code: i32,
}

impl BassError {
	pub fn get() -> BassErrorCode {
		BASS_ErrorGetCode().into()
	}

	pub fn consume() {
		let _ = BASS_ErrorGetCode();
	}

	pub fn result<T>() -> Result<T, BassError> {
		Err(BassError::default())
	}

	pub fn unknown() -> BassError {
		BassError { code: -1 }
	}
}

impl Default for BassError {
	fn default() -> Self {
		let code = BASS_ErrorGetCode();
		Self { code }
	}
}

impl Display for BassError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "BASS Failed with code: {}", self.code)
	}
}

impl Error for BassError {}

impl From<BassErrorCode> for BassError {
	fn from(value: BassErrorCode) -> Self {
		BassError { code: value as i32 }
	}
}
