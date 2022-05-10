use crate::types;
use crate::Config;
use codec::alloc::string::ToString;
use core::cmp::Ordering;
use core::convert::TryFrom;
use core::marker::PhantomData;
use sp_std::prelude::*;
use types::MerkelProofValidator;

pub trait Hasher: Clone {
	type Hash: Copy + PartialEq + Into<sp_std::vec::Vec<u8>> + TryFrom<sp_std::vec::Vec<u8>>;

	fn hash(data: &[u8]) -> Self::Hash;
}

pub struct AirdropMerkleValidator<T>(PhantomData<T>);

impl<T: Config> MerkelProofValidator<T> for AirdropMerkleValidator<T> {
	fn validate(
		icon_address: &types::IconAddress,
		amount: u64,
		defi_user: bool,
		root_hash: types::MerkleHash,
		leaf_hash: types::MerkleHash,
		proofs: types::MerkleProofs<T>,
	) -> bool {
		let computed_leaf = hex::encode(hash_leaf(icon_address, amount, defi_user));
		let input_leaf = hex::encode(leaf_hash);

		if computed_leaf.ne(&input_leaf) {
			return false;
		}
		let computed_root = hex::encode(proof_root(leaf_hash, proofs.to_vec()));
		let root_hex = hex::encode(root_hash);

		if computed_root.ne(&root_hex) {
			return false;
		}

		return true;
	}
}

#[derive(Clone)]
pub struct Blake2bAlgorithm {}

impl Hasher for Blake2bAlgorithm {
	type Hash = [u8; 32];

	fn hash(data: &[u8]) -> [u8; 32] {
		use sp_io::hashing::blake2_256;
		let val = blake2_256(data);
		return val;
	}
}

pub fn hash_leaf(
	icon_address: &types::IconAddress,
	amount: types::ServerBalance,
	defi_user: bool,
) -> [u8; 32] {
	let defi_str: &str = if defi_user { "1" } else { "0" };
	let mut byte_vec = icon_address.to_vec();
	byte_vec.extend_from_slice(amount.to_string().as_bytes());
	byte_vec.extend_from_slice(&defi_str.as_bytes());
	return Blake2bAlgorithm::hash(&byte_vec);
}

pub fn proof_root(leaf_hash: types::MerkleHash, proofs: Vec<types::MerkleHash>) -> [u8; 32] {
	let mut one = leaf_hash;
	for proof in proofs {
		one = create_hash(one, proof);
		let one_hex = hex::encode(one);
		log::info!("Calculated: {:?}", one_hex);
	}
	return one;
}

pub fn create_hash(one: types::MerkleHash, other: types::MerkleHash) -> [u8; 32] {
	let sorted = sort_array(one, other, 0 as usize);
	Blake2bAlgorithm::hash(&sorted)
}

pub fn sort_array(one: types::MerkleHash, other: types::MerkleHash, pos: usize) -> Vec<u8> {
	let max_pos = 31 as usize;
	let mut pos = pos;
	let ord = one[pos].cmp(&other[pos]);
	match ord {
		Ordering::Greater => [other, one].concat(),
		Ordering::Less => [one, other].concat(),
		Ordering::Equal => {
			if pos == max_pos {
				return [one, other].concat();
			}
			pos = pos + 1;
			sort_array(one, other, pos)
		}
	}
}
