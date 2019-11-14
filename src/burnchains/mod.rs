/*
 copyright: (c) 2013-2018 by Blockstack PBC, a public benefit corporation.

 This file is part of Blockstack.

 Blockstack is free software. You may redistribute or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License or
 (at your option) any later version.

 Blockstack is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY, including without the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with Blockstack. If not, see <http://www.gnu.org/licenses/>.
*/

/// This module contains drivers and types for all burn chains we support.

pub mod bitcoin;
pub mod indexer;
pub mod burnchain;

use std::fmt;
use std::error;
use std::io;

use self::bitcoin::Error as btc_error;

use chainstate::burn::operations::Error as op_error;
use chainstate::burn::ConsensusHash;

use util::hash::Hash160;
use util::db::Error as db_error;

#[derive(Serialize, Deserialize)]
pub struct Txid([u8; 32]);
impl_array_newtype!(Txid, u8, 32);
impl_array_hexstring_fmt!(Txid);
impl_byte_array_newtype!(Txid, u8, 32);
pub const TXID_ENCODED_SIZE : u32 = 32;

#[derive(Serialize, Deserialize)]
pub struct BurnchainHeaderHash([u8; 32]);
impl_array_newtype!(BurnchainHeaderHash, u8, 32);
impl_array_hexstring_fmt!(BurnchainHeaderHash);
impl_byte_array_newtype!(BurnchainHeaderHash, u8, 32);
pub const BURNCHAIN_HEADER_HASH_ENCODED_SIZE : u32 = 32;

pub const MAGIC_BYTES_LENGTH: usize = 2;

#[derive(Debug, Serialize, Deserialize)]
pub struct MagicBytes([u8; MAGIC_BYTES_LENGTH]);
impl_array_newtype!(MagicBytes, u8, MAGIC_BYTES_LENGTH);

pub const BLOCKSTACK_MAGIC_MAINNET : MagicBytes = MagicBytes([105, 100]);  // 'id'

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BurnQuotaConfig {
    pub inc: u64,
    pub dec_num: u64,
    pub dec_den: u64
}

#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
pub enum BurnchainInputType {
    BitcoinInput,
    BitcoinSegwitP2SHInput,

    // TODO: expand this as more burnchains are supported
}

#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
pub enum StableConfirmations {
    Bitcoin = 7

    // TODO: expand this as more burnchains are supported
}

#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
pub enum ConsensusHashLifetime {
    Bitcoin = 24

    // TODO: expand this as more burnchains are supported
}

pub trait PublicKey : Clone + fmt::Debug + serde::Serialize + serde::de::DeserializeOwned {
    fn to_bytes(&self) -> Vec<u8>;
    fn verify(&self, data_hash: &[u8], sig: &[u8]) -> Result<bool, &'static str>;
}

pub trait PrivateKey : Clone + fmt::Debug + serde::Serialize + serde::de::DeserializeOwned {
    fn to_bytes(&self) -> Vec<u8>;
    fn sign(&self, data_hash: &[u8]) -> Result<Vec<u8>, &'static str>;
}

pub trait Address : Clone + fmt::Debug {
    fn to_bytes(&self) -> Vec<u8>;
    fn to_string(&self) -> String;
    fn from_string(&String) -> Option<Self>
        where Self: Sized;
    fn burn_bytes() -> Vec<u8>;
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BurnchainTxOutput<A> {
    pub address: A,
    pub units: u64
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BurnchainTxInput<K> {
    pub keys: Vec<K>,
    pub num_required: usize,
    pub in_type: BurnchainInputType
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BurnchainTransaction<A, K> {
    pub txid: Txid,
    pub vtxindex: u32,
    pub opcode: u8,
    pub data: Vec<u8>,
    pub inputs: Vec<BurnchainTxInput<K>>,
    pub outputs: Vec<BurnchainTxOutput<A>>
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BurnchainBlock<A, K> {
    pub block_height: u64,
    pub block_hash: BurnchainHeaderHash,
    pub parent_block_hash: BurnchainHeaderHash,
    pub txs: Vec<BurnchainTransaction<A, K>>
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Burnchain {
    pub peer_version: u32,
    pub network_id: u32,
    pub chain_name: String,
    pub network_name: String,
    pub working_dir: String,
    pub burn_quota : BurnQuotaConfig,
    pub consensus_hash_lifetime: u32,
    pub stable_confirmations: u32,
    pub first_block_height: u64,
    pub first_block_hash: BurnchainHeaderHash
}

/// Structure for encoding our view of the network 
#[derive(Debug, PartialEq, Clone)]
pub struct BurnchainView {
    pub burn_block_height: u64,                     // last-seen block height (at chain tip)
    pub burn_consensus_hash: ConsensusHash,         // consensus hash at block_height
    pub burn_stable_block_height: u64,              // latest stable block height (e.g. chain tip minus 7)
    pub burn_stable_consensus_hash: ConsensusHash,  // consensus hash for burn_stable_block_height
}

#[derive(Debug)]
pub enum Error {
    /// Unsupported burn chain
    UnsupportedBurnchain,
    /// Bitcoin-related error
    Bitcoin(btc_error),
    /// burn database error 
    DBError(db_error),
    /// Download error 
    DownloadError(btc_error),
    /// Parse error 
    ParseError,
    /// Thread channel error 
    ThreadChannelError,
    /// Missing headers 
    MissingHeaders,
    /// filesystem error 
    FSError(io::Error),
    /// Operation processing error 
    OpError(op_error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UnsupportedBurnchain => f.write_str(error::Error::description(self)),
            Error::Bitcoin(ref btce) => fmt::Display::fmt(btce, f),
            Error::DBError(ref dbe) => fmt::Display::fmt(dbe, f),
            Error::DownloadError(ref btce) => fmt::Display::fmt(btce, f),
            Error::ParseError => f.write_str(error::Error::description(self)),
            Error::MissingHeaders => f.write_str(error::Error::description(self)),
            Error::ThreadChannelError => f.write_str(error::Error::description(self)),
            Error::FSError(ref e) => fmt::Display::fmt(e, f),
            Error::OpError(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::UnsupportedBurnchain => None,
            Error::Bitcoin(ref e) => Some(e),
            Error::DBError(ref e) => Some(e),
            Error::DownloadError(ref e) => Some(e),
            Error::ParseError => None,
            Error::MissingHeaders => None,
            Error::ThreadChannelError => None,
            Error::FSError(ref e) => Some(e),
            Error::OpError(ref e) => Some(e),
        }
    }

    fn description(&self) -> &str {
        match *self {
            Error::UnsupportedBurnchain => "Unsupported burnchain",
            Error::Bitcoin(ref e) => e.description(),
            Error::DBError(ref e) => e.description(),
            Error::DownloadError(ref e) => e.description(),
            Error::ParseError => "Parse error",
            Error::MissingHeaders => "Missing block headers",
            Error::ThreadChannelError => "Error in thread channel",
            Error::FSError(ref e) => e.description(),
            Error::OpError(ref e) => e.description(),
        }
    }
}