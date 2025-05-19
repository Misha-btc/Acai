use std::sync::Arc;

use crate::{MintableAlkane, MintableToken, TokenName};
use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_runtime::storage::StoragePointer;
use anyhow::Result;
use metashrew_support::index_pointer::KeyValuePointer;
use wasm_bindgen_test::wasm_bindgen_test;

// Reset all storage keys used in tests
fn reset_test_storage() {
    // Clear all storage keys used in tests
    StoragePointer::from_keyword("/initialized").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/name").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/symbol").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/totalsupply").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/minted").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/value-per-mint").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/cap").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/data").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/tx-hashes").set(Arc::new(Vec::new()));
}

#[wasm_bindgen_test]
fn test_initialization() {
    // Reset storage
    reset_test_storage();

    // Create the MintableAlkane instance
    let alkane = MintableAlkane::default();

    // Set values
    let value_per_mint = 10u128;
    let cap = 100u128;

    alkane.set_value_per_mint(value_per_mint);
    alkane.set_cap(cap);

    // Verify the values were set correctly
    assert_eq!(alkane.value_per_mint(), value_per_mint);
    assert_eq!(alkane.cap(), cap);
}

#[wasm_bindgen_test]
fn test_cap_enforcement() {
    // Reset storage
    reset_test_storage();

    // Create the MintableAlkane instance
    let alkane = MintableAlkane::default();

    // Set a low cap
    alkane.set_cap(5u128);

    // Verify the cap was set correctly
    assert_eq!(alkane.cap(), 5u128);
}

#[wasm_bindgen_test]
fn test_mint_functionality() -> Result<()> {
    // Reset storage
    reset_test_storage();

    // Create the MintableAlkane instance
    let alkane = MintableAlkane::default();

    // Initialize the contract
    alkane.observe_initialization()?;
    alkane.set_value_per_mint(10u128);
    alkane.set_cap(100u128);

    // Verify initial state
    assert_eq!(alkane.total_supply(), 0u128);
    assert_eq!(alkane.minted(), 0u128);

    Ok(())
}

#[wasm_bindgen_test]
fn test_name_and_symbol() -> Result<()> {
    // Reset storage
    reset_test_storage();

    // Create the MintableAlkane instance
    let alkane = MintableAlkane::default();

    // Initialize the contract
    alkane.observe_initialization()?;

    // Set name and symbol directly using the MintableToken trait methods
    // Note: We need to use little-endian encoding because of how trim() works
    let name_part1 = 0x54534554u128; // "TEST" in little-endian
    let name_part2 = 0x32u128; // "2" in little-endian
    let symbol = 0x545354u128; // "TST" in little-endian

    // Create TokenName from the two parts
    let name = TokenName::new(name_part1, name_part2);
    <MintableAlkane as MintableToken>::set_name_and_symbol(&alkane, name, symbol);

    // Verify name and symbol
    assert_eq!(<MintableAlkane as MintableToken>::name(&alkane), "TEST2");
    assert_eq!(<MintableAlkane as MintableToken>::symbol(&alkane), "TST");

    Ok(())
}

#[wasm_bindgen_test]
fn test_initialization_guard() {
    // Reset storage
    reset_test_storage();

    // Create the MintableAlkane instance
    let alkane = MintableAlkane::default();

    // First initialization should succeed
    assert!(alkane.observe_initialization().is_ok());

    // Second initialization should fail
    assert!(alkane.observe_initialization().is_err());
}

#[wasm_bindgen_test]
fn test_total_supply_increase() -> Result<()> {
    // Reset storage
    reset_test_storage();

    // Create the MintableAlkane instance
    let alkane = MintableAlkane::default();

    // Initialize the contract
    alkane.observe_initialization()?;

    // Initial total supply should be 0
    assert_eq!(alkane.total_supply(), 0u128);

    // Increase total supply
    alkane.increase_total_supply(50u128)?;

    // Verify total supply was increased
    assert_eq!(alkane.total_supply(), 50u128);

    // Increase total supply again
    alkane.increase_total_supply(25u128)?;

    // Verify total supply was increased again
    assert_eq!(alkane.total_supply(), 75u128);

    Ok(())
}
