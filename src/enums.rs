use std::ffi::c_int;

use bass_sys::{BASS_ACTIVE_PAUSED, BASS_ACTIVE_PAUSED_DEVICE, BASS_ACTIVE_PLAYING, BASS_ACTIVE_QUEUED, BASS_ACTIVE_STALLED, BASS_ACTIVE_STOPPED, BASS_ACTIVE_WAITING, BASS_ERROR_ALREADY, BASS_ERROR_BUFLOST, BASS_ERROR_BUSY, BASS_ERROR_CODEC, BASS_ERROR_CREATE, BASS_ERROR_DECODE, BASS_ERROR_DEVICE, BASS_ERROR_DRIVER, BASS_ERROR_DX, BASS_ERROR_EMPTY, BASS_ERROR_ENDED, BASS_ERROR_FILEFORM, BASS_ERROR_FILEOPEN, BASS_ERROR_FORMAT, BASS_ERROR_FREQ, BASS_ERROR_HANDLE, BASS_ERROR_ILLPARAM, BASS_ERROR_ILLTYPE, BASS_ERROR_INIT, BASS_ERROR_MEM, BASS_ERROR_NO3D, BASS_ERROR_NOCHAN, BASS_ERROR_NOEAX, BASS_ERROR_NOFX, BASS_ERROR_NOHW, BASS_ERROR_NONET, BASS_ERROR_NOPLAY, BASS_ERROR_NOTAUDIO, BASS_ERROR_NOTAVAIL, BASS_ERROR_NOTFILE, BASS_ERROR_POSITION, BASS_ERROR_PROTOCOL, BASS_ERROR_REINIT, BASS_ERROR_SPEAKER, BASS_ERROR_SSL, BASS_ERROR_START, BASS_ERROR_TIMEOUT, BASS_ERROR_UNKNOWN, BASS_ERROR_UNSTREAMABLE, BASS_ERROR_VERSION, BASS_OK, DWORD, QWORD};

/// The Represented values here are... imperfect.
///
/// I'd prefer to find a nicer way to write this,
/// such as representing it as a DWORD directly...
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ChannelState {
    BassActiveStopped = BASS_ACTIVE_STOPPED.0,
    BassActivePlaying = BASS_ACTIVE_PLAYING.0,
    BassActiveStalled = BASS_ACTIVE_STALLED.0,
    BassActivePaused = BASS_ACTIVE_PAUSED.0,
    BassActivePausedDevice = BASS_ACTIVE_PAUSED_DEVICE.0,
    BassActiveWaiting = BASS_ACTIVE_WAITING.0,
    BassActiveQueued = BASS_ACTIVE_QUEUED.0,
}

impl From<u32> for ChannelState {
    fn from(value: u32) -> Self {
        match DWORD(value) {
			BASS_ACTIVE_STOPPED => ChannelState::BassActiveStopped,
			BASS_ACTIVE_PLAYING => ChannelState::BassActivePlaying,
			BASS_ACTIVE_STALLED => ChannelState::BassActiveStalled,
			BASS_ACTIVE_PAUSED => ChannelState::BassActivePaused,
			BASS_ACTIVE_PAUSED_DEVICE => ChannelState::BassActivePausedDevice,
			BASS_ACTIVE_WAITING => ChannelState::BassActiveWaiting,
			BASS_ACTIVE_QUEUED => ChannelState::BassActiveQueued,
			_ => ChannelState::BassActiveStopped,
		}
    }
}

impl From<DWORD> for ChannelState {
    fn from(value: DWORD) -> Self {
        value.0.into()
    }
}

impl From<ChannelState> for DWORD {
    fn from(value: ChannelState) -> Self {
        DWORD(value as u32)
    }
}

#[derive(PartialEq, Eq)]
#[repr(u32)]
pub enum BassErrorCode {
	BassOk = BASS_OK, // all is OK
	BassErrorMem = BASS_ERROR_MEM, // memory error
	BassErrorFileOpen = BASS_ERROR_FILEOPEN, // can't open the file
	BassErrorDriver = BASS_ERROR_DRIVER, // can't find a free/valid driver
	BassErrorBufferLost = BASS_ERROR_BUFLOST, // the sample buffer was lost
	BassErrorHandle = BASS_ERROR_HANDLE, // invalid handle
	BassErrorFormat = BASS_ERROR_FORMAT, // unsupported sample format
	BassErrorPosition = BASS_ERROR_POSITION, // invalid position
	BassErrorInit = BASS_ERROR_INIT, // BASS_Init has not been successfully called
	BassErrorStart = BASS_ERROR_START, // BASS_Start has not been successfully called
	BassErrorSsl = BASS_ERROR_SSL, // SSL/HTTPS support isn't available
	BassErrorReInit = BASS_ERROR_REINIT, // device needs to be reinitialized
	BassErrorAlready = BASS_ERROR_ALREADY, // already initialized/paused/whatever
	BassErrorNotAudio = BASS_ERROR_NOTAUDIO, // file does not contain audio
	BassErrorNoChan = BASS_ERROR_NOCHAN, // can't get a free channel
	BassErrorIllType = BASS_ERROR_ILLTYPE, // an illegal type was specified
	BassErrorIllParam = BASS_ERROR_ILLPARAM, // an illegal parameter was specified
	BassErrorNo3D = BASS_ERROR_NO3D, // no 3D support
	BassErrorNoEax = BASS_ERROR_NOEAX, // no EAX support
	BassErrorDevice = BASS_ERROR_DEVICE, // illegal device number
	BassErrorNoPlay = BASS_ERROR_NOPLAY, // not playing
	BassErrorFreq = BASS_ERROR_FREQ, // illegal sample rate
	BassErrorNotFile = BASS_ERROR_NOTFILE, // the stream is not a file stream
	BassErrorNoHw = BASS_ERROR_NOHW, // no hardware voices available
	BassErrorEmpty = BASS_ERROR_EMPTY, // the MOD music has no sequence data
	BassErrorNoNet = BASS_ERROR_NONET, // no internet connection could be opened
	BassErrorCreate = BASS_ERROR_CREATE, // couldn't create the file
	BassErrorNoFx = BASS_ERROR_NOFX, // effects are not available
	BassErrorNotAvailable = BASS_ERROR_NOTAVAIL, // requested data/action is not available
	BassErrorDecode = BASS_ERROR_DECODE, // the channel is/isn't a "decoding channel"
	BassErrorDx = BASS_ERROR_DX, // a sufficient DirectX version is not installed
	BassErrorTimeout = BASS_ERROR_TIMEOUT, // connection timedout
	BassErrorFileForm = BASS_ERROR_FILEFORM, // unsupported file format
	BassErrorSpeaker = BASS_ERROR_SPEAKER, // unavailable speaker
	BassErrorVersion = BASS_ERROR_VERSION, // invalid BASS version (used by add-ons)
	BassErrorCodec = BASS_ERROR_CODEC, // codec is not available/supported
	BassErrorEnded = BASS_ERROR_ENDED, // the channel/file has ended
	BassErrorBusy = BASS_ERROR_BUSY, // the device is busy
	BassErrorUnstreamable = BASS_ERROR_UNSTREAMABLE, // unstreamable file
	BassErrorProtocol = BASS_ERROR_PROTOCOL, // unsupported protocol
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

pub enum Position {
	Bytes(QWORD),
	Seconds(f64),
}

use Position::*;

impl From<f64> for Position {
    fn from(value: f64) -> Self {
        Seconds(value)
    }
}

impl From<QWORD> for Position {
    fn from(value: QWORD) -> Self {
        Bytes(value)
    }
}