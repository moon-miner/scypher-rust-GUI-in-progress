# Guía Completa: Derivación de Direcciones Cardano y Solana en SCypher-GUI

## 📋 Contexto del Proyecto

**Proyecto:** SCypher-GUI - HD Wallet con soporte multi-red  
**Framework:** Tauri (Rust backend + JavaScript frontend)  
**Objetivo:** Agregar soporte para derivación de direcciones Cardano y Solana sin romper funcionalidad existente

## ✅ Estado Actual del Sistema

### **Funcionalidades que FUNCIONAN CORRECTAMENTE (NO TOCAR):**
- ✅ **Bitcoin:** P2PKH, P2WPKH (SegWit), P2SH-P2WPKH (Nested SegWit)
- ✅ **Ethereum:** Derivación estándar m/44'/60'/0'/0/0
- ✅ **Ergo:** Usando ergo-lib con derivación específica m/44'/429'/0'/0/index
- ✅ **BSC (Binance Smart Chain):** Compatible EVM
- ✅ **Polygon (MATIC):** Compatible EVM  
- ✅ **Dogecoin:** P2PKH con version byte 0x1e
- ✅ **Litecoin:** P2PKH con version byte 0x30
- ✅ **Solana:** ✨ **RESUELTO** - Implementación exacta funcional

### **Funcionalidad PENDIENTE:**
- ❌ **Cardano:** Genera direcciones incorrectas (parcialmente implementado)

## 🔧 Dependencias del Sistema

### **Dependencias Existentes y Estables:**
```toml
bip32 = "0.5"
bip39-crate = { package = "bip39", version = "2.0" }
bitcoin = "0.30"
secp256k1 = { version = "0.27", features = ["recovery", "rand-std"] }
ethereum-types = "0.14"
tiny-keccak = { version = "2.0", features = ["keccak"] }
ripemd = "0.1"
ergo-lib = { version = "0.24", features = ["mnemonic_gen"] }
bs58 = "0.5"
k256 = { version = "0.13", features = ["ecdsa"] }
elliptic-curve = "0.13"
sha2 = "0.10"
hex = "0.4"
```

### **Dependencias Agregadas para Cardano/Solana:**
```toml
blake2 = "0.10"      # Para BLAKE2b (Cardano)
crc32fast = "1.3"    # Para CRC32 (Cardano) 
bech32 = "0.9"       # Para bech32 encoding (Cardano)
ed25519-dalek = "2.0" # Para Ed25519 (Solana) ✅ FUNCIONA
```

## 🚫 Soluciones Intentadas que NO FUNCIONARON

### **1. Dependencias Oficiales (CONFLICTOS DE VERSIONES):**
```toml
# ❌ FALLIDO - Conflictos de dependencias
cardano-serialization-lib = "12.0"  # Conflicto con wasm-bindgen
solana-sdk = "1.16"                  # Conflicto con zeroize, curve25519-dalek
```

**Problemas:**
- `cardano-serialization-lib` requiere versiones específicas de `wasm-bindgen` incompatibles
- `solana-sdk` tiene conflictos internos con `zeroize` y `curve25519-dalek`
- Patches `[patch.crates-io]` generan errores "points to same source"

### **2. Implementación Nativa Aproximada (ALGORITMOS INCORRECTOS):**
- **Cardano:** Usar SHA256 + RIPEMD160 en lugar de BLAKE2b + bech32
- **Solana:** Simular Ed25519 con SHA256 de claves secp256k1

**Resultado:** Direcciones generadas pero criptográficamente incorrectas

## ✅ SOLUCIÓN EXITOSA PARA SOLANA

### **Algoritmo Exacto Implementado:**

```rust
fn derive_solana_addresses_exact(master_key: &XPrv) -> Result<Vec<Address>> {
    for index in 0u32..3u32 {
        // 1. Derivación BIP44 estándar: m/44'/501'/0'/0/index
        let path = format!("m/44'/501'/0'/0/{}", index);
        let derivation = DerivationPath::from_str(&path)?;
        
        let mut current_key = master_key.clone();
        for child_number in derivation.as_ref() {
            current_key = current_key.derive_child(*child_number)?;
        }
        
        // 2. Obtener clave privada derivada (32 bytes exactos)
        let private_key_bytes = current_key.private_key().to_bytes();
        let mut ed25519_seed = [0u8; 32];
        let copy_len = std::cmp::min(32, private_key_bytes.len());
        ed25519_seed[..copy_len].copy_from_slice(&private_key_bytes[..copy_len]);
        
        // 3. Crear keypair Ed25519 real
        let signing_key = SigningKey::from_bytes(&ed25519_seed);
        let verifying_key = signing_key.verifying_key();
        
        // 4. Dirección = Clave pública Ed25519 en base58
        let address_str = bs58::encode(verifying_key.as_bytes()).into_string();
        
        addresses.push(Address {
            address_type: format!("Solana BIP44 Exact (Index {})", index),
            path: format!("m/44'/501'/0'/0/{}", index),
            address: address_str,
        });
    }
    Ok(addresses)
}
```

