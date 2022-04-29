use crate::Config;
use crate::types;
use codec::alloc::string::String;
use codec::alloc::string::ToString;
use core::cmp::Ordering;
use core::convert::TryFrom;
use sp_std::prelude::*;

pub trait Hasher: Clone {
	type Hash: Copy + PartialEq + Into<sp_std::vec::Vec<u8>> + TryFrom<sp_std::vec::Vec<u8>>;

	fn hash(data: &[u8]) -> Self::Hash;
}

pub trait MerkelProofValidator {
	fn validate(
		icon_address:types::IconAddress,
		amount:u64,
		defi_user: bool,
		root_hash:types::MerkleHash,
		leaf_hash:types::MerkleHash,
		proofs:types::MerkleProofs)-> bool {
		let computed_leaf= hex::encode(hash_leaf(icon_address,amount,defi_user));
		let input_leaf =hex::encode(leaf_hash);

		if computed_leaf.ne(&input_leaf){
			return false;
		}
		let computed_root= hex::encode(proof_root(leaf_hash,proofs));
		let root_hex = hex::encode(root_hash);

		if computed_root.ne(&root_hex){
			return false;
		}


		return true;
		

	}
}

pub struct AirdropMerkleValidator {}

impl MerkelProofValidator for AirdropMerkleValidator{}



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
	icon_address: types::IconAddress,
	amount: types::ServerBalance,
	defi_user: bool,
) -> [u8; 32] {
	let defi_str: &str = if defi_user { "1" } else { "0" };
	let mut byte_vec = icon_address.to_vec();
	byte_vec.extend_from_slice(amount.to_string().as_bytes());
	byte_vec.extend_from_slice(&defi_str.as_bytes());
	return Blake2bAlgorithm::hash(&byte_vec);
}

pub fn proof_root(leaf_hash: types::MerkleHash, proofs: types::MerkleProofs) -> [u8; 32] {
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

mod tests {

	use super::*;

	#[test]
	fn test_hash_leaf() {
		let expected = "7fe522d63ebcabfa052eec3647366138c23c9870995f4af94d9b22b8c5923f49";
		let icon_addr: [u8; 20] = hex_literal::hex!("a99344ea068864f8af6cbcf89328d6eb3d7e8c9c");
		let result = hex::encode(hash_leaf(icon_addr, 0, true));
		assert_eq!(expected, &result);
	}

	#[test]
	fn test_verify_proof() {
		let root = "0ad37ff10c4e2f80b4f66c376077e664e5333fd6e256385cf7ff2b03952bb2e2";
		let cases = [
			(
				"7fe522d63ebcabfa052eec3647366138c23c9870995f4af94d9b22b8c5923f49",
				[
					"813340daefd7f1ca705faf8318cf6455632259d113c06e97b70eeeccd43519a9",
					"409519ab7129397bdc895e4da05871c9725697a5e092addf2fe90f6e795feb8f",
					"38055bb872670c69ac3461707f8c0b4b8e436eecfc84cfd80db30db3030c489a",
				]
				.to_vec(),
			),
			(
				"c0401b78aed1385426cf3edba8f8b2d25f9fae0f26883fe9b72cf9b4f2d121f0",
				[
					"be77163fe3d25465685bb3d8004b7a8b6a260906b9d4e5fa49427c2f1b789f10",
					"15defb2be27c700d7e3e673652a8dd7609c6d6f84f4dd007880884e0701bcb38",
				]
				.to_vec(),
			),
			(
				"be77163fe3d25465685bb3d8004b7a8b6a260906b9d4e5fa49427c2f1b789f10",
				[
					"c0401b78aed1385426cf3edba8f8b2d25f9fae0f26883fe9b72cf9b4f2d121f0",
					"15defb2be27c700d7e3e673652a8dd7609c6d6f84f4dd007880884e0701bcb38",
				]
				.to_vec(),
			),
			(
				"23ac30dcdf69edb2f243b94608340ba3164424c24f9b5c0f56959b737327b515",
				[
					"4b4bb156d99b6d40e8edfa3c0d50fc4296452276b5cd4f936bceddee37c0505d",
					"d7608226420bd49c0d8e5f79a58c5d693341f2c299911abc1eb96665b85e551d",
					"38055bb872670c69ac3461707f8c0b4b8e436eecfc84cfd80db30db3030c489a",
				]
				.to_vec(),
			),
		];
		for case in cases {
			verify_proof_case(root, case.0, case.1);
		}
	}

	#[test]
	fn test_sort_array() {
		let arr1 = [0u8; 32];
		let arr2 = [0u8; 32];
		let result = sort_array(arr1, arr2, 0 as usize);
		assert_eq!(result, [arr1, arr2].concat());

		let arr1 = [0u8; 32];
		let arr2 = [1u8; 32];
		let result = sort_array(arr1, arr2, 0 as usize);
		assert_eq!(result, [arr1, arr2].concat());

		let arr1 = [2u8; 32];
		let arr2 = [0u8; 32];
		let result = sort_array(arr1, arr2, 0 as usize);
		assert_eq!(result, [arr2, arr1].concat());
	}

	pub fn verify_proof_case(root: &str, leaf: &str, proofs: Vec<&str>) {
		let mut leaf_hash = [0u8; 32];
		hex::decode_to_slice(leaf, &mut leaf_hash as &mut [u8]).unwrap();
		let proofs = proofs
			.into_iter()
			.map(|h| {
				let mut bytes: [u8; 32] = [0u8; 32];
				hex::decode_to_slice(h, &mut bytes as &mut [u8]).unwrap();
				bytes
			})
			.collect::<Vec<[u8; 32]>>();
		let proof_root = proof_root(leaf_hash, proofs);
		assert_eq!(root, hex::encode(proof_root));
	}
}
