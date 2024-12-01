use bass_sys::{DWORD, WORD};

pub fn make_word(input: impl Into<DWORD>) -> (u16, u16) {
	let value = input.into().0;
	((value as u32 >> 16) as u16, value as u16)
}

pub fn make_long(hi: impl Into<WORD>, lo: impl Into<WORD>) -> DWORD {
	DWORD(((hi.into().0 as u32) << 16) | (lo.into().0 as u32))
}