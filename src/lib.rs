pub mod bass;
pub mod channel;
pub mod dsp;
pub mod functions;
pub mod flags;
pub mod fx;
#[cfg(feature = "mixer")]
pub mod mixer;
pub mod music;
pub mod recording;
pub mod sample;
#[cfg(feature = "mixer")]
pub mod split;
pub mod stream;
pub mod sync;
pub mod types;
pub use types::*;

#[cfg(test)]
mod tests {
	use std::{
		error::Error, f32::consts::PI, sync::{
			mpsc::{self, Receiver, Sender},
			Arc,
		}, thread, time::Duration
	};

	use bass_sys::{
		BASS_ATTRIB_FREQ, BASS_ATTRIB_VOL, BASS_LEVEL_MONO, BASS_LEVEL_STEREO, BASS_SAMPLE_FLOAT, BASS_SLIDE_LOG,
		BASS_SYNC_SLIDE, BASS_SYNC_THREAD, DWORD, HDSP, HSYNC,
	};

	use crate::{
		bass::{error::BassErrorCode, Bass},
		channel::Channel,
		functions::make_word,
		stream::Stream,
		BassResult,
	};

	fn signal_process<T>(_: &mut T, data: &mut [f32], dsp: HDSP, channel: DWORD) {
		println!("data props: len: {}", data.len());
		println!("HDSP: {:?}", dsp);
	}

	// static mut rotpos: f32 = 0.;

	#[test]
	fn test_helpers() {
		assert_eq!(make_word(0b110000000000001111), (0b11, 0b1111));
	}

	struct TestStruct;

	impl TestStruct {
		pub fn new() -> Arc<Self> {
			Arc::new(TestStruct)
		}
		pub fn handle_sync(self: &mut Arc<Self>, sync: HSYNC, channel: DWORD, data: DWORD) {
			self.private_test_func();
		}
		fn private_test_func(&self) -> u32 {
			println!("Sync proc in struct impl!");
			2
		}
	}

	#[test]
	/// A test of the following:
	///
	/// - DSP (manual panning specifically)
	/// - Syncs (passing user data, being run multiple times, etc.)
	/// - Sliding of attributes
	/// - Sleeping/waiting for a sync
	///
	/// Best run with `cargo test -- --nocapture` to see output
	fn test() -> Result<(), Box<dyn Error>> {
		let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

		// let barrier = Arc::new(Barrier::new(2));
		// let barrier2 = barrier.clone();

		let test = TestStruct::new();

		let sync_handler = |user: &mut Sender<i32>, sync: HSYNC, channel: DWORD, data: DWORD| {
			// println!("User Data: {user}");
			println!("In sync proc!");
			if data == BASS_ATTRIB_VOL {
				println!("Slid volume")
			} else {
				println!("Frequency changed")
			}
			user.send(8);
			// barrier2.wait();
		};

		let mut rotpos: f32 = 0.;

		let dsp_pan = move |_: &mut Option<u8>, data: &mut [f32], dsp: HDSP, channel: DWORD| {
			// float *d = (float*)buffer;
			// DWORD a;
			// for (a = 0; a < length / 4; a += 2) {
			// 	d[a] *= fabs(sin(rotpos));
			// 	d[a + 1] *= fabs(cos(rotpos));
			// 	rotpos += 0.00003;
			// }
			// rotpos = fmod(rotpos, 2 * M_PI);
			// println!("data props: len: {}", data.len());
			// println!("HDSP: {:?}", dsp);

			let mut is_even = true;

			for sample in data {
				if is_even {
					*sample = sample.clone() * (rotpos.sin().abs());
					is_even = false;
				} else {
					*sample = sample.clone() * (rotpos.cos().abs());
					is_even = true;
					rotpos = rotpos + 0.00003;
				}
			}

			rotpos = rotpos % (2. * PI)
		};

		let bass = Bass::init(-1, 48000, None)?;
		let device = bass.device();
		println!("BASS Device: {device}");
		let devices = Bass::devices();
		println!("BASS Device: {devices:#?}");
		let stream =
			Stream::create_file("./orchestra-tune-up.mp3", 0, 0, BASS_SAMPLE_FLOAT)?;
		let _sync = stream.set_sync(
			BASS_SYNC_SLIDE | BASS_SYNC_THREAD,
			stream.seconds_to_bytes(3.)?,
			sync_handler,
			tx.clone(),
		)?;
		let _sync2 = stream.set_sync(BASS_SYNC_SLIDE, 0, TestStruct::handle_sync, test.clone());
		if stream.represents_handle(0) {
			panic!() // This should never happen
		}
		let dsp = stream.set_dsp(0, None::<u8>, dsp_pan)?;
		stream.play(false)?;

		assert_eq!(stream.get_level_ex(1., Some(BASS_LEVEL_MONO)).expect("Levels").len(), 1);
		assert_eq!(stream.get_level_ex(1., Some(BASS_LEVEL_STEREO)).expect("Levels").len(), 2);

		stream.slide_attribute(BASS_ATTRIB_FREQ | BASS_SLIDE_LOG, 4800., 5000)?;
		stream.slide_attribute(BASS_ATTRIB_VOL, 0.5, 2000)?;

		let ok = rx.recv_timeout(Duration::from_secs(12));
		match ok {
			Ok(ok) => println!("Successful! {ok}"),
			Err(e) => eprintln!("{}", e),
		}

		stream.slide_attribute(BASS_ATTRIB_VOL, 1., 2000)?;
		println!("{:#?}", stream.get_level());

		let ok = rx.recv_timeout(Duration::from_secs(12));
		match ok {
			Ok(ok) => println!("Successful! {ok}"),
			Err(e) => eprintln!("{}", e),
		}

		thread::sleep(Duration::from_secs(3));

		stream.slide_attribute(BASS_ATTRIB_FREQ | BASS_SLIDE_LOG, 48000., 5000)?;
		println!("{:#?}", stream.get_level_ex(1., None));

		// match rx.recv() {
		// 	Ok(ok) => println!("Successful! {ok}"),
		// 	Err(e) => eprintln!("{}", e),
		// }

		thread::sleep(Duration::from_secs(10));
		ok.map(|_| ())?;
		Ok(())
	}
}
