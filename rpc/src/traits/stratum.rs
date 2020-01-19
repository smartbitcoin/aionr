/*******************************************************************************
 * Copyright (c) 2018-2019 Aion foundation.
 *
 *     This file is part of the aion network project.
 *
 *     The aion network project is free software: you can redistribute it
 *     and/or modify it under the terms of the GNU General Public License
 *     as published by the Free Software Foundation, either version 3 of
 *     the License, or any later version.
 *
 *     The aion network project is distributed in the hope that it will
 *     be useful, but WITHOUT ANY WARRANTY; without even the implied
 *     warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 *     See the GNU General Public License for more details.
 *
 *     You should have received a copy of the GNU General Public License
 *     along with the aion network project source files.
 *     If not, see <https://www.gnu.org/licenses/>.
 *
 ******************************************************************************/

//! Stratum rpc interface.
use jsonrpc_core::Result;
use aion_types::{H256, H512, U256};
use jsonrpc_derive::rpc;

use crate::types::{Work, AddressValidation, Info, MiningInfo, MinerStats, TemplateParam, StratumHeader, BlockNumber};

/// Stratum rpc interface.
#[rpc(server)]
pub trait Stratum {
    type Metadata;

    /// Returns the work of current block
    #[rpc(name = "getblocktemplate")]
    fn work(&self, _tpl_param: Option<TemplateParam>) -> Result<Work>;

    /// Submit a proof-of-work solution
    #[rpc(name = "submitblock")]
    fn submit_work(
        &self,
        nonce_str: String,
        solution_str: String,
        pow_hash_str: String,
    ) -> Result<bool>;

    /// Get information
    #[rpc(name = "getinfo")]
    fn get_info(&self) -> Result<Info>;

    /// Check if address is valid
    #[rpc(name = "validateaddress")]
    fn validate_address(&self, address: H256) -> Result<AddressValidation>;

    /// Get difficulty
    #[rpc(name = "getdifficulty")]
    fn get_difficulty(&self) -> Result<U256>;

    /// Get mining information
    #[rpc(name = "getmininginfo")]
    fn get_mining_info(&self) -> Result<MiningInfo>;

    /// Get miner stats
    #[rpc(name = "getMinerStats")]
    fn get_miner_stats(&self, address: H256) -> Result<MinerStats>;

    /// Get block header by number
    #[rpc(name = "getHeaderByBlockNumber")]
    fn get_block_by_number(&self, num: BlockNumber) -> Result<StratumHeader>;

    /// return [u8; 96] seed of current block
    #[rpc(name = "getseed")]
    fn pos_get_seed(&self) -> Result<H512>;

    /// Seed: seed of block (n+1) generated by stratum client
    /// public key: signing public key
    /// coinbase: address for claiming the block rewards
    /// return: hash of block (n+1)
    #[rpc(name = "submitseed")]
    fn pos_submit_seed(&self, seed: H512, psk: H256, coinbase: H256) -> Result<H256>;

    /// Signature: signature of staker
    /// Hash: hash of block (n+1)
    /// return: if work submitted in success
    #[rpc(name = "submitsignature")]
    fn pos_submit_work(&self, sig: H512, hash: H256) -> Result<bool>;
}
