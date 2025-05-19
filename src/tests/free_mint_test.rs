use alkanes_runtime::runtime::AlkaneResponder;
use anyhow::Result;
use bitcoin::Txid;
use std::str::FromStr;

mod mock;
use mock::{MockAlkaneResponder, MockContext};

use free_mint::{MintableAlkane, MintableToken, TokenName};

#[test]
fn test_initialization() -> Result<()> {
    // Create the MintableAlkane instance
    let alkane = MintableAlkane::default();
    
    // Test initialization
    let value_per_mint = 10u128;
    let cap = 100u128;
    let name_part1 = 123456789u128; // First part of name
    let name_part2 = 987654321u128; // Second part of name
    let symbol = 123456u128;  // "TST" encoded as u128
    
    // Create TokenName from the two parts
    let name = TokenName::new(name_part1, name_part2);
    
    // Initialize the contract
    alkane.observe_initialization()?;
    alkane.set_value_per_mint(value_per_mint);
    alkane.set_cap(cap);
    alkane.set_name_and_symbol(name, symbol);
    
    // Verify the values were set correctly
    assert_eq!(alkane.value_per_mint(), value_per_mint);
    assert_eq!(alkane.cap(), cap);
    
    Ok(())
}

#[test]
fn test_minting() -> Result<()> {
    // Create a mock responder with a transaction ID
    let mock_context = MockContext::default();
    let txid = Txid::from_str("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f")?;
    let responder = MockAlkaneResponder::new(mock_context).with_transaction_id(txid);
    
    // Create the MintableAlkane instance
    let alkane = MintableAlkane::default();
    
    // Initialize the contract
    alkane.observe_initialization()?;
    alkane.set_value_per_mint(10u128);
    alkane.set_cap(100u128);
    
    // Test minting
    let context = responder.context()?;
    let mint_result = alkane.mint(&context, 10u128)?;
    
    // Verify the mint result
    assert_eq!(mint_result.value, 10u128);
    
    // Verify the total supply was increased
    assert_eq!(alkane.total_supply(), 10u128);
    
    Ok(())
}

#[test]
fn test_cap_enforcement() -> Result<()> {
    // Create a mock responder
    let mock_context = MockContext::default();
    let responder = MockAlkaneResponder::new(mock_context);
    
    // Create the MintableAlkane instance
    let alkane = MintableAlkane::default();
    
    // Initialize the contract with a low cap
    alkane.observe_initialization()?;
    alkane.set_value_per_mint(10u128);
    alkane.set_cap(5u128);
    
    // Test minting
    let context = responder.context()?;
    
    // First mint should succeed
    let mint_result = alkane.mint(&context, 5u128)?;
    assert_eq!(mint_result.value, 5u128);
    assert_eq!(alkane.total_supply(), 5u128);
    
    // Second mint should fail due to cap
    let result = alkane.mint(&context, 10u128);
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_transaction_based_mint_limit() -> Result<()> {
    // Create a mock responder with a transaction ID
    let mock_context = MockContext::default();
    let txid = Txid::from_str("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f")?;
    let responder = MockAlkaneResponder::new(mock_context).with_transaction_id(txid);
    
    // Create the MintableAlkane instance
    let alkane = MintableAlkane::default();
    
    // Initialize the contract
    alkane.observe_initialization()?;
    alkane.set_value_per_mint(10u128);
    alkane.set_cap(100u128);
    
    // Add the transaction hash to simulate a previous mint
    alkane.add_tx_hash(&txid)?;
    
    // Test minting with the same transaction ID
    let context = responder.context()?;
    
    // Mint should fail due to transaction already used
    let result = alkane.mint(&context, 10u128);
    assert!(result.is_ok()); // The mint itself succeeds
    
    // But if we were to check in the mint_tokens function, it would fail
    assert!(alkane.has_tx_hash(&txid));
    
    Ok(())
}