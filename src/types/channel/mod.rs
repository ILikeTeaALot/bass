use std::ffi::{CStr, CString};
use widestring::U16CStr;
use bass_sys::{BASS_3DVECTOR, BASS_ATTRIB_BITRATE, BASS_ATTRIB_BUFFER, BASS_ATTRIB_CPU, BASS_ATTRIB_FREQ, BASS_ATTRIB_GRANULE, BASS_ATTRIB_MUSIC_ACTIVE, BASS_ATTRIB_MUSIC_AMPLIFY, BASS_ATTRIB_MUSIC_BPM, BASS_ATTRIB_MUSIC_PANSEP, BASS_ATTRIB_MUSIC_PSCALER, BASS_ATTRIB_MUSIC_SPEED, BASS_ATTRIB_MUSIC_VOL_CHAN, BASS_ATTRIB_MUSIC_VOL_GLOBAL, BASS_ATTRIB_MUSIC_VOL_INST, BASS_ATTRIB_NET_RESUME, BASS_ATTRIB_NORAMP, BASS_ATTRIB_PAN, BASS_ATTRIB_PUSH_LIMIT, BASS_ATTRIB_SRC, BASS_ATTRIB_TAIL, BASS_ATTRIB_VOL, BASS_ATTRIB_VOLDSP, BASS_ATTRIB_VOLDSP_PRIORITY, BASS_CHANNELINFO, BASS_UNICODE, DWORD, HPLUGIN, HSAMPLE};

pub struct Bass3DPosition {
	pub position: Option<BASS_3DVECTOR>,
	pub orientation: Option<BASS_3DVECTOR>,
	pub velocity: Option<BASS_3DVECTOR>,
}

pub struct Bass3DAttributes {
	pub mode: DWORD,
	pub minimum: f32,
	pub maximum: f32,
	pub angle_of_inside_projection_cone: DWORD,
	pub angle_of_outside_projection_cone: DWORD,
	pub output_volume: f32,
}

#[repr(u32)]
pub enum ChannelGetAttribute {
	Bitrate = BASS_ATTRIB_BITRATE.0,
	Buffer = BASS_ATTRIB_BUFFER.0,
	CPU = BASS_ATTRIB_CPU.0,
	Frequency = BASS_ATTRIB_FREQ.0,
	Granule = BASS_ATTRIB_GRANULE.0,
	MusicActive = BASS_ATTRIB_MUSIC_ACTIVE.0,
	MusicAmplify = BASS_ATTRIB_MUSIC_AMPLIFY.0,
	MusicBPM = BASS_ATTRIB_MUSIC_BPM.0,
	MusicPanSeparation = BASS_ATTRIB_MUSIC_PANSEP.0,
	MusicPositionScaler = BASS_ATTRIB_MUSIC_PSCALER.0,
	MusicSpeed = BASS_ATTRIB_MUSIC_SPEED.0,
	MusicChannelVolume = BASS_ATTRIB_MUSIC_VOL_CHAN.0,
	MusicGlobalVolume = BASS_ATTRIB_MUSIC_VOL_GLOBAL.0,
	MusicInstrumentVolume = BASS_ATTRIB_MUSIC_VOL_INST.0,
	NetworkResume = BASS_ATTRIB_NET_RESUME.0,
	Ramp = BASS_ATTRIB_NORAMP.0,
	Pan = BASS_ATTRIB_PAN.0,
	BufferPushLimit = BASS_ATTRIB_PUSH_LIMIT.0,
	SRCQuality = BASS_ATTRIB_SRC.0,
	Tail = BASS_ATTRIB_TAIL.0,
	Volume = BASS_ATTRIB_VOL.0,
	VolumeDSP = BASS_ATTRIB_VOLDSP.0,
}

impl From<ChannelGetAttribute> for DWORD {
	fn from(value: ChannelGetAttribute) -> Self {
		DWORD(value as u32)
	}
}

#[repr(u32)]
pub enum ChannelSetAttribute {
	Buffer = BASS_ATTRIB_BUFFER.0,
	Frequency = BASS_ATTRIB_FREQ.0,
	Granule = BASS_ATTRIB_GRANULE.0,
	MusicActive = BASS_ATTRIB_MUSIC_ACTIVE.0,
	MusicAmplify = BASS_ATTRIB_MUSIC_AMPLIFY.0,
	MusicBPM = BASS_ATTRIB_MUSIC_BPM.0,
	MusicPanSeparation = BASS_ATTRIB_MUSIC_PANSEP.0,
	MusicPositionScaler = BASS_ATTRIB_MUSIC_PSCALER.0,
	MusicSpeed = BASS_ATTRIB_MUSIC_SPEED.0,
	MusicChannelVolume = BASS_ATTRIB_MUSIC_VOL_CHAN.0,
	MusicGlobalVolume = BASS_ATTRIB_MUSIC_VOL_GLOBAL.0,
	MusicInstrumentVolume = BASS_ATTRIB_MUSIC_VOL_INST.0,
	NetworkResume = BASS_ATTRIB_NET_RESUME.0,
	Ramp = BASS_ATTRIB_NORAMP.0,
	Pan = BASS_ATTRIB_PAN.0,
	BufferPushLimit = BASS_ATTRIB_PUSH_LIMIT.0,
	SRCQuality = BASS_ATTRIB_SRC.0,
	Tail = BASS_ATTRIB_TAIL.0,
	Volume = BASS_ATTRIB_VOL.0,
	VolumeDSP = BASS_ATTRIB_VOLDSP.0,
	VolumeDSPPriority = BASS_ATTRIB_VOLDSP_PRIORITY.0,
}

pub struct BassChannelInfo {
	#[doc = " default playback rate"]
	pub frequency: DWORD,
	#[doc = " channels"]
	pub channels: DWORD,
	pub flags: DWORD,
	#[doc = " type of channel"]
	pub channel_type: DWORD,
	#[doc = " original resolution"]
	pub original_resolution: DWORD,
	pub plugin: HPLUGIN,
	pub sample: HSAMPLE,
	pub filename: Option<String>,
}

impl From<BASS_CHANNELINFO> for BassChannelInfo {
	fn from(value: BASS_CHANNELINFO) -> Self {
		// This is very-much imperfect... but it should get the job done reasonable well.
		let filename = if value.flags & BASS_UNICODE == BASS_UNICODE {
			unsafe { U16CStr::from_ptr_str(value.filename as *const u16) }.to_string().ok()
		} else {
			CString::from(unsafe { CStr::from_ptr(value.filename) }).into_string().ok()
		};
		Self {
			frequency: value.freq,
			channels: value.chans,
			flags: value.flags,
			channel_type: value.ctype,
			original_resolution: value.origres,
			plugin: value.plugin,
			sample: value.sample,
			filename,
		}
	}
}