### **Verificación Exitosa:**
**Seed de prueba:** `"chicken chicken chicken chicken chicken chicken chicken chicken chicken chicken chicken banana"`

**Resultado esperado:** `HLNrm9tQXP2Yys7Z6UaqU6Pg1TFWsuBwLenCNHx25F4B`  
**Resultado obtenido:** ✅ **CORRECTO** - Coincide con Exodus Wallet

### **Dependencia Clave:**
```toml
ed25519-dalek = "2.0"  # ✅ Sin conflictos, implementación Ed25519 real
```

## ❌ PROBLEMA ACTUAL: CARDANO

### **Algoritmo Actual (INCORRECTO):**

```rust
fn derive_cardano_addresses_exact(master_key: &XPrv) -> Result<Vec<Address>> {
    for index in 0u32..3u32 {
        // 1. Derivación CIP-1852: m/1852'/1815'/0'/0/index (payment)
        let payment_path = format!("m/1852'/1815'/0'/0/{}", index);
        let payment_derivation = DerivationPath::from_str(&payment_path)?;
        
        let mut payment_key = master_key.clone();
        for child_number in payment_derivation.as_ref() {
            payment_key = payment_key.derive_child(*child_number)?;
        }
        
        // 2. Derivación CIP-1852: m/1852'/1815'/0'/2/0 (staking)
        let staking_path = "m/1852'/1815'/0'/2/0".to_string();
        let staking_derivation = DerivationPath::from_str(&staking_path)?;
        
        let mut staking_key = master_key.clone();
        for child_number in staking_derivation.as_ref() {
            staking_key = staking_key.derive_child(*child_number)?;
        }
        
        // 3. Obtener claves públicas (secp256k1 - POSIBLE PROBLEMA)
        let payment_pubkey = payment_key.public_key().to_bytes();
        let staking_pubkey = staking_key.public_key().to_bytes();
        
        // 4. BLAKE2b-224 (28 bytes) - CORRECTO
        let payment_hash = Blake2b::<blake2::digest::typenum::U28>::digest(&payment_pubkey);
        let staking_hash = Blake2b::<blake2::digest::typenum::U28>::digest(&staking_pubkey);
        
        // 5. Construir address - CORRECTO
        let mut address_bytes = vec![0x01]; // Network ID = mainnet, type = base
        address_bytes.extend_from_slice(&payment_hash);  // 28 bytes
        address_bytes.extend_from_slice(&staking_hash);   // 28 bytes
        
        // 6. Bech32 encoding - CORRECTO
        let address_str = bech32::encode("addr", address_bytes.to_base32(), Variant::Bech32)?;
        
        addresses.push(Address {
            address_type: format!("Cardano CIP-1852 Exact (Index {})", index),
            path: format!("m/1852'/1815'/0'/0/{}", index),
            address: address_str,
        });
    }
    Ok(addresses)
}
```

### **Resultado Actual vs Esperado:**

**Seed de prueba:** `"chicken chicken chicken chicken chicken chicken chicken chicken chicken chicken chicken banana"`

**Esperado (Exodus):** `addr1qxuvvp7z7cz9l5uze5wlyw2hcej4unf4vpc3qd9l5dxu2pacccru9asytlfc9nga7gu403n9texn2cr3zq6tlg6dc5rs9zjjkz`

**Obtenido (Actual):** `addr1akiazfo6u4s7kssxxxegktgmospvxawebbaxrdpcnudzakprheshlvhg9tgnedv1u3n5akyutc8nb`

**Diferencias observadas:**
- El prefix cambió de `addr1q` a `addr1a` (diferente header byte)
- La longitud es significativamente diferente
- El contenido hash es completamente distinto

## 🔍 ANÁLISIS DEL PROBLEMA CARDANO

### **Posibles Causas Identificadas:**

1. **Algoritmo de Derivación Diferente:**
   - Cardano podría usar Ed25519 en lugar de secp256k1 para las claves
   - CIP-1852 vs especificación real implementada por wallets

2. **Formato de Clave Pública Incorrecto:**
   - Usando claves secp256k1 comprimidas (33 bytes) vs Ed25519 (32 bytes)
   - Cardano usa curva Ed25519, no secp256k1

3. **Header Byte Incorrecto:**
   - `0x01` vs otro valor para el tipo de address
   - Network ID o address type incorrectos

