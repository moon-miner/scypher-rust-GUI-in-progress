# Derivación de Direcciones HD en Cardano y Solana (Rust)

## Introducción

Este documento detalla el proceso completo de derivación de direcciones desde una frase semilla para las redes **Cardano** y **Solana**, con enfoque en implementaciones en **Rust**. Se cubren los estándares relevantes (BIP39, BIP32, BIP44, CIP-1852, SLIP-0010), curvas criptográficas utilizadas, rutas de derivación, formatos de dirección y librerías recomendadas para cada caso.

---

## Cardano

### Esquema general

- Usa **Ed25519-BIP32** (una variante adaptada de BIP32 para Ed25519)
- Derivación basada en **CIP-1852** (propósito 1852')
- Frase semilla BIP39 -> seed -> clave maestra usando HMAC-SHA512 con tag: `"ed25519 cardano seed"`

### Ruta de derivación

```
m / 1852' / 1815' / account' / role / index
```

- 1852' = CIP-1852 (Shelley wallet)
- 1815' = Coin type de ADA (SLIP-44)
- account' = cuenta (hardened)
- role = 0 (pago), 1 (cambio), 2 (staking)
- index = número de dirección dentro del rol

### Construcción de dirección

- Hash de pubkey de pago (Blake2b-224)
- Hash de pubkey de staking (Blake2b-224)
- Header indicando tipo de dirección y red
- Codificación en Bech32 con prefijo `addr` (mainnet) o `addr_test`

### Librerías recomendadas en Rust

- [`ed25519-bip32`](https://crates.io/crates/ed25519-bip32)
- [`bip39`](https://crates.io/crates/bip39)
- [`blake2`](https://crates.io/crates/blake2)
- [`bech32`](https://crates.io/crates/bech32)

### Recomendaciones

- Usar derivación Shelley (CIP-1852), no Byron (legacy)
- Implementar correctamente el clamping de Ed25519
- Asegurar Blake2b-224 para hashing

---

## Solana

### Esquema general

- Usa **SLIP-0010** derivación Ed25519 (sólo hardened)
- Frase semilla BIP39 -> seed -> clave maestra usando HMAC-SHA512 con tag: `"ed25519 seed"`

### Ruta de derivación más usada

```
m / 44' / 501' / 0' / 0'
```

- 44' = BIP44 purpose
- 501' = Coin type Solana (SLIP-44)
- 0' = cuenta principal
- 0' = derivación adicional (por convención)

> *Phantom, Sollet y la mayoría de wallets usan esta ruta.*

### Construcción de dirección

- La dirección es la **clave pública Ed25519 (32 bytes)**
- Codificada en **Base58**

### Librerías recomendadas en Rust

- [`slip10`](https://docs.rs/slip10)
- [`ed25519-dalek`](https://crates.io/crates/ed25519-dalek)
- [`bs58`](https://crates.io/crates/bs58)
- [`bip39`](https://crates.io/crates/bip39)

### Recomendaciones

- Seguir la ruta Phantom por defecto para compatibilidad
- No implementar derivación no-hardened (no es soportada en Solana)
- Usar ed25519-dalek para generación de clave pública

---

## Recursos clave

### Cardano

- [CIP-1852](https://cips.cardano.org/cips/cip1852/)
- [CIP-003](https://cips.cardano.org/cips/cip3/)
- [CIP-19 - Address format](https://cips.cardano.org/cips/cip19/)
- [IOHK Adrestia](https://github.com/input-output-hk/adrestia)

### Solana

- [Solana Cookbook - Mnemonic Derivation](https://solanacookbook.com/references/keypairs.html#deriving-a-keypair-from-a-mnemonic)
- [Solana BIP44 paths](https://github.com/solana-labs/solana-keygen/blob/master/src/cli.rs)
- [SLIP-0010](https://github.com/satoshilabs/slips/blob/master/slip-0010.md)
- [SLIP-0044](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)

---

## Conclusión

Con esta información, Claude puede auditar o implementar correctamente la derivación de direcciones HD en Rust tanto para Cardano como para Solana. Se cubren los detalles críticos del proceso, las diferencias entre implementaciones, y se apuntan los recursos oficiales y crates recomendados.

> Si se necesita soporte para otras redes, esta estructura puede reutilizarse adaptando los valores de coin type, curva, derivación y formato de dirección.

