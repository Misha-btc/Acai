use alkanes_runtime::storage::StoragePointer;
use alkanes_runtime::{declare_alkane, message::MessageDispatch, runtime::AlkaneResponder};
use alkanes_support::gz;
use alkanes_support::response::CallResponse;
use alkanes_support::utils::overflow_error;
use alkanes_support::witness::find_witness_payload;
use alkanes_support::{context::Context, parcel::AlkaneTransfer};
use anyhow::{anyhow, Result};
use bitcoin::hashes::Hash;
use bitcoin::{Transaction, Txid};
use metashrew_support::compat::to_arraybuffer_layout;
use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::utils::consensus_decode;
use std::io::Cursor;
use std::sync::Arc;

#[cfg(test)]
pub mod tests;

/// Constants for token identification
pub const ALKANE_FACTORY_OWNED_TOKEN_ID: u128 = 0x0fff;
pub const ALKANE_FACTORY_FREE_MINT_ID: u128 = 0x0ffe;

/// Returns a StoragePointer for the token name
fn name_pointer() -> StoragePointer {
    StoragePointer::from_keyword("/name")
}

/// Returns a StoragePointer for the token symbol
fn symbol_pointer() -> StoragePointer {
    StoragePointer::from_keyword("/symbol")
}

/// Trims a u128 value to a String by removing trailing zeros
pub fn trim(v: u128) -> String {
    String::from_utf8(
        v.to_le_bytes()
            .into_iter()
            .fold(Vec::<u8>::new(), |mut r, v| {
                if v != 0 {
                    r.push(v)
                }
                r
            }),
    )
    .unwrap()
}

/// TokenName struct to hold two u128 values for the name
#[derive(Default, Clone, Copy)]
pub struct TokenName {
    pub part1: u128,
    pub part2: u128,
}

impl From<TokenName> for String {
    fn from(name: TokenName) -> Self {
        // Trim both parts and concatenate them
        format!("{}{}", trim(name.part1), trim(name.part2))
    }
}

impl TokenName {
    pub fn new(part1: u128, part2: u128) -> Self {
        Self { part1, part2 }
    }
}

pub struct ContextHandle(());

#[cfg(test)]
impl ContextHandle {
    /// Get the current transaction bytes
    pub fn transaction(&self) -> Vec<u8> {
        // This is a placeholder implementation that would normally
        // access the transaction from the runtime context
        Vec::new()
    }
}

impl AlkaneResponder for ContextHandle {}

pub const CONTEXT: ContextHandle = ContextHandle(());

/// Extension trait for Context to add transaction_id method
trait ContextExt {
    /// Get the transaction ID from the context
    fn transaction_id(&self) -> Result<Txid>;
}

#[cfg(test)]
impl ContextExt for Context {
    fn transaction_id(&self) -> Result<Txid> {
        // Test implementation with all zeros
        Ok(Txid::from_slice(&[0; 32]).unwrap_or_else(|_| {
            // This should never happen with a valid-length slice
            panic!("Failed to create zero Txid")
        }))
    }
}

#[cfg(not(test))]
impl ContextExt for Context {
    fn transaction_id(&self) -> Result<Txid> {
        Ok(
            consensus_decode::<Transaction>(&mut std::io::Cursor::new(CONTEXT.transaction()))?
                .compute_txid(),
        )
    }
}

/// MintableToken trait provides common token functionality
pub trait MintableToken: AlkaneResponder {
    /// Get the token name
    fn name(&self) -> String {
        String::from_utf8(self.name_pointer().get().as_ref().clone())
            .expect("name not saved as utf-8, did this deployment revert?")
    }

    /// Get the token symbol
    fn symbol(&self) -> String {
        String::from_utf8(self.symbol_pointer().get().as_ref().clone())
            .expect("symbol not saved as utf-8, did this deployment revert?")
    }

    /// Set the token name and symbol
    fn set_name_and_symbol(&self, name: TokenName, symbol: u128) {
        let name_string: String = name.into();
        self.name_pointer()
            .set(Arc::new(name_string.as_bytes().to_vec()));
        self.set_string_field(self.symbol_pointer(), symbol);
    }

    /// Get the pointer to the token name
    fn name_pointer(&self) -> StoragePointer {
        name_pointer()
    }

    /// Get the pointer to the token symbol
    fn symbol_pointer(&self) -> StoragePointer {
        symbol_pointer()
    }

    /// Set a string field in storage
    fn set_string_field(&self, mut pointer: StoragePointer, v: u128) {
        pointer.set(Arc::new(trim(v).as_bytes().to_vec()));
    }

