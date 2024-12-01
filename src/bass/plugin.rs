use bass_sys::{BASS_PluginFree, DWORD, HPLUGIN};

pub struct Plugin(pub(crate) HPLUGIN);

pub(crate) fn plugin(handle: HPLUGIN) -> Plugin {
	Plugin(handle)
}

impl Drop for Plugin {
    fn drop(&mut self) {
        BASS_PluginFree(self.0);
    }
}

#[derive(Debug, Default)]
pub struct PluginInfo {
	pub version: u32,
	pub formats: Vec<PluginFormat>,
}

#[derive(Debug, Default)]
pub struct PluginFormat {
	/// The channel type, as would appear in the BASS_CHANNELINFO structure.
	pub channel_type: DWORD,
	/// Format description.
	pub name: String,
	/// File extension filter, in the form of "*.ext1;*.ext2;...".
	pub exts: String,
}