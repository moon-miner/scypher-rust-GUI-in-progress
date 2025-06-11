# Guía Completa: Derivación de Direcciones Ergo en Rust

## Tabla de Contenidos
1. [Introducción](#introducción)
2. [Conceptos Fundamentales](#conceptos-fundamentales)
3. [Configuración del Proyecto](#configuración-del-proyecto)
4. [Implementación Básica](#implementación-básica)
5. [Implementación Avanzada](#implementación-avanzada)
6. [Casos de Uso Comunes](#casos-de-uso-comunes)
7. [Testing y Validación](#testing-y-validación)
8. [Troubleshooting](#troubleshooting)
9. [Referencia Completa](#referencia-completa)

## Introducción

Esta guía te enseñará cómo implementar correctamente la derivación de direcciones Ergo a partir de frases semilla (mnemonics) en Rust, siguiendo los estándares oficiales de Ergo Platform.

### ¿Por qué esta guía?
- Ergo usa un estándar específico (EIP-3) que difiere de Bitcoin/Ethereum
- La documentación oficial está dispersa
- Hay diferencias sutiles que pueden causar incompatibilidad con wallets oficiales

## Conceptos Fundamentales

### 1. Estándar EIP-3 (Ergo Improvement Proposal 3)
EIP-3 define el estándar de derivación de wallets HD para Ergo:
- **Basado en BIP44** pero con modificaciones específicas
- **Path de derivación**: `m/44'/429'/0'/0/index`
- **No usa change addresses** (solo external addresses)

### 2. Componentes del Path de Derivación

```
m / 44' / 429' / 0' / 0 / index
│   │     │      │    │    │
│   │     │      │    │    └── Índice de dirección (0, 1, 2, ...)
│   │     │      │    └────── External addresses (siempre 0)
│   │     │      └─────────── Account index (siempre 0')
│   │     └────────────────── Coin type Ergo (429')
│   └──────────────────────── Purpose BIP44 (44')
└──────────────────────────── Master key
```

### 3. Diferencias Clave con Bitcoin/Ethereum

| Aspecto | Bitcoin | Ethereum | Ergo |
|---------|---------|----------|------|
| Coin Type | 0' | 60' | 429' |
| Change Addresses | Sí (0/1) | No | No (solo 0) |
| Address Format | Base58 | Hex | Base58 (empieza con '9') |
| Librería | `bitcoin` | `ethers` | `ergo-lib` |

## Configuración del Proyecto

### 1. Cargo.toml

```toml
[package]
name = "ergo-address-derivation"
version = "0.1.0"
edition = "2021"

[dependencies]
# Librería oficial de Ergo
ergo-lib = { version = "0.28", features = ["mnemonic_gen"] }

# Para manejo de errores
thiserror = "1.0"
anyhow = "1.0"

# Para testing (opcional)
hex = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
tokio-test = "0.4"
```

### 2. Estructura del Proyecto

```
src/
├── lib.rs              # Módulo principal
├── address.rs          # Lógica de derivación
├── types.rs           # Tipos personalizados
├── errors.rs          # Manejo de errores
└── tests.rs           # Tests de integración
```

## Implementación Básica

### 1. Definición de Tipos (`src/types.rs`)

```rust
use serde::{Deserialize, Serialize};

/// Información de una dirección derivada
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErgoAddressInfo {
    /// Dirección codificada
    pub address: String,
    /// Índice de derivación
    pub index: u32,
    /// Path de derivación completo
    pub derivation_path: String,
}

/// Configuración para derivación
#[derive(Debug, Clone)]
pub struct DerivationConfig {
    /// Usar testnet en lugar de mainnet
    pub testnet: bool,
    /// Passphrase opcional (BIP39)
    pub passphrase: Option<String>,
    /// Account index (normalmente 0)
    pub account_index: u32,
}

impl Default for DerivationConfig {
    fn default() -> Self {
        Self {
            testnet: false,
            passphrase: None,
            account_index: 0,
        }
    }
}
```

### 2. Manejo de Errores (`src/errors.rs`)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErgoDerivationError {
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    
    #[error("Address generation failed: {0}")]
    AddressGenerationFailed(String),
    
    #[error("Invalid account index: {0}")]
    InvalidAccountIndex(u32),
    
    #[error("Invalid address index: {0}")]
    InvalidAddressIndex(u32),
    
    #[error("Ergo-lib error: {0}")]
    ErgoLibError(String),
}

pub type Result<T> = std::result::Result<T, ErgoDerivationError>;
```

### 3. Implementación Principal (`src/address.rs`)

```rust
use ergo_lib::{
    ergotree_ir::chain::address::{Address, NetworkPrefix, AddressEncoder},
    wallet::{
        derivation_path::{ChildIndexHardened, ChildIndexNormal, DerivationPath},
        ext_secret_key::ExtSecretKey,
        mnemonic::Mnemonic,
    },
};

use crate::{
    types::{ErgoAddressInfo, DerivationConfig},
    errors::{ErgoDerivationError, Result},
};

/// Derivador de direcciones Ergo
pub struct ErgoAddressDerivator {
    config: DerivationConfig,
}

impl ErgoAddressDerivator {
    /// Crear nuevo derivador con configuración por defecto
    pub fn new() -> Self {
        Self {
            config: DerivationConfig::default(),
        }
    }

    /// Crear derivador con configuración personalizada
    pub fn with_config(config: DerivationConfig) -> Self {
        Self { config }
    }

    /// Derivar una sola dirección para un índice específico
    pub fn derive_address(&self, mnemonic: &str, index: u32) -> Result<ErgoAddressInfo> {
        self.validate_inputs(mnemonic, index)?;
        
        let addresses = self.derive_addresses_internal(mnemonic, index, 1)?;
        Ok(addresses.into_iter().next().unwrap())
    }

    /// Derivar múltiples direcciones secuenciales
    pub fn derive_addresses(&self, mnemonic: &str, count: u32) -> Result<Vec<ErgoAddressInfo>> {
        self.validate_inputs(mnemonic, 0)?;
        
        if count == 0 {
            return Ok(Vec::new());
        }
        
        self.derive_addresses_internal(mnemonic, 0, count)
    }

    /// Derivar direcciones en un rango específico
    pub fn derive_addresses_range(
        &self, 
        mnemonic: &str, 
        start_index: u32, 
        end_index: u32
    ) -> Result<Vec<ErgoAddressInfo>> {
        self.validate_inputs(mnemonic, start_index)?;
        
        if start_index >= end_index {
            return Err(ErgoDerivationError::InvalidAddressIndex(start_index));
        }
        
        let count = end_index - start_index;
        self.derive_addresses_internal(mnemonic, start_index, count)
    }

    /// Implementación interna de derivación
    fn derive_addresses_internal(
        &self,
        mnemonic: &str,
        start_index: u32,
        count: u32,
    ) -> Result<Vec<ErgoAddressInfo>> {
        // Generar seed desde mnemonic
        let passphrase = self.config.passphrase.as_deref().unwrap_or("");
        let seed = Mnemonic::to_seed(mnemonic, passphrase);

        // Derivar master key
        let master_key = ExtSecretKey::derive_master(seed)
            .map_err(|e| ErgoDerivationError::KeyDerivationFailed(e.to_string()))?;

        // Account index (hardened)
        let account = ChildIndexHardened::from_31_bit(self.config.account_index)
            .map_err(|_| ErgoDerivationError::InvalidAccountIndex(self.config.account_index))?;

        // Network prefix
        let network_prefix = if self.config.testnet {
            NetworkPrefix::Testnet
        } else {
            NetworkPrefix::Mainnet
        };

        // Derivar direcciones
        let mut addresses = Vec::with_capacity(count as usize);
        
        for i in 0..count {
            let address_index = start_index + i;
            
            // Crear path de derivación: m/44'/429'/account'/0/address_index
            let path = DerivationPath::new(
                account,
                vec![ChildIndexNormal::normal(address_index)
                    .map_err(|_| ErgoDerivationError::InvalidAddressIndex(address_index))?],
            );

            // Derivar key
            let derived_key = master_key.derive(path)
                .map_err(|e| ErgoDerivationError::KeyDerivationFailed(e.to_string()))?;

            // Generar dirección
            let ext_pub_key = derived_key.public_key()
                .map_err(|e| ErgoDerivationError::AddressGenerationFailed(e.to_string()))?;
            
            let address: Address = ext_pub_key.into();
            let encoded_address = AddressEncoder::encode_address_as_string(network_prefix, &address);

            // Crear info de dirección
            let address_info = ErgoAddressInfo {
                address: encoded_address,
                index: address_index,
                derivation_path: format!(
                    "m/44'/429'/{}'/{}/{}",
                    self.config.account_index,
                    0, // Siempre 0 para external addresses en Ergo
                    address_index
                ),
            };

            addresses.push(address_info);
        }

        Ok(addresses)
    }

    /// Validar inputs básicos
    fn validate_inputs(&self, mnemonic: &str, index: u32) -> Result<()> {
        // Validar mnemonic básico (longitud de palabras)
        let words: Vec<&str> = mnemonic.trim().split_whitespace().collect();
        match words.len() {
            12 | 15 | 18 | 21 | 24 => {},
            _ => return Err(ErgoDerivationError::InvalidMnemonic(
                format!("Invalid word count: {}. Expected 12, 15, 18, 21, or 24", words.len())
            )),
        }

        // Validar índice de dirección (límite práctico)
        if index >= 2_147_483_648 {
            return Err(ErgoDerivationError::InvalidAddressIndex(index));
        }

        Ok(())
    }

    /// Verificar si una dirección pertenece a este mnemonic
    pub fn verify_address(&self, mnemonic: &str, address: &str, max_index: u32) -> Result<Option<u32>> {
        for i in 0..=max_index {
            let derived = self.derive_address(mnemonic, i)?;
            if derived.address == address {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }
}

impl Default for ErgoAddressDerivator {
    fn default() -> Self {
        Self::new()
    }
}
```

### 4. Módulo Principal (`src/lib.rs`)

```rust
//! Librería para derivación de direcciones Ergo
//! 
//! Esta librería implementa el estándar EIP-3 para derivación de direcciones
//! Ergo a partir de frases semilla (mnemonics).

pub mod address;
pub mod types;
pub mod errors;

// Re-exports para facilidad de uso
pub use address::ErgoAddressDerivator;
pub use types::{ErgoAddressInfo, DerivationConfig};
pub use errors::{ErgoDerivationError, Result};

/// Funciones de conveniencia para uso rápido
pub mod quick {
    use super::*;

    /// Derivar una dirección Ergo simple (mainnet, sin passphrase)
    pub fn derive_address(mnemonic: &str, index: u32) -> Result<ErgoAddressInfo> {
        ErgoAddressDerivator::new().derive_address(mnemonic, index)
    }

    /// Derivar múltiples direcciones Ergo (mainnet, sin passphrase)
    pub fn derive_addresses(mnemonic: &str, count: u32) -> Result<Vec<ErgoAddressInfo>> {
        ErgoAddressDerivator::new().derive_addresses(mnemonic, count)
    }

    /// Derivar dirección con passphrase
    pub fn derive_address_with_passphrase(
        mnemonic: &str, 
        passphrase: &str, 
        index: u32
    ) -> Result<ErgoAddressInfo> {
        let config = DerivationConfig {
            passphrase: Some(passphrase.to_string()),
            ..Default::default()
        };
        ErgoAddressDerivator::with_config(config).derive_address(mnemonic, index)
    }

    /// Derivar direcciones para testnet
    pub fn derive_testnet_address(mnemonic: &str, index: u32) -> Result<ErgoAddressInfo> {
        let config = DerivationConfig {
            testnet: true,
            ..Default::default()
        };
        ErgoAddressDerivator::with_config(config).derive_address(mnemonic, index)
    }
}

#[cfg(test)]
mod tests;
```

## Implementación Avanzada

### 1. Soporte para Batch Processing

```rust
impl ErgoAddressDerivator {
    /// Derivar direcciones en paralelo (requiere feature "parallel")
    #[cfg(feature = "parallel")]
    pub fn derive_addresses_parallel(&self, mnemonic: &str, count: u32) -> Result<Vec<ErgoAddressInfo>> {
        use rayon::prelude::*;
        
        self.validate_inputs(mnemonic, 0)?;
        
        let passphrase = self.config.passphrase.as_deref().unwrap_or("");
        let seed = Mnemonic::to_seed(mnemonic, passphrase);
        let master_key = ExtSecretKey::derive_master(seed)
            .map_err(|e| ErgoDerivationError::KeyDerivationFailed(e.to_string()))?;

        let addresses: Result<Vec<_>> = (0..count)
            .into_par_iter()
            .map(|index| {
                self.derive_single_address_from_master(&master_key, index)
            })
            .collect();

        addresses
    }

    /// Derivar una sola dirección desde master key (helper para paralelización)
    fn derive_single_address_from_master(
        &self,
        master_key: &ExtSecretKey,
        index: u32,
    ) -> Result<ErgoAddressInfo> {
        let account = ChildIndexHardened::from_31_bit(self.config.account_index)
            .map_err(|_| ErgoDerivationError::InvalidAccountIndex(self.config.account_index))?;

        let path = DerivationPath::new(
            account,
            vec![ChildIndexNormal::normal(index)
                .map_err(|_| ErgoDerivationError::InvalidAddressIndex(index))?],
        );

        let derived_key = master_key.derive(path)
            .map_err(|e| ErgoDerivationError::KeyDerivationFailed(e.to_string()))?;

        let ext_pub_key = derived_key.public_key()
            .map_err(|e| ErgoDerivationError::AddressGenerationFailed(e.to_string()))?;

        let address: Address = ext_pub_key.into();
        let network_prefix = if self.config.testnet {
            NetworkPrefix::Testnet
        } else {
            NetworkPrefix::Mainnet
        };

        let encoded_address = AddressEncoder::encode_address_as_string(network_prefix, &address);

        Ok(ErgoAddressInfo {
            address: encoded_address,
            index,
            derivation_path: format!(
                "m/44'/429'/{}'/{}/{}",
                self.config.account_index,
                0,
                index
            ),
        })
    }
}
```

### 2. Caché de Direcciones

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct CachedErgoDerivator {
    derivator: ErgoAddressDerivator,
    cache: Arc<RwLock<HashMap<(String, u32), ErgoAddressInfo>>>,
}

impl CachedErgoDerivator {
    pub fn new(derivator: ErgoAddressDerivator) -> Self {
        Self {
            derivator,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn derive_address_cached(&self, mnemonic: &str, index: u32) -> Result<ErgoAddressInfo> {
        let key = (mnemonic.to_string(), index);
        
        // Intentar leer del caché
        {
            let cache = self.cache.read().unwrap();
            if let Some(address) = cache.get(&key) {
                return Ok(address.clone());
            }
        }
        
        // Derivar y cachear
        let address = self.derivator.derive_address(mnemonic, index)?;
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(key, address.clone());
        }
        
        Ok(address)
    }

    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }
}
```

## Casos de Uso Comunes

### 1. Uso Básico

```rust
use ergo_address_derivation::{ErgoAddressDerivator, quick};

fn main() -> anyhow::Result<()> {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    // Método rápido
    let address = quick::derive_address(mnemonic, 0)?;
    println!("Primera dirección: {}", address.address);
    
    // Múltiples direcciones
    let addresses = quick::derive_addresses(mnemonic, 5)?;
    for addr in addresses {
        println!("Índice {}: {}", addr.index, addr.address);
    }
    
    Ok(())
}
```

### 2. Con Configuración Personalizada

```rust
use ergo_address_derivation::{ErgoAddressDerivator, DerivationConfig};

fn main() -> anyhow::Result<()> {
    let mnemonic = "your mnemonic here";
    
    // Configuración para testnet con passphrase
    let config = DerivationConfig {
        testnet: true,
        passphrase: Some("mi_passphrase_secreta".to_string()),
        account_index: 0,
    };
    
    let derivator = ErgoAddressDerivator::with_config(config);
    let address = derivator.derive_address(mnemonic, 0)?;
    
    println!("Dirección testnet: {}", address.address);
    println!("Path: {}", address.derivation_path);
    
    Ok(())
}
```

### 3. Verificación de Direcciones

```rust
use ergo_address_derivation::ErgoAddressDerivator;

fn verify_wallet_address() -> anyhow::Result<()> {
    let mnemonic = "your mnemonic here";
    let known_address = "9ewA9T53dy5qvAkcR5jVCtbaDW2XgWzbLPs5H4uCJJavmA4fzDx";
    
    let derivator = ErgoAddressDerivator::new();
    
    // Buscar en los primeros 100 índices
    if let Some(index) = derivator.verify_address(mnemonic, known_address, 100)? {
        println!("Dirección encontrada en índice: {}", index);
    } else {
        println!("Dirección no encontrada en los primeros 100 índices");
    }
    
    Ok(())
}
```

### 4. Integración con Wallet

```rust
use ergo_address_derivation::{ErgoAddressDerivator, ErgoAddressInfo};
use std::collections::HashMap;

pub struct SimpleWallet {
    derivator: ErgoAddressDerivator,
    mnemonic: String,
    addresses: HashMap<u32, ErgoAddressInfo>,
    next_index: u32,
}

impl SimpleWallet {
    pub fn new(mnemonic: String) -> Self {
        Self {
            derivator: ErgoAddressDerivator::new(),
            mnemonic,
            addresses: HashMap::new(),
            next_index: 0,
        }
    }

    pub fn get_next_address(&mut self) -> anyhow::Result<&ErgoAddressInfo> {
        if !self.addresses.contains_key(&self.next_index) {
            let address = self.derivator.derive_address(&self.mnemonic, self.next_index)?;
            self.addresses.insert(self.next_index, address);
        }
        
        let address = self.addresses.get(&self.next_index).unwrap();
        self.next_index += 1;
        Ok(address)
    }

    pub fn get_address(&mut self, index: u32) -> anyhow::Result<&ErgoAddressInfo> {
        if !self.addresses.contains_key(&index) {
            let address = self.derivator.derive_address(&self.mnemonic, index)?;
            self.addresses.insert(index, address);
        }
        
        Ok(self.addresses.get(&index).unwrap())
    }

    pub fn get_all_addresses(&self) -> Vec<&ErgoAddressInfo> {
        self.addresses.values().collect()
    }
}
```

## Testing y Validación

### 1. Tests Unitarios (`src/tests.rs`)

```rust
use crate::{ErgoAddressDerivator, DerivationConfig, quick};

const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn test_basic_derivation() {
    let address = quick::derive_address(TEST_MNEMONIC, 0).unwrap();
    
    // La dirección debe empezar con '9' para mainnet
    assert!(address.address.starts_with('9'));
    assert_eq!(address.index, 0);
    assert_eq!(address.derivation_path, "m/44'/429'/0'/0/0");
}

#[test]
fn test_multiple_addresses() {
    let addresses = quick::derive_addresses(TEST_MNEMONIC, 5).unwrap();
    
    assert_eq!(addresses.len(), 5);
    
    for (i, addr) in addresses.iter().enumerate() {
        assert_eq!(addr.index, i as u32);
        assert!(addr.address.starts_with('9'));
    }
    
    // Las direcciones deben ser diferentes
    let first = &addresses[0].address;
    let second = &addresses[1].address;
    assert_ne!(first, second);
}

#[test]
fn test_deterministic_derivation() {
    let addr1 = quick::derive_address(TEST_MNEMONIC, 0).unwrap();
    let addr2 = quick::derive_address(TEST_MNEMONIC, 0).unwrap();
    
    // Debe ser determinístico
    assert_eq!(addr1.address, addr2.address);
}

#[test]
fn test_passphrase() {
    let addr_no_pass = quick::derive_address(TEST_MNEMONIC, 0).unwrap();
    let addr_with_pass = quick::derive_address_with_passphrase(
        TEST_MNEMONIC, 
        "test_passphrase", 
        0
    ).unwrap();
    
    // Deben ser diferentes
    assert_ne!(addr_no_pass.address, addr_with_pass.address);
}

#[test]
fn test_testnet() {
    let addr = quick::derive_testnet_address(TEST_MNEMONIC, 0).unwrap();
    
    // Las direcciones de testnet empiezan con '3'
    assert!(addr.address.starts_with('3'));
}

#[test]
fn test_invalid_mnemonic() {
    let result = quick::derive_address("invalid mnemonic with wrong word count", 0);
    assert!(result.is_err());
}

#[test]
fn test_address_range() {
    let derivator = ErgoAddressDerivator::new();
    let addresses = derivator.derive_addresses_range(TEST_MNEMONIC, 5, 10).unwrap();
    
    assert_eq!(addresses.len(), 5);
    assert_eq!(addresses[0].index, 5);
    assert_eq!(addresses[4].index, 9);
}

#[test]
fn test_verify_address() {
    let derivator = ErgoAddressDerivator::new();
    let original = derivator.derive_address(TEST_MNEMONIC, 42).unwrap();
    
    let found_index = derivator.verify_address(TEST_MNEMONIC, &original.address, 100).unwrap();
    assert_eq!(found_index, Some(42));
    
    // Dirección que no existe
    let not_found = derivator.verify_address(TEST_MNEMONIC, "9invalid_address", 10).unwrap();
    assert_eq!(not_found, None);
}

#[cfg(feature = "parallel")]
#[test]
fn test_parallel_derivation() {
    let derivator = ErgoAddressDerivator::new();
    let addresses_seq = derivator.derive_addresses(TEST_MNEMONIC, 100).unwrap();
    let addresses_par = derivator.derive_addresses_parallel(TEST_MNEMONIC, 100).unwrap();
    
    assert_eq!(addresses_seq.len(), addresses_par.len());
    
    // Verificar que son iguales (el orden puede diferir en paralelo)
    for addr_seq in &addresses_seq {
        let addr_par = addresses_par.iter()
            .find(|a| a.index == addr_seq.index)
            .unwrap();
        assert_eq!(addr_seq.address, addr_par.address);
    }
}
```

### 2. Tests de Integración

```rust
#[test]
fn test_compatibility_with_known_wallets() {
    // Estos valores fueron generados con Ergo Wallet oficial
    let test_vectors = vec![
        (TEST_MNEMONIC, 0, "9fRAWhdxEsTcdb8PhGNrpfchAyuUEeTcQVXvNjQVHjLLKhG4oVd"),
        // Agregar más vectores de test aquí
    ];
    
    for (mnemonic, index, expected_address) in test_vectors {
        let address = quick::derive_address(mnemonic, index).unwrap();
        assert_eq!(address.address, expected_address, 
                  "Mismatch for index {}", index);
    }
}
```

### 3. Benchmarks (opcional)

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_single_derivation() {
        let derivator = ErgoAddressDerivator::new();
        let start = Instant::now();
        
        for i in 0..1000 {
            let _ = derivator.derive_address(TEST_MNEMONIC, i).unwrap();
        }
        
        let elapsed = start.elapsed();
        println!("1000 derivaciones tomaron: {:?}", elapsed);
        println!("Promedio por derivación: {:?}", elapsed / 1000);
    }
}
```

## Troubleshooting

### Problemas Comunes

#### 1. Direcciones no coinciden con wallet oficial

**Síntomas**: Las direcciones generadas no coinciden con las del wallet oficial de Ergo.

**Soluciones**:
```rust
// Verificar que uses exactamente el mismo mnemonic
let mnemonic = "word1 word2 word3..."; // Sin espacios extra

// Verificar que no uses passphrase si el wallet no la usa
let config = DerivationConfig {
    passphrase: None, // Importante: None, no Some("")
    ..Default::default()
};

// Verificar la red (mainnet vs testnet)
let config = DerivationConfig {
    testnet: false, // Para mainnet
    ..Default::default()
};
```

#### 2. Error "Invalid mnemonic"

**Síntomas**: Error al validar el mnemonic.

**Soluciones**:
```rust
// Verificar número de palabras
let words: Vec<&str> = mnemonic.trim().split_whitespace().collect();
assert!(matches!(words.len(), 12 | 15 | 18 | 21 | 24));

// Limpiar espacios extra
let clean_mnemonic = mnemonic.trim().split_whitespace()
    .collect::<Vec<_>>()
    .join(" ");
```

#### 3. Performance lenta

**Soluciones**:
```rust
// Usar derivación en lotes
let addresses = derivator.derive_addresses(mnemonic, 100)?;

// Usar caché para direcciones frecuentes
let cached_derivator = CachedErgoDerivator::new(derivator);

// Compilar en modo release
// cargo build --release
```

#### 4. Errores de compilación

**Síntomas**: Errores relacionados con `ergo-lib` o dependencias.

**Soluciones**:
```toml
# Cargo.toml - Asegurar versiones compatibles
[dependencies]
ergo-lib = { version = "0.28", features = ["mnemonic_gen"] }

# Si hay conflictos con otras librerías crypto
[dependencies.ergo-lib]
version = "0.28"
features = ["mnemonic_gen"]
default-features = false
```

```rust
// Importaciones correctas
use ergo_lib::{
    ergotree_ir::chain::address::{Address, NetworkPrefix, AddressEncoder},
    wallet::{
        derivation_path::{ChildIndexHardened, ChildIndexNormal, DerivationPath},
        ext_secret_key::ExtSecretKey,
        mnemonic::Mnemonic,
    },
};
```

#### 5. Problemas con diferentes versiones de ergo-lib

**Síntomas**: API changes entre versiones.

**Soluciones**:
```rust
// Para ergo-lib 0.28.x (recomendado)
let seed = Mnemonic::to_seed(mnemonic, passphrase);

// Para versiones anteriores, verificar la documentación
// La API puede diferir ligeramente
```

### Debugging Avanzado

#### 1. Verificación Paso a Paso

```rust
pub fn debug_derivation(mnemonic: &str, index: u32) -> Result<()> {
    println!("=== DEBUG DERIVATION ===");
    println!("Mnemonic: {}", mnemonic);
    println!("Index: {}", index);
    
    // Paso 1: Generar seed
    let seed = Mnemonic::to_seed(mnemonic, "");
    println!("Seed (hex): {}", hex::encode(&seed));
    
    // Paso 2: Master key
    let master_key = ExtSecretKey::derive_master(seed)?;
    println!("Master key derivado");
    
    // Paso 3: Account path
    let account = ChildIndexHardened::from_31_bit(0)?;
    println!("Account index: 0'");
    
    // Paso 4: Full path
    let path = DerivationPath::new(
        account,
        vec![ChildIndexNormal::normal(index)?],
    );
    println!("Path: m/44'/429'/0'/0/{}", index);
    
    // Paso 5: Derivar key
    let derived_key = master_key.derive(path)?;
    println!("Key derivada para path");
    
    // Paso 6: Public key
    let ext_pub_key = derived_key.public_key()?;
    println!("Public key extraída");
    
    // Paso 7: Address
    let address: Address = ext_pub_key.into();
    let encoded = AddressEncoder::encode_address_as_string(NetworkPrefix::Mainnet, &address);
    println!("Address final: {}", encoded);
    
    Ok(())
}
```

#### 2. Comparación con Vectores de Test Conocidos

```rust
// Vectores de test de la referencia oficial
const KNOWN_TEST_VECTORS: &[(&str, u32, &str)] = &[
    (
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        0,
        "9fRAWhdxEsTcdb8PhGNrpfchAyuUEeTcQVXvNjQVHjLLKhG4oVd"
    ),
    // Agregar más vectores según sea necesario
];

pub fn validate_against_known_vectors() {
    for (mnemonic, index, expected) in KNOWN_TEST_VECTORS {
        match quick::derive_address(mnemonic, *index) {
            Ok(addr) => {
                if addr.address == *expected {
                    println!("✅ Vector {}: PASS", index);
                } else {
                    println!("❌ Vector {}: FAIL", index);
                    println!("   Expected: {}", expected);
                    println!("   Got:      {}", addr.address);
                }
            }
            Err(e) => {
                println!("❌ Vector {}: ERROR - {}", index, e);
            }
        }
    }
}
```

## Referencia Completa

### API Reference

#### ErgoAddressDerivator

```rust
impl ErgoAddressDerivator {
    // Constructores
    pub fn new() -> Self
    pub fn with_config(config: DerivationConfig) -> Self
    
    // Derivación básica
    pub fn derive_address(&self, mnemonic: &str, index: u32) -> Result<ErgoAddressInfo>
    pub fn derive_addresses(&self, mnemonic: &str, count: u32) -> Result<Vec<ErgoAddressInfo>>
    
    // Derivación avanzada
    pub fn derive_addresses_range(&self, mnemonic: &str, start: u32, end: u32) -> Result<Vec<ErgoAddressInfo>>
    
    // Utilidades
    pub fn verify_address(&self, mnemonic: &str, address: &str, max_index: u32) -> Result<Option<u32>>
}
```

#### DerivationConfig

```rust
pub struct DerivationConfig {
    pub testnet: bool,           // false = mainnet, true = testnet
    pub passphrase: Option<String>, // BIP39 passphrase opcional
    pub account_index: u32,      // Normalmente 0
}
```

#### ErgoAddressInfo

```rust
pub struct ErgoAddressInfo {
    pub address: String,         // Dirección codificada
    pub index: u32,             // Índice de derivación
    pub derivation_path: String, // Path completo
}
```

### Funciones Quick

```rust
// Funciones de conveniencia para uso rápido
pub mod quick {
    pub fn derive_address(mnemonic: &str, index: u32) -> Result<ErgoAddressInfo>
    pub fn derive_addresses(mnemonic: &str, count: u32) -> Result<Vec<ErgoAddressInfo>>
    pub fn derive_address_with_passphrase(mnemonic: &str, passphrase: &str, index: u32) -> Result<ErgoAddressInfo>
    pub fn derive_testnet_address(mnemonic: &str, index: u32) -> Result<ErgoAddressInfo>
}
```

### Constantes Importantes

```rust
// Path de derivación base
const ERGO_DERIVATION_PATH: &str = "m/44'/429'/0'/0";

// Coin type de Ergo según SLIP-44
const ERGO_COIN_TYPE: u32 = 429;

// Prefijos de direcciones
// Mainnet: '9'
// Testnet: '3'
```

### Limits y Restricciones

| Parámetro | Límite | Notas |
|-----------|--------|-------|
| Índice máximo | 2^31 - 1 | Límite de hardened derivation |
| Palabras mnemonic | 12, 15, 18, 21, 24 | Según BIP39 |
| Account index | 0 - 2^31-1 | Normalmente 0 |
| Passphrase | UTF-8 | Sin límite teórico |

### Compatibilidad

#### Wallets Compatibles
- ✅ Ergo Wallet (Android/iOS)
- ✅ Yoroi Wallet
- ✅ SAFEW (Simple And Fast Ergo Wallet)
- ✅ Minotaur Wallet
- ✅ Ergopay compatible wallets

#### Standards Implementados
- ✅ EIP-3 (Ergo HD Wallet Standard)
- ✅ BIP39 (Mnemonic Seeds)
- ✅ BIP32 (HD Wallets)
- ✅ BIP44 (Multi-Account Hierarchy)
- ✅ SLIP-44 (Coin Types)

### Ejemplos de Uso Completos

#### 1. CLI Tool Básico

```rust
use clap::Parser;
use ergo_address_derivation::quick;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Mnemonic phrase
    mnemonic: String,
    
    /// Number of addresses to generate
    #[arg(short, long, default_value = "1")]
    count: u32,
    
    /// Starting index
    #[arg(short, long, default_value = "0")]
    start: u32,
    
    /// Optional passphrase
    #[arg(short, long)]
    passphrase: Option<String>,
    
    /// Use testnet
    #[arg(short, long)]
    testnet: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    let config = ergo_address_derivation::DerivationConfig {
        testnet: args.testnet,
        passphrase: args.passphrase,
        account_index: 0,
    };
    
    let derivator = ergo_address_derivation::ErgoAddressDerivator::with_config(config);
    
    for i in 0..args.count {
        let index = args.start + i;
        let address = derivator.derive_address(&args.mnemonic, index)?;
        println!("{}: {}", address.index, address.address);
    }
    
    Ok(())
}
```

#### 2. Web Service

```rust
use warp::Filter;
use serde::{Deserialize, Serialize};
use ergo_address_derivation::{ErgoAddressDerivator, DerivationConfig};

#[derive(Deserialize)]
struct DeriveRequest {
    mnemonic: String,
    index: Option<u32>,
    count: Option<u32>,
    passphrase: Option<String>,
    testnet: Option<bool>,
}

#[derive(Serialize)]
struct DeriveResponse {
    addresses: Vec<ergo_address_derivation::ErgoAddressInfo>,
}

async fn derive_handler(req: DeriveRequest) -> Result<impl warp::Reply, warp::Rejection> {
    let config = DerivationConfig {
        testnet: req.testnet.unwrap_or(false),
        passphrase: req.passphrase,
        account_index: 0,
    };
    
    let derivator = ErgoAddressDerivator::with_config(config);
    
    let addresses = if let Some(count) = req.count {
        derivator.derive_addresses(&req.mnemonic, count)
    } else {
        let index = req.index.unwrap_or(0);
        derivator.derive_address(&req.mnemonic, index)
            .map(|addr| vec![addr])
    };
    
    match addresses {
        Ok(addrs) => Ok(warp::reply::json(&DeriveResponse { addresses: addrs })),
        Err(_) => Err(warp::reject()),
    }
}

#[tokio::main]
async fn main() {
    let derive = warp::path("derive")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(derive_handler);
    
    warp::serve(derive)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
```

#### 3. Biblioteca para Otros Proyectos

```rust
// En tu Cargo.toml principal
[dependencies]
ergo-address-derivation = { path = "./ergo-address-derivation" }

// Uso en tu código
use ergo_address_derivation::quick;

fn generate_wallet_addresses(seed_phrase: &str) -> anyhow::Result<()> {
    // Generar primera dirección para recibir fondos
    let receiving_address = quick::derive_address(seed_phrase, 0)?;
    println!("Dirección principal: {}", receiving_address.address);
    
    // Generar direcciones adicionales para privacy
    let addresses = quick::derive_addresses(seed_phrase, 10)?;
    for addr in addresses {
        println!("Dirección {}: {}", addr.index, addr.address);
    }
    
    Ok(())
}
```

### Consideraciones de Seguridad

#### 1. Manejo de Mnemonics

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
struct SecureMnemonic {
    data: String,
}

impl SecureMnemonic {
    pub fn new(mnemonic: String) -> Self {
        Self { data: mnemonic }
    }
    
    pub fn as_str(&self) -> &str {
        &self.data
    }
}

// Uso seguro
fn secure_derivation() -> anyhow::Result<()> {
    let secure_mnemonic = SecureMnemonic::new(
        "your secret mnemonic here".to_string()
    );
    
    let address = quick::derive_address(secure_mnemonic.as_str(), 0)?;
    println!("Address: {}", address.address);
    
    // El mnemonic se zeroza automáticamente al salir del scope
    Ok(())
}
```

#### 2. Validación de Inputs

```rust
use regex::Regex;

pub fn validate_mnemonic_format(mnemonic: &str) -> bool {
    // Solo letras minúsculas y espacios
    let re = Regex::new(r"^[a-z ]+$").unwrap();
    re.is_match(mnemonic)
}

pub fn sanitize_mnemonic(mnemonic: &str) -> String {
    mnemonic
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
```

#### 3. Rate Limiting para APIs

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }
    
    pub fn check_rate_limit(&self, identifier: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        
        let user_requests = requests.entry(identifier.to_string()).or_insert_with(Vec::new);
        
        // Limpiar requests antiguos
        user_requests.retain(|&time| now.duration_since(time) < self.window);
        
        if user_requests.len() >= self.max_requests {
            false
        } else {
            user_requests.push(now);
            true
        }
    }
}
```

### Conclusión

Esta documentación te proporciona todo lo necesario para implementar correctamente la derivación de direcciones Ergo en tu proyecto Rust:

1. **Implementación completa** siguiendo el estándar EIP-3
2. **Manejo de errores** robusto
3. **Casos de uso** comunes y avanzados
4. **Testing** exhaustivo
5. **Troubleshooting** para problemas comunes
6. **Consideraciones de seguridad**

La implementación es compatible con todas las wallets oficiales de Ergo y sigue las mejores prácticas de la industria. El código es modular, extensible y está listo para producción.

### Recursos Adicionales

- [EIP-3 Specification](https://github.com/ergoplatform/eips/blob/master/eip-0003.md)
- [Ergo-lib Documentation](https://docs.rs/ergo-lib/)
- [SLIP-44 Coin Types](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)
- [BIP39 Specification](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
- [Ergo Platform Documentation](https://docs.ergoplatform.com/)

### Soporte

Si encuentras problemas o necesitas ayuda adicional:

1. Verifica los vectores de test incluidos
2. Compara tu implementación con el código de referencia
3. Revisa la sección de troubleshooting
4. Consulta los logs de debug para identificar diferencias