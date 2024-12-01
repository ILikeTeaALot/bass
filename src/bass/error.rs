use std::ffi::c_int;
use std::{error::Error, fmt::Display};

use bass_sys::BASS_ErrorGetCode;
use bass_sys::{BASS_ERROR_ALREADY, BASS_ERROR_BUFLOST, BASS_ERROR_BUSY, BASS_ERROR_CODEC, BASS_ERROR_CREATE, BASS_ERROR_DECODE, BASS_ERROR_DEVICE, BASS_ERROR_DRIVER, BASS_ERROR_DX, BASS_ERROR_EMPTY, BASS_ERROR_ENDED, BASS_ERROR_FILEFORM, BASS_ERROR_FILEOPEN, BASS_ERROR_FORMAT, BASS_ERROR_FREQ, BASS_ERROR_HANDLE, BASS_ERROR_ILLPARAM, BASS_ERROR_ILLTYPE, BASS_ERROR_INIT, BASS_ERROR_MEM, BASS_ERROR_NO3D, BASS_ERROR_NOCHAN, BASS_ERROR_NOEAX, BASS_ERROR_NOFX, BASS_ERROR_NOHW, BASS_ERROR_NONET, BASS_ERROR_NOPLAY, BASS_ERROR_NOTAUDIO, BASS_ERROR_NOTAVAIL, BASS_ERROR_NOTFILE, BASS_ERROR_POSITION, BASS_ERROR_PROTOCOL, BASS_ERROR_REINIT, BASS_ERROR_SPEAKER, BASS_ERROR_SSL, BASS_ERROR_START, BASS_ERROR_TIMEOUT, BASS_ERROR_UNKNOWN, BASS_ERROR_UNSTREAMABLE, BASS_ERROR_VERSION, BASS_OK, DWORD};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[repr(u32)]
pub enum BassErrorCode {
	#[error("All is OK")]
	BassOk = BASS_OK, // all is OK
	#[error("Memory Error")]
	BassErrorMem = BASS_ERROR_MEM, // memory error
	#[error("Can't open the file")]
	BassErrorFileOpen = BASS_ERROR_FILEOPEN, // can't open the file
	#[error("Can't find a free/valid driver")]
	BassErrorDriver = BASS_ERROR_DRIVER, // can't find a free/valid driver
	#[error("The sample buffer was lost")]
	BassErrorBufferLost = BASS_ERROR_BUFLOST, // the sample buffer was lost
	#[error("Invalid handle")]
	BassErrorHandle = BASS_ERROR_HANDLE, // invalid handle
	#[error("Unsupported sample format")]
	BassErrorFormat = BASS_ERROR_FORMAT, // unsupported sample format
	#[error("Invalid position")]
	BassErrorPosition = BASS_ERROR_POSITION, // invalid position
	#[error("BASS_Init has not been successfully called")]
	BassErrorInit = BASS_ERROR_INIT, // BASS_Init has not been successfully called
	#[error("BASS_Start has not been successfully called")]
	BassErrorStart = BASS_ERROR_START, // BASS_Start has not been successfully called
	#[error("SSL/HTTPS support isn't available")]
	BassErrorSsl = BASS_ERROR_SSL, // SSL/HTTPS support isn't available
	#[error("Device needs to be reinitialized")]
	BassErrorReInit = BASS_ERROR_REINIT, // device needs to be reinitialized
	#[error("Alread initialized/paused/whatever")]
	BassErrorAlready = BASS_ERROR_ALREADY, // already initialized/paused/whatever
	#[error("File does not contain audio")]
	BassErrorNotAudio = BASS_ERROR_NOTAUDIO, // file does not contain audio
	#[error("Can't get a free channel")]
	BassErrorNoChan = BASS_ERROR_NOCHAN, // can't get a free channel
	#[error("An illegal/invalid type was specified")]
	BassErrorIllType = BASS_ERROR_ILLTYPE, // an illegal type was specified
	#[error("An illegal/invalid parameter was specified")]
	BassErrorIllParam = BASS_ERROR_ILLPARAM, // an illegal parameter was specified
	#[error("No 3D support")]
	BassErrorNo3D = BASS_ERROR_NO3D, // no 3D support
	#[error("No EAX support")]
	BassErrorNoEax = BASS_ERROR_NOEAX, // no EAX support
	#[error("Illegal/Incorrect device number")]
	BassErrorDevice = BASS_ERROR_DEVICE, // illegal device number
	#[error("Not playing")]
	BassErrorNoPlay = BASS_ERROR_NOPLAY, // not playing
	#[error("Illegal sample rate")]
	BassErrorFreq = BASS_ERROR_FREQ, // illegal sample rate
	#[error("The stream is not a file stream")]
	BassErrorNotFile = BASS_ERROR_NOTFILE, // the stream is not a file stream
	#[error("No hardware voices available")]
	BassErrorNoHw = BASS_ERROR_NOHW, // no hardware voices available
	#[error("The MOD music has no sequence data")]
	BassErrorEmpty = BASS_ERROR_EMPTY, // the MOD music has no sequence data
	#[error("No internet connection could be opened")]
	BassErrorNoNet = BASS_ERROR_NONET, // no internet connection could be opened
	#[error("Couldn't create the file")]
	BassErrorCreate = BASS_ERROR_CREATE, // couldn't create the file
	#[error("Effects are not available")]
	BassErrorNoFx = BASS_ERROR_NOFX, // effects are not available
	#[error("Requested data/action is not available")]
	BassErrorNotAvailable = BASS_ERROR_NOTAVAIL, // requested data/action is not available
	#[error("The channel is/isn't a “decoding channel”")]
	BassErrorDecode = BASS_ERROR_DECODE, // the channel is/isn't a "decoding channel"
	#[error("A sufficient DirectX version is not installed")]
	BassErrorDx = BASS_ERROR_DX, // a sufficient DirectX version is not installed
	#[error("Connection timeout")]
	BassErrorTimeout = BASS_ERROR_TIMEOUT, // connection timedout
	#[error("Unsupported file format")]
	BassErrorFileForm = BASS_ERROR_FILEFORM, // unsupported file format
	#[error("Unavailable speaker")]
	BassErrorSpeaker = BASS_ERROR_SPEAKER, // unavailable speaker
	#[error("Invalid BASS version (used by add-ons)")]
	BassErrorVersion = BASS_ERROR_VERSION, // invalid BASS version (used by add-ons)
	#[error("Codec is not available/supported")]
	BassErrorCodec = BASS_ERROR_CODEC, // codec is not available/supported
	#[error("The channel/file has ended")]
	BassErrorEnded = BASS_ERROR_ENDED, // the channel/file has ended
	#[error("The device is busy")]
	BassErrorBusy = BASS_ERROR_BUSY, // the device is busy
	#[error("Unstreamable file")]
	BassErrorUnstreamable = BASS_ERROR_UNSTREAMABLE, // unstreamable file
	#[error("Unsupported protocol")]
	BassErrorProtocol = BASS_ERROR_PROTOCOL, // unsupported protocol
	#[error("Some other mystery problem!")]
	BassErrorUnknown = BASS_ERROR_UNKNOWN as u32, // some other mystery problem // Should be "-1" but the compiler doesn't like it
}