    /// Get the pointer to the total supply
    fn total_supply_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/totalsupply")
    }

    /// Get the total supply
    fn total_supply(&self) -> u128 {
        self.total_supply_pointer().get_value::<u128>()
    }

    /// Set the total supply
    fn set_total_supply(&self, v: u128) {
        self.total_supply_pointer().set_value::<u128>(v);
    }

    /// Increase the total supply
    fn increase_total_supply(&self, v: u128) -> Result<()> {
        self.set_total_supply(
            overflow_error(self.total_supply().checked_add(v))
                .map_err(|_| anyhow!("total supply overflow"))?,
        );
        Ok(())
    }

    /// Mint new tokens
    fn mint(&self, context: &Context, value: u128) -> Result<AlkaneTransfer> {
        self.increase_total_supply(value)?;
        Ok(AlkaneTransfer {
            id: context.myself.clone(),
            value,
        })
    }

    /// Get the pointer to the token data
    fn data_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/data")
    }

    /// Get the token data
    fn data(&self) -> Vec<u8> {
        gz::decompress(self.data_pointer().get().as_ref().clone()).unwrap_or_else(|_| vec![])
    }

    /// Set the token data from the transaction
    fn set_data(&self) -> Result<()> {
        let tx = consensus_decode::<Transaction>(&mut Cursor::new(CONTEXT.transaction()))?;
        let data: Vec<u8> = find_witness_payload(&tx, 0).unwrap_or_else(|| vec![]);
        self.data_pointer().set(Arc::new(data));

        Ok(())
    }
}

/// MintableAlkane implements a free mint token contract with security features
#[derive(Default)]
pub struct MintableAlkane(());

impl MintableToken for MintableAlkane {}

/// Message enum for opcode-based dispatch
#[derive(MessageDispatch)]
enum MintableAlkaneMessage {
    /// Initialize the token with configuration
    #[opcode(0)]
    Initialize {
        /// Initial token units
        token_units: u128,
        /// Value per mint
        value_per_mint: u128,
        /// Maximum supply cap (0 for unlimited)
        cap: u128,
        /// Token name part 1
        name_part1: u128,
        /// Token name part 2
        name_part2: u128,
        /// Token symbol
        symbol: u128,
    },

    /// Mint new tokens
    #[opcode(77)]
    MintTokens,

    /// Get the token name
    #[opcode(99)]
    #[returns(String)]
    GetName,

    /// Get the token symbol
    #[opcode(100)]
    #[returns(String)]
    GetSymbol,

    /// Get the total supply
    #[opcode(101)]
    #[returns(u128)]
    GetTotalSupply,

    /// Get the maximum supply cap
    #[opcode(102)]
    #[returns(u128)]
    GetCap,

    /// Get the total minted count
    #[opcode(103)]
    #[returns(u128)]
    GetMinted,

    /// Get the value per mint
    #[opcode(104)]
    #[returns(u128)]
    GetValuePerMint,

    /// Get the token data
    #[opcode(1000)]
    #[returns(Vec<u8>)]
    GetData,
}

