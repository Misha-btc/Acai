# Technical Context

## Technologies Used

### Core Technologies
- **Rust** - Primary programming language
- **WebAssembly (WASM)** - Compilation target for the contract
- **Alkanes Framework** - Smart contract framework for token implementation
- **MessageDispatch** - Macro for opcode-based message dispatching

### Dependencies
- **alkanes-support** - Core support library for alkane contracts
- **alkanes-runtime** - Runtime support for alkane execution
- **metashrew-support** - Support library for metashrew compatibility
- **protorune-support** - Support for protorune protocol
- **alkane-factory-support** - Factory pattern support for token creation
- **ordinals** - Ordinals protocol integration
- **anyhow** - Error handling library
- **bitcoin** - Bitcoin protocol library
- **serde** - Serialization/deserialization framework
- **serde_json** - JSON support for serde

## Development Setup

### Project Structure
```
free-mint/
├── Cargo.toml           - Project manifest
├── memory-bank/         - Documentation and context
├── reference/           - Reference implementations
│   └── owned.rs         - Reference owned token implementation
└── src/                 - Source code
    ├── constants.rs     - Constant definitions
    ├── factory.rs       - Factory implementation
    └── lib.rs           - Main contract implementation with MessageDispatch
```

### Build Configuration
The project is configured as both a cdylib (for WebAssembly compilation) and rlib (for Rust library usage):

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

## Technical Constraints

### Compatibility Requirements
- Must be compatible with the MintableToken trait in the factory module
- Must follow the owned token opcode format for consistency
- Must support proper initialization sequence with authentication
- Must maintain backward compatibility with existing systems

### Security Requirements
- Must use proper initialization guard via observe_initialization()
- Must validate all numeric operations for overflow
- Must ensure transaction-bound mint limit is enforced cryptographically
- Must validate cap constraints before any mint operation
- Must prevent transaction replay attacks

### Performance Considerations
- Transaction hash storage must be efficient for large-scale usage
- Serialization/deserialization operations should be optimized
- Storage operations should be optimized to minimize resource usage
- Numeric operations must be checked for overflow to prevent vulnerabilities
- String operations use efficient encoding/decoding methods

## Integration Points

### Factory Integration
The contract implements the MintableToken trait to ensure compatibility with the factory module:
```rust
impl MintableToken for MintableAlkane {}
```

### Message Dispatch
The contract uses the MessageDispatch derive macro for opcode handling:
```rust
#[derive(MessageDispatch)]
enum MintableAlkaneMessage {
    #[opcode(0)]
    Initialize { /* ... */ },
    
    // Other operations...
}
```

### Opcode Interface
The contract exposes a standardized opcode interface for external interaction:
- Standard operations (0, 77, 88, 99-101, 1000)
- Free mint specific operations (102-104)
- All operations have comprehensive documentation

### Storage Interface
The contract uses StoragePointer for persistent state management:
- Key-value storage for token properties
- Structured data storage for complex objects
- Transaction hash storage using HashSet and serde_json

## Deployment Considerations
- The contract is compiled to WebAssembly for deployment
- Initialization must be performed with proper parameters:
  - Token units
  - Value per mint
  - Supply cap (0 for unlimited)
  - Name and symbol
- Transaction hash storage should be considered for long-term usage patterns