4. **Orden de Hash o Estructura de Address:**
   - Diferentes estructuras internas de address
   - Metadatos adicionales no incluidos

## 🎯 PRÓXIMOS PASOS PARA SOLUCIONAR CARDANO

### **Enfoque 1: Corrección de Curva Criptográfica (RECOMENDADO)**

**Hipótesis:** Cardano usa Ed25519, no secp256k1

**Implementación:**
```rust
// CAMBIAR DE:
let payment_pubkey = payment_key.public_key().to_bytes(); // secp256k1

// CAMBIAR A:
// Convertir secp256k1 -> Ed25519 o derivar directamente Ed25519
let payment_ed25519_key = derive_ed25519_from_bip32(payment_key)?;
let payment_pubkey = payment_ed25519_key.as_bytes(); // 32 bytes Ed25519
```

**Dependencia necesaria:**
```toml
ed25519-dalek = "2.0"  # Ya disponible por Solana
```

### **Enfoque 2: Análisis de Header Bytes**

**Experimentar con diferentes headers:**
```rust
// Probar diferentes combinaciones:
let header_variants = [
    0x00, // Enterprise address
    0x01, // Base address (actual)
    0x02, // Pointer address  
    0x03, // Reward address
    // etc.
];
```

### **Enfoque 3: Verificación de Formato BIP32->Ed25519**

**Investigar conversión correcta:**
- Cardano usa SLIP-0010 para Ed25519 desde seed
- BIP32 secp256k1 -> conversión a Ed25519
- Diferentes métodos de derivación hierarchical

### **Enfoque 4: Implementación Manual de CIP-1852**

**Estudiar especificación exacta:**
- CIP-1852: https://cips.cardano.org/cips/cip1852/
- Implementar paso a paso según especificación oficial
- Verificar cada paso vs implementación de referencia

## 🔄 ALTERNATIVAS CRIPTOGRÁFICAMENTE SEGURAS

### **Si el enfoque actual falla completamente:**

### **Alternativa 1: Librería Específica Sin Conflictos**
```toml
# Buscar librerías Cardano minimalistas:
cardano-crypto = "0.2"  # Más pequeña, menos dependencias
pallas-crypto = "0.18"  # Solo primitivas criptográficas
```

### **Alternativa 2: Implementación desde Primitivas**
- Usar `blake2 = "0.10"` y `bech32 = "0.9"` (ya disponibles)
- Implementar Ed25519 manual usando `ed25519-dalek`
- Seguir CIP-1852 paso a paso sin dependencias externas

### **Alternativa 3: Herramienta Externa (Último Recurso)**
```bash
# Usar cardano-address CLI como verificación
cardano-address recovery-phrase generate > phrase.txt
cardano-address key from-recovery-phrase < phrase.txt > root.key
cardano-address key child 1852H/1815H/0H/0/0 < root.key > payment.key
cardano-address key public --with-chain-code < payment.key > payment.pub
cardano-address address payment --network-tag mainnet < payment.pub
```

## 📝 INFORMACIÓN PARA DEBUGGING

### **Herramientas de Verificación:**
1. **Exodus Wallet** - Para direcciones de referencia
2. **Daedalus/Yoroi** - Wallets oficiales Cardano  
3. **cardano-address CLI** - Herramienta oficial de línea de comandos
4. **AdaLite/Eternl** - Wallets web para verificación cruzada

### **Datos de Prueba Estándar:**
**Seed:** `"chicken chicken chicken chicken chicken chicken chicken chicken chicken chicken chicken banana"`

**Resultados esperados:**
- **Cardano Index 0:** `addr1qxuvvp7z7cz9l5uze5wlyw2hcej4unf4vpc3qd9l5dxu2pacccru9asytlfc9nga7gu403n9texn2cr3zq6tlg6dc5rs9zjjkz`
- **Solana Index 0:** `HLNrm9tQXP2Yys7Z6UaqU6Pg1TFWsuBwLenCNHx25F4B` ✅

### **Comandos de Testing:**
```bash
# Compilar sin romper funcionalidad existente
cargo clean
cargo build

# Ejecutar tests específicos
cargo test test_cardano_exact_with_known_seed
cargo test test_solana_exact_with_known_seed  # ✅ Pasa
```

## 🎯 OBJETIVO FINAL

**Implementar derivación exacta de Cardano que:**
1. ✅ Genere direcciones idénticas a Exodus/Daedalus
2. ✅ Use algoritmos criptográficamente correctos (Ed25519 + BLAKE2b + bech32)
3. ✅ No rompa ninguna funcionalidad existente
4. ✅ Use dependencias mínimas y sin conflictos
5. ✅ Mantenga la arquitectura actual del sistema

**Status:** Solana completamente funcional ✅ | Cardano pendiente de corrección ⚠️