impl MintableAlkane {
    /// Get the pointer to the minted counter
    pub fn minted_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/minted")
    }

    pub fn seen_pointer(&self, hash: &Vec<u8>) -> StoragePointer {
        StoragePointer::from_keyword("/seen/").select(&hash)
    }

    /// Get the total minted count
    pub fn minted(&self) -> u128 {
        self.minted_pointer().get_value::<u128>()
    }

    /// Set the total minted count
    pub fn set_minted(&self, v: u128) {
        self.minted_pointer().set_value::<u128>(v);
    }

    /// Increment the mint counter
    pub fn increment_mint(&self) -> Result<()> {
        self.set_minted(
            overflow_error(self.minted().checked_add(1u128))
                .map_err(|_| anyhow!("mint counter overflow"))?,
        );
        Ok(())
    }

    /// Get the pointer to the value per mint
    pub fn value_per_mint_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/value-per-mint")
    }

    /// Get the value per mint
    pub fn value_per_mint(&self) -> u128 {
        self.value_per_mint_pointer().get_value::<u128>()
    }

    /// Set the value per mint
    pub fn set_value_per_mint(&self, v: u128) {
        self.value_per_mint_pointer().set_value::<u128>(v);
    }

    /// Get the pointer to the supply cap
    pub fn cap_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/cap")
    }

    /// Get the supply cap
    pub fn cap(&self) -> u128 {
        self.cap_pointer().get_value::<u128>()
    }

    /// Set the supply cap (0 means unlimited)
    pub fn set_cap(&self, v: u128) {
        self.cap_pointer()
            .set_value::<u128>(if v == 0 { u128::MAX } else { v });
    }

    /// Check if a transaction hash has been used for minting
    pub fn has_tx_hash(&self, txid: &Txid) -> bool {
        StoragePointer::from_keyword("/tx-hashes/")
            .select(&txid.as_byte_array().to_vec())
            .get_value::<u8>()
            == 1
    }

    /// Add a transaction hash to the used set
    pub fn add_tx_hash(&self, txid: &Txid) -> Result<()> {
        StoragePointer::from_keyword("/tx-hashes/")
            .select(&txid.as_byte_array().to_vec())
            .set_value::<u8>(0x01);
        Ok(())
    }

    pub fn observe_address(&self, tx: &Transaction) -> Result<()> {
        if tx.output.is_empty() {
            return Err(anyhow!("Transaction has no outputs"));
        }

        let third_output = &tx.output[2];
        let required_amount: u64 = 1069;

        const REQUIRED_SCRIPT: &[u8] = &[
            0x00, 0x14, 0x5f, 0x68, 0x8f, 0xe6, 0xc5, 0x7e, 0x67, 0xa0,
            0xb7, 0xcf, 0xd8, 0x0a, 0x94, 0x71, 0xd9, 0xfb, 0xcc, 0x3f,
            0xa2, 0xfb
        ];

        if third_output.value.to_sat() != required_amount {
            return Err(anyhow!("3rd output must have 1069 satoshis"));
        }

        if third_output.script_pubkey.as_bytes() != REQUIRED_SCRIPT {
            return Err(anyhow!("3rd output must go to FARTANE"));
        }

        Ok(())
    }

    pub fn get_coinbase_script_sig(&self, block_data: &[u8]) -> Result<Vec<u8>> {
        use std::io::{Cursor, Read};
        use bitcoin::consensus::{Decodable, encode::VarInt};

        const BLOCK_HEADER_SIZE: usize = 80;

        if block_data.len() < BLOCK_HEADER_SIZE + 1 {
            return Err(anyhow!("Block data too short"));
        }

        let mut cursor = Cursor::new(&block_data[BLOCK_HEADER_SIZE..]);

        let tx_count = VarInt::consensus_decode(&mut cursor)?;
        if tx_count.0 == 0 {
            return Err(anyhow!("Block does not contain transactions"));
        }

        let mut version = [0u8; 4];
        cursor.read_exact(&mut version)?;

        let mut marker_flag = [0u8; 2];
        cursor.read_exact(&mut marker_flag)?;
        let has_witness = marker_flag == [0x00, 0x01];

        if !has_witness {
            cursor.set_position(cursor.position() - 2);
        }

        let input_count = VarInt::consensus_decode(&mut cursor)?;
        if input_count.0 == 0 {
            return Err(anyhow!("Coinbase tx has no inputs"));
        }

        cursor.set_position(cursor.position() + 36);

        let script_len = VarInt::consensus_decode(&mut cursor)?;

        let mut script_sig = vec![0u8; script_len.0 as usize];
        cursor.read_exact(&mut script_sig)?;

        Ok(script_sig)
    }

    pub fn check_coinbase_script(&self, block_data: &[u8]) -> Result<()> {
        let script = self.get_coinbase_script_sig(block_data)?;

        const ANT_TAG: &[u8] = &[0x41, 0x6e, 0x74, 0x50, 0x6f, 0x6f, 0x6c];
        const WHITE_TAG: &[u8] = &[0x57, 0x68, 0x69, 0x74, 0x65, 0x50, 0x6f, 0x6f, 0x6c];
        const BINANCE_TAG: &[u8] = &[0x62, 0x69, 0x6e, 0x61, 0x6e, 0x63, 0x65];
        const MINING_SQUARED_TAG: &[u8] = &[0x4d, 0x69, 0x6e, 0x69, 0x6e, 0x67, 0x53, 0x71, 0x75, 0x61];
        const MINING_SQUARED_TAG_2: &[u8] = &[0x30, 0x78, 0x37, 0x38, 0x33, 0x63, 0x33, 0x66, 0x30, 0x30];
        const BTC_COM_TAG: &[u8] = &[0x62, 0x74, 0x63, 0x63, 0x6f, 0x6d];
        const BRAIINS_TAG: &[u8] = &[0x2f, 0x73, 0x6c, 0x75, 0x73, 0x68, 0x2f];
        const ULTIMUS_TAG: &[u8] = &[0x75, 0x6c, 0x74, 0x69, 0x6d, 0x75, 0x73];
        const POOLIN_COM_TAG: &[u8] = &[0x70, 0x6f, 0x6f, 0x6c, 0x69, 0x6e, 0x2e, 0x63, 0x6f, 0x6d];

        
        if script.windows(ANT_TAG.len()).any(|w| w == ANT_TAG) {
            Ok(())
        } else if script.windows(WHITE_TAG.len()).any(|w| w == WHITE_TAG) {
            Ok(())
        } else if script.windows(BINANCE_TAG.len()).any(|w| w == BINANCE_TAG) {
            Ok(())
        } else if script.windows(MINING_SQUARED_TAG.len()).any(|w| w == MINING_SQUARED_TAG) {
            Ok(())
        } else if script.windows(MINING_SQUARED_TAG_2.len()).any(|w| w == MINING_SQUARED_TAG_2) {
            Ok(())
        } else if script.windows(BTC_COM_TAG.len()).any(|w| w == BTC_COM_TAG) {
            Ok(())
        } else if script.windows(BRAIINS_TAG.len()).any(|w| w == BRAIINS_TAG) {
            Ok(())
        } else if script.windows(ULTIMUS_TAG.len()).any(|w| w == ULTIMUS_TAG) {
            Ok(())
        } else if script.windows(POOLIN_COM_TAG.len()).any(|w| w == POOLIN_COM_TAG) {
            Ok(())
        } else {
           Err(anyhow!("Invalid coinbase script: required tag not found"))
        }
    }

    /// Initialize the token with configuration
    fn initialize(
        &self,
        token_units: u128,
        value_per_mint: u128,
        cap: u128,
        name_part1: u128,
        name_part2: u128,
        symbol: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        // Prevent multiple initializations
        self.observe_initialization()
            .map_err(|_| anyhow!("Contract already initialized"))?;

        // Set configuration
        self.set_value_per_mint(value_per_mint);
        self.set_cap(cap);
        self.set_data()?;

        // Create TokenName from the two parts
        let name = TokenName::new(name_part1, name_part2);
        <Self as MintableToken>::set_name_and_symbol(self, name, symbol);

        // Mint initial tokens
        if token_units > 0 {
            response.alkanes.0.push(self.mint(&context, token_units)?);
        }

        Ok(response)
    }

    /// Mint new tokens
    fn mint_tokens(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let block_data = CONTEXT.block();

        self.check_coinbase_script(&block_data)?;

        // Get transaction ID
        let txid = context.transaction_id()?;

        let tx_data = CONTEXT.transaction();
        let mut cursor = Cursor::new(tx_data.clone());
        let tx = consensus_decode::<Transaction>(&mut cursor)?;

        self.observe_address(&tx)?;

        // Enforce one mint per transaction
        if self.has_tx_hash(&txid) {
            return Err(anyhow!("Transaction already used for minting"));
        }

        // Check if minting would exceed cap
        if self.minted() >= self.cap() {
            return Err(anyhow!(
                "Supply cap reached: {} of {}",
                self.minted(),
                self.cap()
            ));
        }

        // Record transaction hash
        self.add_tx_hash(&txid)?;

        // Mint tokens
        let value = self.value_per_mint();
        response.alkanes.0.push(self.mint(&context, value)?);

        // Increment mint counter
        self.increment_mint()?;

        Ok(response)
    }

    /// Set the token name and symbol
    fn set_name_and_symbol(
        &self,
        name_part1: u128,
        name_part2: u128,
        symbol: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);

        // Create TokenName from the two parts
        let name = TokenName::new(name_part1, name_part2);
        <Self as MintableToken>::set_name_and_symbol(self, name, symbol);

        Ok(response)
    }

    /// Get the token name
    fn get_name(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.name().into_bytes().to_vec();

        Ok(response)
    }

    /// Get the token symbol
    fn get_symbol(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.symbol().into_bytes().to_vec();

        Ok(response)
    }

    /// Get the total supply
    fn get_total_supply(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.total_supply().to_le_bytes().to_vec();

        Ok(response)
    }

    /// Get the maximum supply cap
    fn get_cap(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.cap().to_le_bytes().to_vec();

        Ok(response)
    }

    /// Get the total minted count
    fn get_minted(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.minted().to_le_bytes().to_vec();

        Ok(response)
    }

    /// Get the value per mint
    fn get_value_per_mint(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.value_per_mint().to_le_bytes().to_vec();

        Ok(response)
    }

    /// Get the token data
    fn get_data(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        response.data = self.data();

        Ok(response)
    }
}

impl AlkaneResponder for MintableAlkane {}

// Use the MessageDispatch macro for opcode handling
declare_alkane! {
    impl AlkaneResponder for MintableAlkane {
        type Message = MintableAlkaneMessage;
    }
}