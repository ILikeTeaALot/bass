pub mod bass;
mod channel;
pub mod enums;
pub mod error;
pub mod fx;
pub mod mixer;
/// Not ready for public yet
mod music;
pub mod null;
pub mod result;
pub mod sample;
pub mod stream;
pub mod types;
pub(crate) mod syncproc;
mod split;

pub use bass_sys;

// mod quick_test {
// 	use bass_sys::HSTREAM;

// 	// fn test_union(value: impl AsDWORD) { }

// 	// fn test_upper() {
// 	// 	let x: HSTREAM = HSTREAM(4);
// 	// 	test_union(x);
// 	// }
// }