impl From<c_int> for BassErrorCode {
    fn from(value: c_int) -> Self {
        match value as u32 {
			BASS_OK => Self::BassOk,
			BASS_ERROR_MEM => Self::BassErrorMem,
			BASS_ERROR_FILEOPEN => Self::BassErrorFileOpen,
			BASS_ERROR_DRIVER => Self::BassErrorDriver,
			BASS_ERROR_BUFLOST => Self::BassErrorBufferLost,
			BASS_ERROR_HANDLE => Self::BassErrorHandle,
			BASS_ERROR_FORMAT => Self::BassErrorFormat,
			BASS_ERROR_POSITION => Self::BassErrorPosition,
			BASS_ERROR_INIT => Self::BassErrorInit,
			BASS_ERROR_START => Self::BassErrorStart,
			BASS_ERROR_SSL => Self::BassErrorSsl,
			BASS_ERROR_REINIT => Self::BassErrorReInit,
			BASS_ERROR_ALREADY => Self::BassErrorAlready,
			BASS_ERROR_NOTAUDIO => Self::BassErrorNotAudio,
			BASS_ERROR_NOCHAN => Self::BassErrorNoChan,
			BASS_ERROR_ILLTYPE => Self::BassErrorIllType,
			BASS_ERROR_ILLPARAM => Self::BassErrorIllParam,
			BASS_ERROR_NO3D => Self::BassErrorNo3D,
			BASS_ERROR_NOEAX => Self::BassErrorNoEax,
			BASS_ERROR_DEVICE => Self::BassErrorDevice,
			BASS_ERROR_NOPLAY => Self::BassErrorNoPlay,
			BASS_ERROR_FREQ => Self::BassErrorFreq,
			BASS_ERROR_NOTFILE => Self::BassErrorNotFile,
			BASS_ERROR_NOHW => Self::BassErrorNoHw,
			BASS_ERROR_EMPTY => Self::BassErrorEmpty,
			BASS_ERROR_NONET => Self::BassErrorNoNet,
			BASS_ERROR_CREATE => Self::BassErrorCreate,
			BASS_ERROR_NOFX => Self::BassErrorNoFx,
			BASS_ERROR_NOTAVAIL => Self::BassErrorNotAvailable,
			BASS_ERROR_DECODE => Self::BassErrorDecode,
			BASS_ERROR_DX => Self::BassErrorDx,
			BASS_ERROR_TIMEOUT => Self::BassErrorTimeout,
			BASS_ERROR_FILEFORM => Self::BassErrorFileForm,
			BASS_ERROR_SPEAKER => Self::BassErrorSpeaker,
			BASS_ERROR_VERSION => Self::BassErrorVersion,
			BASS_ERROR_CODEC => Self::BassErrorCodec,
			BASS_ERROR_ENDED => Self::BassErrorEnded,
			BASS_ERROR_BUSY => Self::BassErrorBusy,
			BASS_ERROR_UNSTREAMABLE => Self::BassErrorUnstreamable,
			BASS_ERROR_PROTOCOL => Self::BassErrorProtocol,
			// BASS_ERROR_UNKNOWN => Self::BassErrorUnknown,
			_ => Self::BassErrorUnknown,
		}
    }
}

impl From<DWORD> for BassErrorCode {
    fn from(value: DWORD) -> Self {
        (value.0 as i32).into()
    }
}

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
