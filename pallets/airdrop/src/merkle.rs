use core::convert::TryFrom;
use core::mem;
use core::fmt::{Debug};

use crate::utils;
use codec::alloc::string::String;

use sp_std::vec::Vec;
use sp_runtime::traits::Convert;
use frame_support::pallet_prelude::*;
use sp_std::prelude::*;

#[derive(Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
#[cfg_attr(not(feature = "std"), derive(RuntimeDebug))]
pub enum MerkleTreeError {
	SerializedProofSizeIsIncorrect,
    NotEnoughHelperNodes,
    HashConversionError,
    NotEnoughHashesToCalculateRoot,
    LeavesIndicesCountMismatch,
}





pub trait Hasher: Clone {
    
    type Hash: Copy + PartialEq + Into<sp_std::vec::Vec<u8>> + TryFrom<sp_std::vec::Vec<u8>>;

    fn hash(data: &[u8]) -> Self::Hash;

    fn concat_and_hash(left: &Self::Hash, right: Option<&Self::Hash>) -> Self::Hash {
        let mut concatenated: Vec<u8> = (*left).into();

        match right {
            Some(right_node) => {
                let mut right_node_clone: Vec<u8> = (*right_node).into();
                concatenated.append(&mut right_node_clone);
                Self::hash(&concatenated)
            }
            None => *left,
        }
    }
    fn hash_size() -> usize {
        mem::size_of::<Self::Hash>()
    }
}


type PartialTreeLayer<H> = Vec<(usize, H)>;

#[derive(Clone)]
pub struct PartialTree<T: Hasher> {
    layers: Vec<Vec<(usize, T::Hash)>>,
}

impl<T: Hasher> Default for PartialTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Hasher> PartialTree<T> {
    /// Takes leaves (item hashes) as an argument and build a Merkle Tree from them.
    /// Since it's a partial tree, hashes must be accompanied by their index in the original tree.
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// This is a helper function to build a full tree from a full set of leaves without any
    /// helper indices
    pub fn from_leaves(leaves: &[T::Hash]) -> Result<Self, MerkleTreeError> {
        let leaf_tuples: Vec<(usize, T::Hash)> = leaves.iter().cloned().enumerate().collect();

        Self::build(vec![leaf_tuples], utils::indices::tree_depth(leaves.len()))
    }

    pub fn build(partial_layers: Vec<Vec<(usize, T::Hash)>>, depth: usize) -> Result<Self, MerkleTreeError> {
        let layers = Self::build_tree(partial_layers, depth)?;
        Ok(Self { layers })
    }

    
    fn build_tree(
        mut partial_layers: Vec<Vec<(usize, T::Hash)>>,
        full_tree_depth: usize,
    ) -> Result<Vec<PartialTreeLayer<T::Hash>>, MerkleTreeError> {
        let mut partial_tree: Vec<Vec<(usize, T::Hash)>> = Vec::new();
        let mut current_layer = Vec::new();

        // Reversing helper nodes, so we can remove one layer starting from 0 each iteration
        let mut reversed_layers: Vec<Vec<(usize, T::Hash)>> =
            partial_layers.drain(..).rev().collect();

        // This iterates to full_tree_depth and not to the partial_layers_len because
        // when constructing

        // It is iterating to full_tree_depth instead of partial_layers.len to address the case
        // of applying changes to a tree when tree requires a resize, and partial layer len
        // in that case going to be lower that the resulting tree depth
        for _ in 0..full_tree_depth {
            // Appending helper nodes to the current known nodes
            if let Some(mut nodes) = reversed_layers.pop() {
                current_layer.append(&mut nodes);
            }
            current_layer.sort_by(|(a, _), (b, _)| a.cmp(b));

            // Adding partial layer to the tree
            partial_tree.push(current_layer.clone());

            // This empties `current` layer and prepares it to be reused for the next iteration
            let (indices, nodes): (Vec<usize>, Vec<T::Hash>) = current_layer.drain(..).unzip();
            let parent_layer_indices = utils::indices::parent_indices(&indices);

            for (i, parent_node_index) in parent_layer_indices.iter().enumerate() {
                match nodes.get(i * 2) {
                    // Populate `current_layer` back for the next iteration
                    Some(left_node) => current_layer.push((
                        *parent_node_index,
                        T::concat_and_hash(left_node, nodes.get(i * 2 + 1)),
                    )),
                    None => return Err(MerkleTreeError::NotEnoughHelperNodes),
                }
            }
        }

        partial_tree.push(current_layer.clone());

        Ok(partial_tree)
    }

    /// Returns how many layers there is between leaves and the root
    pub fn depth(&self) -> usize {
        self.layers.len() - 1
    }

    /// Return the root of the tree
    pub fn root(&self) -> Option<&T::Hash> {
        Some(&self.layers.last()?.first()?.1)
    }

