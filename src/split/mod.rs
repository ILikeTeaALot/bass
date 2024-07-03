use std::sync::{Arc, Weak};

pub struct SplitStream {
	weak_self: Weak<SplitStream>,
}

impl SplitStream {
	pub fn new() -> Arc<Self> {
		Arc::new_cyclic(|split| {
			Self {
				weak_self: split.clone(),
			}
		})
	}
}