// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Handling epoch changes in BABE.
//!
//! This exposes the `SharedEpochChanges`, which is a wrapper around a
//! persistent DAG superimposed over the forks of the blockchain.

use std::sync::Arc;
use babe_primitives::{Epoch, SlotNumber};
use fork_tree::ForkTree;
use parking_lot::{Mutex, MutexGuard};
use sr_primitives::traits::{Block as BlockT, NumberFor};
use codec::{Encode, Decode};
use client::error::Error as ClientError;

/// Tree of all epoch changes across all *seen* forks. Data stored in tree is
/// the hash and block number of the block signaling the epoch change, and the
/// epoch that was signalled at that block.
#[derive(Clone, Encode, Decode)]
pub struct EpochChanges<Block: BlockT> {
	inner: ForkTree<Block::Hash, NumberFor<Block>, Epoch>,
}

impl<Block: BlockT> EpochChanges<Block> {
	/// Create a new epoch-change tracker.
	fn new() -> Self {
		EpochChanges { inner: ForkTree::new() }
	}

	/// Prune out finalized epochs, except for the ancestor of the finalized block.
	pub fn prune_finalized(&mut self, hash: &Block::Hash, number: NumberFor<Block>) {
		// TODO: needs "is-descendent-of"
		unimplemented!()
	}

	/// Finds the epoch for a child of the given block, assuming the given slot number.
	pub fn epoch_for_child_of(
		&mut self,
		parent_hash: &Block::Hash,
		parent_number: NumberFor<Block>,
		slot_number: SlotNumber,
	) -> Option<Epoch> {
		use sr_primitives::traits::One;

		// find_node_where will give you the node in the fork-tree which is an ancestor
		// of the `parent_hash` by default. if the last epoch was signalled at the parent_hash,
		// then it won't be returned. we need to create a new fake chain head hash which
		// "descends" from our parent-hash.
		let fake_head_hash = {
			let mut h = parent_hash.clone();
			// dirty trick: flip the first bit of the parent hash to create a hash
			// which has not been in the chain before (assuming a strong hash function).
			h.as_mut()[0] ^= 0b10000000;
			h
		};

		// TODO: let is_descendent_of = is_descendent_of(client, Some((&fake_head_hash, &parent_hash)));
		let is_descendent_of = |a: &_, b: &_| { Ok(unimplemented!()) };
		self.inner.find_node_where::<_, ClientError, _>(
			&fake_head_hash,
			&(parent_number + One::one()),
			&is_descendent_of,
			&|epoch| epoch.start_slot <= slot_number,
		)
			.ok()
			.and_then(|n| n)
			.map(|n| n.data.clone())
	}

	/// Import a new epoch-change, signalled at the given block.
	pub fn import(
		&mut self,
		hash: Block::Hash,
		number: NumberFor<Block>,
		epoch: Epoch,
	) {
		unimplemented!()
	}
}

/// A shared epoch changes tree.
#[derive(Clone)]
pub struct SharedEpochChanges<Block: BlockT> {
	inner: Arc<Mutex<EpochChanges<Block>>>,
}

impl<Block: BlockT> SharedEpochChanges<Block> {
	/// Create a new instance of the `SharedEpochChanges`.
	pub fn new() -> Self {
		SharedEpochChanges {
			inner: Arc::new(Mutex::new(EpochChanges::<Block>::new()))
		}
	}

	/// Lock the shared epoch changes,
	pub fn lock(&self) -> MutexGuard<EpochChanges<Block>> {
		self.inner.lock()
	}
}

impl<Block: BlockT> From<EpochChanges<Block>> for SharedEpochChanges<Block> {
	fn from(epoch_changes: EpochChanges<Block>) -> Self {
		SharedEpochChanges {
			inner: Arc::new(Mutex::new(epoch_changes))
		}
	}
}