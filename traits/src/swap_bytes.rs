use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

/// This trait is typically implemented for integers, see [`u32::swap_bytes`]
/// and [`u64::swap_bytes`]. It can be used to store a type in big endian format
/// and take advantage of the fact that storage keys are stored in lexical order.
///
/// Use [`Swapped`] data structure in storage to not mix up value representations.
pub trait SwapBytes: Sized {
	/// Reverses the byte order of the value.
	fn swap_bytes(self) -> Self;
}

impl SwapBytes for u32 {
	fn swap_bytes(self) -> u32 {
		u32::swap_bytes(self)
	}
}

impl SwapBytes for u64 {
	fn swap_bytes(self) -> u64 {
		u64::swap_bytes(self)
	}
}

/// Stores a value with swapped bytes in memory.
/// It will be in big endian format in the storage.
#[derive(Clone, Copy, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct Swapped<T: SwapBytes>(T);

impl<T: SwapBytes> From<T> for Swapped<T> {
	fn from(value: T) -> Self {
		Swapped(value.swap_bytes())
	}
}

impl<T: SwapBytes> Swapped<T> {
	/// Restore non-swapped value.
	pub fn into_value(self) -> T {
		self.0.swap_bytes()
	}
}
