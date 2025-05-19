# System Patterns

## Architecture Overview
The free_mint.wasm project implements a token contract with free mint capabilities using the alkane framework. The architecture follows a modular approach with clear separation of concerns:

```
free_mint.wasm
├── Core Token Logic (MintableAlkane)
├── Message Handling (MintableAlkaneMessage)
├── Factory Support (MintableToken trait)
├── Constants
└── Reference Implementation (owned.rs)
```

## Key Components

### MintableAlkane
The main implementation of the free mint token contract. It handles:
- Token initialization with proper guards
- Minting operations with transaction validation
- Supply cap enforcement
- Transaction-based mint limits with hash tracking
- View functions for contract state
- Comprehensive error handling

### MintableAlkaneMessage
An enum that defines the message structure for opcode-based dispatch:
- Uses the MessageDispatch derive macro
- Defines all supported operations with their parameters
- Specifies return types for view functions
- Provides a clean interface for opcode handling

### MintableToken Trait
A shared trait that provides common token functionality:
- Name and symbol management
- Total supply tracking
- Data storage and retrieval
- Initialization guard

## Design Patterns

### Storage Pattern
The contract uses StoragePointer for persistent state management:
- `/minted` - Tracks total mints
- `/value-per-mint` - Stores value per mint
- `/cap` - Stores maximum supply cap
- `/name` - Stores token name
- `/symbol` - Stores token symbol
- `/totalsupply` - Tracks total supply
- `/data` - Stores additional token data
- `/initialized` - Guards against multiple initializations
- `/tx-hashes` - Stores used transaction hashes

### Transaction Hash Tracking
The contract implements a robust transaction hash tracking system:
- Stores transaction hashes in a HashSet
- Serializes/deserializes using serde_json
- Validates each mint operation against previously used hashes
- Prevents replay attacks and enforces one mint per transaction

### Security Patterns
1. **Initialization Guard**
   - Uses `observe_initialization()` to prevent multiple initializations
   - Sets a flag in storage to track initialization state
   - Returns descriptive error messages on failure

2. **Transaction-Based Mint Limit**
   - Enforces one mint per transaction
   - Validates transaction hash to prevent replay attacks
   - Stores transaction hashes for validation

3. **Overflow Protection**
   - Uses `overflow_error` checks for all numeric operations
   - Provides descriptive error messages
   - Prevents integer overflow vulnerabilities

4. **Cap Enforcement**
   - Validates against supply cap before any mint operation
   - Returns detailed error with current and maximum values
   - Prevents exceeding the configured supply limit

### Message Dispatch Pattern
The contract uses the MessageDispatch derive macro for clean opcode handling:
- Automatically generates opcode dispatch logic
- Provides type safety for parameters and return values
- Simplifies the implementation of the AlkaneResponder trait
- Follows the standardized opcode format:
  - 0: Initialize
  - 77: Mint
  - 88: SetNameAndSymbol
  - 99-101: Standard view functions
  - 102-104: Free mint specific view functions
  - 1000: GetData

## Component Relationships
- MintableAlkane implements the MintableToken trait
- MintableAlkaneMessage defines the message structure
- MessageDispatch handles opcode routing
- AlkaneResponder trait is implemented for execution
- Factory module provides shared token functionality
- Constants define token identifiers