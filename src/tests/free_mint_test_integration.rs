use crate::tests::std::free_mint_build;
use crate::{MintableAlkane, MintableToken, TokenName, ALKANE_FACTORY_FREE_MINT_ID};
use alkanes::indexer::index_block;
use alkanes::message::AlkaneMessageContext;
use alkanes::tests::helpers::{self as alkane_helpers, clear};
use alkanes::view;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use alkanes_support::response::ExtendedCallResponse;
use alkanes_support::trace::{Trace, TraceEvent};
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::Witness;
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::utils::consensus_encode;
use protorune::balance_sheet::load_sheet;
use protorune::message::MessageContext;
use protorune::tables::RuneTable;
use protorune::test_helpers::create_block_with_coinbase_tx;
use protorune_support::balance_sheet::{BalanceSheet, BalanceSheetOperations};
use std::fmt::Write;
use wasm_bindgen_test::wasm_bindgen_test;

// Helper function to create a block with a free-mint deployment
fn init_block_with_free_mint_deployment() -> Result<(bitcoin::Block, AlkaneId)> {
    let block_height = 840_000;

    // Initialize the free-mint contract
    let token_units = 1000u128;
    let value_per_mint = 10u128;
    let cap = 100u128;
    let name_part1 = 0x54534554u128; // "TEST" in little-endian
    let name_part2 = 0x32u128; // "2" in little-endian
    let symbol = 0x545354u128; // "TST" in little-endian

    let test_block = create_init_tx(
        token_units,
        value_per_mint,
        cap,
        name_part1,
        name_part2,
        symbol,
    );

    Ok((test_block, AlkaneId::new(4, ALKANE_FACTORY_FREE_MINT_ID)))
}

// Helper function to create a transaction that initializes the free-mint contract
fn create_init_tx(
    token_units: u128,
    value_per_mint: u128,
    cap: u128,
    name_part1: u128,
    name_part2: u128,
    symbol: u128,
) -> bitcoin::Block {
    alkane_helpers::init_with_multiple_cellpacks_with_tx(
        vec![free_mint_build::get_bytes()],
        vec![Cellpack {
            target: AlkaneId::new(3, ALKANE_FACTORY_FREE_MINT_ID),
            // Initialize opcode (0) with parameters
            inputs: vec![
                0,
                token_units,
                value_per_mint,
                cap,
                name_part1,
                name_part2,
                symbol,
            ],
        }],
    )
}

// Helper function to create a transaction that mints tokens
fn create_mint_tx(
    test_block: &mut bitcoin::Block,
    free_mint_deployment: AlkaneId,
    previous_outpoint: OutPoint,
) -> OutPoint {
    // Create a transaction that mints tokens
    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: free_mint_deployment,
                // Mint opcode (77) with no parameters
                inputs: vec![77],
            }],
            previous_outpoint,
            false,
        ),
    );

    // Return the outpoint of the transaction we just added
    OutPoint {
        txid: test_block.txdata.last().unwrap().compute_txid(),
        vout: 0,
    }
}

fn get_sheet_for_outpoint(
    test_block: &bitcoin::Block,
    tx_num: usize,
    vout: u32,
) -> Result<BalanceSheet<IndexPointer>> {
    let outpoint = OutPoint {
        txid: test_block.txdata[tx_num].compute_txid(),
        vout,
    };
    let ptr = RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES
        .select(&consensus_encode(&outpoint)?);
    let sheet = load_sheet(&ptr);
    println!(
        "balances at outpoint tx {} vout {}: {:?}",
        tx_num, vout, sheet
    );
    Ok(sheet)
}

pub fn get_last_outpoint_sheet(test_block: &bitcoin::Block) -> Result<BalanceSheet<IndexPointer>> {
    let len = test_block.txdata.len();
    get_sheet_for_outpoint(test_block, len - 1, 0)
}

// Helper function to get the balance of a token
fn get_token_balance(block: &bitcoin::Block, token_id: AlkaneId) -> Result<u128> {
    let sheet = get_last_outpoint_sheet(block)?;
    Ok(sheet.get_cached(&token_id.into()))
}

#[wasm_bindgen_test]
fn test_free_mint_initialization() -> Result<()> {
    clear();

    let block_height = 840_000;
    let (test_block, free_mint_deployment) = init_block_with_free_mint_deployment()?;

    // Index the block
    index_block(&test_block, block_height)?;

    // Check the token balance
    let balance = get_token_balance(&test_block, free_mint_deployment)?;
    assert_eq!(
        balance, 1000u128,
        "Initial token balance should match token_units"
    );
    Ok(())
}
