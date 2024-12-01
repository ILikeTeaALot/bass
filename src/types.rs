use crate::bass::error::BassErrorCode;

pub type BassResult<T> = Result<T, BassErrorCode>;

pub use bass_sys::{BASS_SAMPLE, HCHANNEL, HDSP, HFX, HMUSIC, HPLUGIN, HRECORD, HSAMPLE, HSTREAM, HSYNC};

pub use bass_sys::{BYTE, DWORD, QWORD, WORD};

#[cfg(feature = "loudness")]
pub use bass_sys::HLOUDNESS;
