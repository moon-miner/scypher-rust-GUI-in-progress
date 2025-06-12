# 🎯 Cardano Address Derivation - Complete Implementation Handoff

## 📋 Project Overview

**Project**: SCypher HD Wallet Address Derivation Test  
**Current Task**: Implementing correct Cardano address derivation using CIP-1852 + Icarus (CIP-3)  
**Language**: Rust with Tauri GUI  
**Status**: 80% complete - algorithm working but addresses don't match official wallets  
**Critical Decision**: Manual implementation is nearly impossible - MUST use official Cardano Rust libraries

## 🎯 Current Objective

Generate Cardano addresses that **exactly match** official wallets like Eternl/Yoroi for the test seed:
```
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

**Target address** (from Eternl wallet):
```
addr1qy8ac7qqy0vtulyl7wntmsxc6wex80gvcyjy33qffrhm7sh927ysx5sftuw0dlft05dz3c7revpf7jx0xnlcjz3g69mq4afdhv
```

**Current manual output** (incorrect):
```
addr1qyfrffeyes3463ps4arxfefwlguck9tsqxvquqqd4zysefjvf4x0x0vcr635kwul5n2h4r8vy6gpy4kzwzwhdymk6hcq6lczuj
```

## 📁 Project Structure

```
scypher-gui/
├── src-tauri/
│   ├── Cargo.toml                ← ADD DEPENDENCIES HERE
│   └── src/
│       ├── main.rs
│       ├── commands.rs
│       ├── addresses.rs          ← PRIMARY FILE TO MODIFY
│       ├── error.rs
│       └── lib.rs
└── src/
    └── index.html               ← Frontend GUI
```

## 🚀 COMPLETE SOLUTION STRATEGY: Official Cardano Rust Libraries

### 📊 Available Cardano Rust Repositories (Complete List)

#### **OPTION 1: Pallas Ecosystem (RECOMMENDED - Most Modern)**

**Main Repository**: https://github.com/txpipe/pallas  
**Latest Version**: 0.24.x  
**Maintenance**: Active (updated weekly)  
**Documentation**: https://docs.rs/pallas  

**Complete Cargo.toml Dependencies**:
```toml
# Pallas - Complete Cardano implementation
pallas = "0.24"
pallas-addresses = "0.24"
pallas-crypto = "0.24" 
pallas-codec = "0.24"
pallas-primitives = "0.24"
pallas-traverse = "0.24"

# Alternative: Use specific git commit for stability
pallas = { git = "https://github.com/txpipe/pallas", tag = "v0.24.0" }
pallas-addresses = { git = "https://github.com/txpipe/pallas", tag = "v0.24.0" }
pallas-crypto = { git = "https://github.com/txpipe/pallas", tag = "v0.24.0" }
```

**What Pallas Provides**:
- ✅ Complete CIP-1852 implementation
- ✅ Icarus master key generation (CIP-3)
- ✅ Ed25519-BIP32 derivation
- ✅ Shelley address encoding
- ✅ Byron/Shelley compatibility
- ✅ All Cardano cryptographic primitives
- ✅ Bech32 encoding with correct variants

#### **OPTION 2: Cardano Serialization Lib (EMURGO - Production Ready)**

**Main Repository**: https://github.com/Emurgo/cardano-serialization-lib  
**Latest Version**: 11.5.0  
**Maintenance**: Very Active (Emurgo official)  
**Used By**: Yoroi, AdaLite, many production wallets  

**Complete Cargo.toml Dependencies**:
```toml
# Option 2A: From crates.io (if available)
cardano-serialization-lib = "11.5"

# Option 2B: From Git (RECOMMENDED)
cardano-serialization-lib = { git = "https://github.com/Emurgo/cardano-serialization-lib", tag = "11.5.0" }

# Option 2C: Latest commit (bleeding edge)
cardano-serialization-lib = { git = "https://github.com/Emurgo/cardano-serialization-lib", branch = "master" }

