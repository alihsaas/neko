
#[derive(Debug, Clone)]
pub struct InterpreterOptions {
	pub disable_calls: bool,
	pub disable_decleration: bool,
}

impl InterpreterOptions {
	pub fn new() -> Self {
		Self {
			disable_calls: false,
			disable_decleration: false,
		}
	}

	pub fn all() -> Self {
		Self {
			disable_calls: true,
			disable_decleration: true,
		}
	}
}