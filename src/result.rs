// use bass_sys::BOOL;

use crate::error::BassError;

pub fn result(ok: bool) -> Result<(), BassError> {
	if ok {
		Ok(())
	} else {
		Err(BassError::default())
	}
    // match ok {
    //     BOOL::FALSE => Err(BassError::default()),
    //     BOOL::TRUE => Ok(()),
    // }
}

pub type BassResult<T> = Result<T, BassError>;