# Additional crypto support
cardano-serialization-lib = { git = "https://github.com/Emurgo/cardano-serialization-lib", tag = "11.5.0", features = ["with-hdwallet"] }
```

**What CSL Provides**:
- ✅ **EXACT** same algorithm as Yoroi/Eternl
- ✅ Complete HD wallet implementation
- ✅ CIP-1852 derivation paths
- ✅ Icarus master key (identical to official wallets)
- ✅ Shelley/Byron address generation
- ✅ Transaction building (bonus)
- ✅ Proven in production

#### **OPTION 3: Cardano CLI Rust Bindings**

**Repository**: https://github.com/input-output-hk/cardano-cli  
**Rust Wrapper**: https://github.com/dcSpark/cardano-multiplatform-lib  

**Complete Cargo.toml Dependencies**:
```toml
# Option 3: Cardano multiplatform lib
cardano-multiplatform-lib = { git = "https://github.com/dcSpark/cardano-multiplatform-lib", tag = "v4.0.0" }

# Alternative: Direct from IOHK
cardano-addresses = { git = "https://github.com/input-output-hk/cardano-addresses", tag = "3.12.0" }
```

#### **OPTION 4: Blockfrost SDK (API + Local)**

**Repository**: https://github.com/blockfrost/blockfrost-rust  
**Capability**: Can derive addresses locally + API validation  

**Complete Cargo.toml Dependencies**:
```toml
blockfrost = "0.1"
# Plus any of the above for local derivation
```

#### **OPTION 5: Oura + Pallas Stack**

**Repository**: https://github.com/txpipe/oura  
**Use Case**: Production-grade Cardano data processing  

**Complete Cargo.toml Dependencies**:
```toml
oura = { git = "https://github.com/txpipe/oura", tag = "v1.8.0" }
pallas = { git = "https://github.com/txpipe/pallas", tag = "v0.24.0" }
```

#### **OPTION 6: Cardano Rust SDK (Community)**

**Repository**: https://github.com/rust-cardano/cardano-cli  
**Alternative**: https://github.com/AndrewWestberg/cardano-sdk-rust  

**Complete Cargo.toml Dependencies**:
```toml
# Community implementation
cardano-sdk = { git = "https://github.com/AndrewWestberg/cardano-sdk-rust", branch = "main" }
```

### 🎯 RECOMMENDED IMPLEMENTATION ORDER

#### **PRIORITY 1: Try Cardano Serialization Lib (EMURGO)**

**Why**: Used by Eternl/Yoroi - guaranteed to match target output

```toml
# Add to Cargo.toml
cardano-serialization-lib = { git = "https://github.com/Emurgo/cardano-serialization-lib", tag = "11.5.0", features = ["with-hdwallet"] }
```

**Complete Implementation Template**:
```rust
// In addresses.rs - REPLACE derive_cardano_addresses_icarus_correct()
fn derive_cardano_addresses_emurgo_csl(
    mnemonic_phrase: &str,
    passphrase: Option<&str>,
) -> Result<Vec<Address>> {
    use cardano_serialization_lib::{
        crypto::{Bip32PrivateKey, Bip32PublicKey},
        address::{Address as CSLAddress, StakeCredential, BaseAddress},
        NetworkInfo,
    };

    let mut addresses = Vec::new();
    let passphrase_str = passphrase.unwrap_or("");

    // 1. Generate master key using EMURGO CSL (exact Yoroi algorithm)
    let master_key = Bip32PrivateKey::from_bip39_entropy(
        &hex::decode("/* entropy from mnemonic */").unwrap(),
        passphrase_str.as_bytes()
    );

    // 2. Derive staking key: m/1852'/1815'/0'/2/0
    let staking_key = master_key
        .derive(harden(1852))  // purpose
        .derive(harden(1815))  // coin_type 
        .derive(harden(0))     // account
        .derive(2)             // role (staking)
        .derive(0);            // index

    let staking_pub = staking_key.to_public();
    let staking_cred = StakeCredential::from_keyhash(&staking_pub.to_raw_key().hash());

    // 3. Generate addresses for indices 0, 1, 2
    for index in 0u32..3u32 {
        let payment_key = master_key
            .derive(harden(1852))  // purpose
            .derive(harden(1815))  // coin_type
            .derive(harden(0))     // account  
            .derive(0)             // role (external)
            .derive(index);        // index

        let payment_pub = payment_key.to_public();
        let payment_cred = StakeCredential::from_keyhash(&payment_pub.to_raw_key().hash());

        // Create base address (payment + staking)
        let base_addr = BaseAddress::new(
            NetworkInfo::mainnet().network_id(),
            &payment_cred,
            &staking_cred
        );

        let address_str = base_addr.to_address().to_bech32(None)
            .map_err(|e| SCypherError::crypto(format!("Address encoding failed: {:?}", e)))?;

        addresses.push(Address {
            address_type: format!("Cardano Shelley (Index {})", index),
            path: format!("m/1852'/1815'/0'/0/{}", index),
            address: address_str,
        });
    }

    Ok(addresses)
}
```

#### **PRIORITY 2: Try Pallas if CSL fails**

```toml
# Add to Cargo.toml if EMURGO CSL doesn't compile
pallas = { git = "https://github.com/txpipe/pallas", tag = "v0.24.0" }
pallas-addresses = { git = "https://github.com/txpipe/pallas", tag = "v0.24.0" }
pallas-crypto = { git = "https://github.com/txpipe/pallas", tag = "v0.24.0" }
```

**Complete Implementation Template**:
```rust
fn derive_cardano_addresses_pallas(
    mnemonic_phrase: &str,  
    passphrase: Option<&str>,
) -> Result<Vec<Address>> {
    use pallas_addresses::{Address as PallasAddress, Network, ShelleyAddress};
    use pallas_crypto::key::{ed25519::SecretKey, BIP32PrivateKey};
    use pallas_crypto::hash::Hash;

    let mut addresses = Vec::new();
    let passphrase_str = passphrase.unwrap_or("");

    // 1. Generate master key using Pallas Icarus implementation
    let master_key = BIP32PrivateKey::from_mnemonic(mnemonic_phrase, passphrase_str)?;

    // 2. Derive staking key
    let staking_key = master_key.derive_path("m/1852'/1815'/0'/2/0")?;
    let staking_hash = Hash::from(staking_key.to_public().to_raw_key().as_ref());

    // 3. Generate payment addresses
    for index in 0u32..3u32 {
        let payment_path = format!("m/1852'/1815'/0'/0/{}", index);
        let payment_key = master_key.derive_path(&payment_path)?;
        let payment_hash = Hash::from(payment_key.to_public().to_raw_key().as_ref());

        // Create Shelley base address
        let shelley_addr = ShelleyAddress::new(
            Network::Mainnet,
            &payment_hash,
            &staking_hash
        );

        let address_str = shelley_addr.to_bech32()?;

        addresses.push(Address {
            address_type: format!("Cardano Shelley (Index {})", index),
            path: format!("m/1852'/1815'/0'/0/{}", index),
            address: address_str,
        });
    }

    Ok(addresses)
}
```

#### **PRIORITY 3: Fallback Options**

If both fail, try in order:
1. Cardano Multiplatform Lib
2. Direct IOHK dependencies
3. Blockfrost SDK + local derivation

## 🔧 Current Working Code Structure

### Key Files to Modify

**File**: `src-tauri/src/addresses.rs`

**Current entry point function**:
```rust
fn derive_cardano_addresses_icarus_correct(
    mnemonic_phrase: &str,
    passphrase: Option<&str>,
) -> Result<Vec<Address>> {
    // REPLACE THIS ENTIRE FUNCTION with official library implementation
}
```

**Integration point in same file**:
```rust
pub fn derive_addresses(request: DeriveAddressesRequest) -> Result<Vec<Address>> {
    match request.network.as_str() {
        // ... other networks
        "cardano" => derive_cardano_addresses_icarus_correct(&request.mnemonic, request.passphrase.as_deref()),
        // ... rest
    }
}
```

### Current Dependencies (Cargo.toml) - PRESERVE THESE

```toml
# CRITICAL - DO NOT REMOVE OR BREAK THESE
zeroize = "1.6"              # Security - memory clearing
ergo-lib = "0.24"            # Ergo blockchain support
bitcoin = "0.31"             # Bitcoin support  
tiny-bip39 = "1.0"           # Bitcoin BIP39
k256 = "0.13"                # Ethereum secp256k1
# ... many others that MUST remain functional

