# free_mint.wasm

## Overview
A modernized and secure version of the free mint alkane contract that follows current best practices and security patterns while providing full functionality of a standard token plus free mint capabilities.

## Core Requirements

### Token Functionality
- Must implement all standard token functionality (name, symbol, total supply)
- Must support proper initialization sequence with authentication
- Must be compatible with the MintableToken trait in the factory module
- Must follow the owned token opcode format for consistency

### Mint Controls
- Must enforce one mint per transaction limit
- Must support configurable value per mint
- Must support optional maximum supply cap
- Must track total mints for analytics

### Security Requirements
- Must use proper initialization guard via observe_initialization()
- Must validate all numeric operations for overflow
- Must ensure transaction-bound mint limit is enforced cryptographically
- Must validate cap constraints before any mint operation

## Opcode Specification 

### Standard Operations (Matching Owned Format)
- 0: Initialize(auth_token_units, token_units)
- 77: Mint(token_units) 
- 88: SetNameAndSymbol(name, symbol)
- 99: GetName() -> String
- 100: GetSymbol() -> String  
- 101: GetTotalSupply() -> u128
- 1000: GetData() -> Vec<u8>

### Free Mint Specific Operations
- 102: GetCap() -> u128
- 103: GetMinted() -> u128
- 104: GetValuePerMint() -> u128

## Technical Implementation
- Use MessageDispatch derive macro for opcode handling
- Use declare_alkane! macro for proper runtime integration
- Store transaction hash for mint tracking
- Implement all view functions as no-state-change operations

## Migration Notes
This contract replaces the previous free mint implementation with a more secure version that:
1. Properly guards initialization
2. Enforces transaction-based mint limits
3. Uses modern alkane patterns
4. Provides complete view functions

## Security Patterns
- Call observe_initialization() in Initialize operation
- Validate transaction hash hasn't been used for minting
- Check cap constraints before allowing any mint
- Ensure all numeric operations use overflow_error checks