    pub fn contains(&self, layer_index: usize, node_index: usize) -> bool {
        match self.layers().get(layer_index) {
            Some(layer) => layer.iter().any(|(index, _)| *index == node_index),
            None => false,
        }
    }

    /// Consumes other partial tree into itself, replacing any conflicting nodes with nodes from
    /// `other` in the process. Doesn't rehash the nodes, so the integrity of the result is
    /// not verified. It gives an advantage in speed, but should be used only if the integrity of
    /// the tree can't be broken, for example, it is used in the `.commit` method of the
    /// `MerkleTree`, since both partial trees are essentially constructed in place and there's
    /// no need to verify integrity of the result.
    pub fn merge_unverified(&mut self, other: Self) {
        // Figure out new tree depth after merge
        let depth_difference = other.layers().len() - self.layers().len();
        let combined_tree_size = if depth_difference > 0 {
            other.layers().len()
        } else {
            self.layers().len()
        };

        for layer_index in 0..combined_tree_size {
            let mut combined_layer: Vec<(usize, T::Hash)> = Vec::new();

            if let Some(self_layer) = self.layers().get(layer_index) {
                let mut filtered_layer: Vec<(usize, T::Hash)> = self_layer
                    .iter()
                    .filter(|(node_index, _)| !other.contains(layer_index, *node_index))
                    .cloned()
                    .collect();

                combined_layer.append(&mut filtered_layer);
            }

            if let Some(other_layer) = other.layers().get(layer_index) {
                let mut cloned_other_layer = other_layer.clone();
                combined_layer.append(&mut cloned_other_layer);
            }

            combined_layer.sort_by(|(a, _), (b, _)| a.cmp(b));
            self.upsert_layer(layer_index, combined_layer);
        }
    }

    /// Replace layer at a given index with a new layer. Used during tree merge
    fn upsert_layer(&mut self, layer_index: usize, mut new_layer: Vec<(usize, T::Hash)>) {
        match self.layers.get_mut(layer_index) {
            Some(layer) => {
                layer.clear();
                layer.append(new_layer.as_mut())
            }
            None => self.layers.push(new_layer),
        }
    }

    pub fn layer_nodes(&self) -> Vec<Vec<T::Hash>> {
        let hashes: Vec<Vec<T::Hash>> = self
            .layers()
            .iter()
            .map(|layer| layer.iter().cloned().map(|(_, hash)| hash).collect())
            .collect();

        hashes
    }

    /// Returns partial tree layers
    pub fn layers(&self) -> &[Vec<(usize, T::Hash)>] {
        &self.layers
    }

    /// Clears all elements in the ree
    pub fn clear(&mut self) {
        self.layers.clear();
    }
}



pub struct MerkleProof<T: Hasher> {
    proof_hashes: Vec<T::Hash>,
}

impl<T: Hasher> MerkleProof<T> {
    pub fn new(proof_hashes: Vec<T::Hash>) -> Self {
        MerkleProof { proof_hashes }
    }

    
    pub fn verify(
        &self,
        root: T::Hash,
        leaf_indices: &[usize],
        leaf_hashes: &[T::Hash],
        total_leaves_count: usize,
    ) -> bool {
        match self.root(leaf_indices, leaf_hashes, total_leaves_count) {
            Ok(extracted_root) => extracted_root == root,
            Err(_) => false,
        }
    }

    
    pub fn root(
        &self,
        leaf_indices: &[usize],
        leaf_hashes: &[T::Hash],
        total_leaves_count: usize,
    ) -> Result<T::Hash, MerkleTreeError> {
        if leaf_indices.len() != leaf_hashes.len() {
            return Err(MerkleTreeError::LeavesIndicesCountMismatch);
        }
        let tree_depth = utils::indices::tree_depth(total_leaves_count);

        // Zipping indices and hashes into a vector of (original_index_in_tree, leaf_hash)
        let mut leaf_tuples: Vec<(usize, T::Hash)> = leaf_indices
            .iter()
            .cloned()
            .zip(leaf_hashes.iter().cloned())
            .collect();
        // Sorting leaves by indexes in case they weren't sorted already
        leaf_tuples.sort_by(|(a, _), (b, _)| a.cmp(b));
        // Getting back _sorted_ indices
        let proof_indices_by_layers =
            utils::indices::proof_indices_by_layers(leaf_indices, total_leaves_count);

        // The next lines copy hashes from proof hashes and group them by layer index
        let mut proof_layers: Vec<Vec<(usize, T::Hash)>> = Vec::with_capacity(tree_depth + 1);
        let mut proof_copy = self.proof_hashes.clone();
        for proof_indices in proof_indices_by_layers {
            let proof_hashes = proof_copy.splice(0..proof_indices.len(), []);
            proof_layers.push(proof_indices.iter().cloned().zip(proof_hashes).collect());
        }

        match proof_layers.first_mut() {
            Some(first_layer) => {
                first_layer.append(&mut leaf_tuples);
                first_layer.sort_by(|(a, _), (b, _)| a.cmp(b));
            }
            None => proof_layers.push(leaf_tuples),
        }

        let partial_tree = PartialTree::<T>::build(proof_layers, tree_depth)?;

        match partial_tree.root() {
            Some(root) => Ok(*root),
            None => Err(MerkleTreeError::NotEnoughHashesToCalculateRoot),
        }
    }

    
    pub fn root_hex(
        &self,
        leaf_indices: &[usize],
        leaf_hashes: &[T::Hash],
        total_leaves_count: usize,
    ) -> Result<String, MerkleTreeError> {
        let root = self.root(leaf_indices, leaf_hashes, total_leaves_count)?;
        Ok(hex::encode(&root.into()))
    }

    
    pub fn proof_hashes(&self) -> &[T::Hash] {
        &self.proof_hashes
    }

    
    pub fn proof_hashes_hex(&self) -> Vec<String> {
        self.proof_hashes
            .iter()
            .map(utils::to_hex_string)
            .collect()
    }

    
  
}