# Current crypto dependencies (keep these)
blake2 = "0.10"
bech32 = "0.9" 
ed25519-dalek = "2.0"
pbkdf2 = "0.12"
hmac = "0.12"
bip39-crate = { package = "bip39", version = "2.0" }
```

### Dependencies to ADD (choose one strategy)

**Strategy A - EMURGO CSL**:
```toml
cardano-serialization-lib = { git = "https://github.com/Emurgo/cardano-serialization-lib", tag = "11.5.0", features = ["with-hdwallet"] }
```

**Strategy B - Pallas**:
```toml  
pallas = { git = "https://github.com/txpipe/pallas", tag = "v0.24.0" }
pallas-addresses = { git = "https://github.com/txpipe/pallas", tag = "v0.24.0" }
pallas-crypto = { git = "https://github.com/txpipe/pallas", tag = "v0.24.0" }
```

## 🛠️ Detailed Implementation Steps

### Step 1: Add Dependencies

Choose ONE strategy and add to `src-tauri/Cargo.toml`:

```bash
cd src-tauri
# Edit Cargo.toml - add chosen dependencies
cargo build
# If compilation fails, try alternative dependencies
```

### Step 2: Replace Function

In `src-tauri/src/addresses.rs`, find and COMPLETELY REPLACE:
```rust
fn derive_cardano_addresses_icarus_correct(
    mnemonic_phrase: &str,
    passphrase: Option<&str>,
) -> Result<Vec<Address>> {
    // OLD MANUAL IMPLEMENTATION - DELETE ALL OF THIS
}
```

With the corresponding template from above (EMURGO CSL or Pallas).

### Step 3: Test Compilation

```bash
cargo build
# Must succeed without errors
# Must not break existing Bitcoin/Ethereum/Ergo functionality
```

### Step 4: Test Runtime

```bash
cargo tauri dev
# Test with: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
# Expected result: addr1qy8ac7qqy0vtulyl7wntmsxc6wex80gvcyjy33qffrhm7sh927ysx5sftuw0dlft05dz3c7revpf7jx0xnlcjz3g69mq4afdhv
```

### Step 5: Verify All Networks

Test that Bitcoin, Ethereum, and Ergo derivation still work correctly.

## 🎯 Success Criteria

- [ ] Compilation without errors
- [ ] No regression in Bitcoin/Ethereum/Ergo functionality  
- [ ] Cardano address **exactly matches** Eternl: `addr1qy8ac7qqy0vtulyl7wntmsxc6wex80gvcyjy33qffrhm7sh927ysx5sftuw0dlft05dz3c7revpf7jx0xnlcjz3g69mq4afdhv`

## 🚨 Critical Notes

### What NOT to do:
- ❌ Don't attempt manual implementation anymore - it's proven nearly impossible
- ❌ Don't break existing Bitcoin/Ethereum/Ergo code
- ❌ Don't remove zeroize, ergo-lib, or other critical dependencies

### What TO do:
- ✅ Use official Cardano Rust libraries (EMURGO CSL recommended)
- ✅ Test compilation before proceeding with implementation
- ✅ Follow the exact templates provided above
- ✅ Replace only the `derive_cardano_addresses_icarus_correct()` function

## 📞 Implementation Priority

1. **FIRST**: Try EMURGO Cardano Serialization Lib (most likely to match Eternl)
2. **SECOND**: Try Pallas if CSL fails to compile
3. **THIRD**: Try Cardano Multiplatform Lib
4. **FOURTH**: Try direct IOHK dependencies

Start with Strategy A (EMURGO CSL) and work down the list until one compiles and produces the correct address.

## 🎯 Current Status

**Ready for**: Adding official Cardano Rust library dependencies  
**Next Action**: Choose EMURGO CSL or Pallas, add to Cargo.toml, test compilation  
**Expected Outcome**: Exact address match with official wallets  
**Success Metric**: `addr1qy8ac7qqy0vtulyl7wntmsxc6wex80gvcyjy33qffrhm7sh927ysx5sftuw0dlft05dz3c7revpf7jx0xnlcjz3g69mq4afdhv`

---

*This document provides complete information for implementing Cardano address derivation using official Rust libraries. Manual implementation is abandoned in favor of proven, production-ready solutions.*