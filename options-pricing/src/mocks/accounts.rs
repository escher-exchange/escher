use sp_core::sr25519::{Public, Signature};
use sp_runtime::traits::{IdentifyAccount, Verify};

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

pub const ADMIN: AccountId = account_id_from_u64(0);
pub const ALICE: AccountId = account_id_from_u64(1);

pub const fn account_id_from_u64(n: u64) -> AccountId {
	let bytes_src = n.to_be_bytes();
	let mut bytes_dst = [0u8; 32];
	let mut k = 0;
	while k < bytes_src.len() {
		bytes_dst[k + 24] = bytes_src[k];
		k += 1;
	}
	Public(bytes_dst)
}