#[derive(Clone)]
pub struct Blake2bAlgorithm {}

impl Hasher for Blake2bAlgorithm {
    type Hash = [u8;32];

    fn hash(data: &[u8]) -> [u8;32] {
        use sp_io::hashing::blake2_256;
        let val= blake2_256(data);
        return val
        
    }

    fn concat_and_hash(left: &Self::Hash, right: Option<&Self::Hash>) -> Self::Hash {
        let mut concatenated: Vec<u8> = (*left).into();

        match right {
            Some(right_node) => {
                let mut right_node_clone: Vec<u8> = (*right_node).into();
                concatenated.append(&mut right_node_clone);
                Self::hash(&concatenated)
            }
            None => *left,
        }
    }

    fn hash_size() -> usize {
        mem::size_of::<Self::Hash>()
    }
}


// #[derive(Clone)]
// pub struct Sha256Algorithm {}

// impl Hasher for Sha256Algorithm{
//     type Hash=[u8;32];

//     fn hash(data: &[u8]) -> Self::Hash {
  //   use sha2::{Digest, Sha256};
//         let mut hasher = Sha256::new();
// 	  hasher.update(data);
// 	  let mut output = [0u8; 32];
// 	  output.copy_from_slice(&hasher.finalize());
// 	  output
//     }
// }





mod tests {

    use super::*;

    // #[test]
    // fn test_tree_sha256(){
    //     let leaf_values = ["a", "b", "c", "d", "e", "f"];
    //     let expected_root_hex = "1f7379539707bcaea00564168d1d4d626b09b73f8a2a365234c62d763f854da2";
    //     let leaf_hashes = leaf_values
    //         .iter()
    //         .map(|x| Sha256Algorithm::hash(x.as_bytes()))
    //         .collect::<Vec<[u8;32]>>();

    //     let tree=PartialTree::<Sha256Algorithm>::from_leaves(&leaf_hashes).unwrap();
    //     let root_val=tree.root().unwrap();
    //     assert_eq!(expected_root_hex,hex::encode(root_val));
    // }

    #[test]
    fn test_tree_blake2b(){
        let leaf_values = ["a", "b", "c", "d", "e", "f"];
        let expected_root_hex = "1b0e542a750f8cbdc5fe4a1b75999a0e9a2caa15a88798dc24ee123e742c2ce1";
        let leaf_hashes = leaf_values
            .iter()
            .map(|x| Blake2bAlgorithm::hash(x.as_bytes()))
            .collect::<Vec<[u8;32]>>();

        let tree=PartialTree::<Blake2bAlgorithm>::from_leaves(&leaf_hashes).unwrap();
        let root_val=tree.root().unwrap();
        assert_eq!(expected_root_hex,hex::encode(root_val));
        let proof_hashes=[
            "00d116515f37a4c0ac872096c8b7412c80693cc5cee2e99e83a7e760dc1ece91",
            "43145816c4f1efa1c8bda6dc342028e63cec088c591dfebac0ef70b4825b3c71",
            "435d24700be0e3213ed5d4ce231438541fb421292ffbec3916e6c9e79eec17e5"
          ].into_iter().map(|h|{
              let mut bytes:[u8;32]=[0u8;32];
            hex::decode_to_slice(h,&mut bytes as &mut [u8]).unwrap();
            bytes
          }).collect::<Vec<[u8;32]>>();

        // verify proof for "c" index 2
       let proof =MerkleProof::<Blake2bAlgorithm>::new(proof_hashes);
       let is_valid= proof.verify(root_val.clone(), &[2], &[leaf_hashes[2]], leaf_hashes.len());
       assert_eq!(is_valid,true);


    }
}

