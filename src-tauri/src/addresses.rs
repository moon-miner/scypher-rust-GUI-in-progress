// =============================================================================
// DERIVACIÓN DE DIRECCIONES MULTI-BLOCKCHAIN
// Implementación criptográficamente segura para Bitcoin, Ethereum, Cardano, Solana, Ergo, TRON y más
// Soporte completo para BIP39 passphrase donde oficialmente soportado
// =============================================================================

use serde::{Deserialize, Serialize};
use crate::error::{SCypherError, Result};

// Importaciones principales
use bip32::{XPrv, DerivationPath, ChildNumber};
use std::str::FromStr;

// Importaciones Ergo
use ergo_lib::{
    ergotree_ir::chain::address::{Address as ErgoAddress, NetworkPrefix, AddressEncoder},
    wallet::{
        derivation_path::{ChildIndexHardened, ChildIndexNormal, DerivationPath as ErgoDerivationPath},
        ext_secret_key::ExtSecretKey,
        mnemonic::Mnemonic as ErgoMnemonic,
    },
};

// Importaciones criptográficas
use blake2::{Blake2b, Digest as Blake2Digest};
use bech32::{ToBase32, Variant};
use ed25519_dalek::{SigningKey, VerifyingKey};
use pbkdf2::pbkdf2;
use hmac::{Hmac, Mac};
use ed25519_dalek::SigningKey as SolanaSigningKey;
use sha2::{Sha256, Sha512, Digest};
use ripemd::Ripemd160;
use tiny_keccak::{Hasher, Keccak};

// Importaciones Cardano - EMURGO CSL
use cardano_serialization_lib::{
    Bip32PrivateKey, Bip32PublicKey,
    Address as CSLAddress, BaseAddress, Credential,
    NetworkInfo, Ed25519KeyHash,
};

/// Estructura para una dirección derivada individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub address_type: String,
    pub path: String,
    pub address: String,
}

/// Configuración para cada red
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub count: u32,           // Cantidad de direcciones a generar
    pub use_passphrase: bool, // Si usar passphrase (solo para redes que lo soporten oficialmente)
}

/// Conjunto completo de direcciones para todas las redes
#[derive(Debug, Serialize, Deserialize)]
pub struct AddressSet {
    pub bitcoin: Vec<Address>,
    pub ethereum: Vec<Address>,
    pub ergo: Vec<Address>,
    pub bsc: Vec<Address>,
    pub polygon: Vec<Address>,
    pub cardano: Vec<Address>,
    pub dogecoin: Vec<Address>,
    pub litecoin: Vec<Address>,
    pub solana: Vec<Address>,
    pub tron: Vec<Address>,
}

/// Información sobre soporte de passphrase por red
pub fn network_supports_passphrase(network: &str) -> bool {
    match network {
        // Redes que oficialmente soportan BIP39 passphrase
        "bitcoin" | "ethereum" | "tron" | "litecoin" | "dogecoin" | "bsc" | "polygon" => true,
        // Ergo soporta passphrase (verificado con wallet SATERGO)
        "ergo" => true,
        // Redes que NO soportan passphrase consistentemente
        "cardano" | "solana" => false,
        _ => false,
    }
}

/// Derivar direcciones para múltiples redes desde una seed phrase
/// Ahora soporta configuración individual por red y cantidad de direcciones
pub fn derive_addresses_with_config(
    seed_phrase: &str,
    passphrase: Option<&str>,
    network_configs: std::collections::HashMap<String, NetworkConfig>,
) -> Result<AddressSet> {
    use bip39_crate::{Mnemonic, Language};

    // Parsear mnemonic BIP39
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, seed_phrase)
        .map_err(|e| SCypherError::crypto(format!("Invalid mnemonic: {}", e)))?;

    // Generar seed con passphrase opcional
    let seed = mnemonic.to_seed(passphrase.unwrap_or(""));

    // Derivar master key para redes BIP32
    let master_key = XPrv::new(&seed)
        .map_err(|e| SCypherError::crypto(format!("Master key derivation failed: {}", e)))?;

    let mut address_set = AddressSet {
        bitcoin: Vec::new(),
        ethereum: Vec::new(),
        ergo: Vec::new(),
        bsc: Vec::new(),
        polygon: Vec::new(),
        cardano: Vec::new(),
        dogecoin: Vec::new(),
        litecoin: Vec::new(),
        solana: Vec::new(),
        tron: Vec::new(),
    };

    // Derivar direcciones para cada red solicitada
    for (network, config) in network_configs {
        // Determinar si usar passphrase según soporte oficial
        let effective_passphrase = if config.use_passphrase && network_supports_passphrase(&network) {
            passphrase
        } else {
            None // No usar passphrase si la red no lo soporta oficialmente
        };

        match network.as_str() {
            "bitcoin" => {
                address_set.bitcoin = derive_bitcoin_addresses(&master_key, config.count)?;
            }
            "ethereum" => {
                address_set.ethereum = derive_ethereum_addresses(&master_key, config.count)?;
            }
            "ergo" => {
                // Ergo soporta passphrase (verificado con wallet SATERGO)
                address_set.ergo = derive_ergo_addresses(seed_phrase, effective_passphrase, config.count)?;
            }
            "bsc" => {
                address_set.bsc = derive_bsc_addresses(&master_key, config.count)?;
            }
            "polygon" => {
                address_set.polygon = derive_polygon_addresses(&master_key, config.count)?;
            }
            "cardano" => {
                // Cardano siempre usa None para passphrase (Yoroi/Daedalus no lo soportan)
                address_set.cardano = derive_cardano_addresses_official(seed_phrase, None, config.count)?;
            }
            "dogecoin" => {
                address_set.dogecoin = derive_dogecoin_addresses(&master_key, config.count)?;
            }
            "litecoin" => {
                address_set.litecoin = derive_litecoin_addresses(&master_key, config.count)?;
            }
            "solana" => {
                // Solana siempre usa None para passphrase (Phantom no lo soporta)
                address_set.solana = derive_solana_from_mnemonic_direct(seed_phrase, None, config.count)?;
            }
            "tron" => {
                address_set.tron = derive_tron_addresses(&master_key, config.count)?;
            }
            _ => return Err(SCypherError::crypto(format!("Unsupported network: {}", network))),
        }
    }

    Ok(address_set)
}

/// Función legacy para compatibilidad hacia atrás
pub fn derive_addresses(
    seed_phrase: &str,
    passphrase: Option<&str>,
    networks: &[String],
) -> Result<AddressSet> {
    // Crear configuración por defecto (3 direcciones cada red)
    let mut network_configs = std::collections::HashMap::new();
    for network in networks {
        network_configs.insert(network.clone(), NetworkConfig {
            count: 3,
            use_passphrase: true, // Será aplicado solo a redes que lo soporten
        });
    }

    derive_addresses_with_config(seed_phrase, passphrase, network_configs)
}

// =============================================================================
// IMPLEMENTACIÓN CARDANO OFICIAL - EMURGO CSL
// =============================================================================

/// Derivar direcciones Cardano usando EMURGO CSL (biblioteca oficial)
/// NOTA: Cardano (Yoroi/Daedalus) no soporta BIP39 passphrase oficialmente
fn derive_cardano_addresses_official(
    mnemonic_phrase: &str,
    _passphrase: Option<&str>, // Ignorado intencionalmente
    count: u32,
) -> Result<Vec<Address>> {
    use bip39_crate::{Mnemonic, Language};

    let mut addresses = Vec::new();

    println!("🔧 CARDANO OFICIAL - EMURGO CSL Implementation (sin passphrase)");

    // Conversión correcta de mnemonic a entropy
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
        .map_err(|e| SCypherError::crypto(format!("Invalid mnemonic: {}", e)))?;

    let entropy = mnemonic.to_entropy();
    println!("🔍 Entropy correcta: {}", hex::encode(&entropy));

    // Generar master key usando EMURGO CSL (sin passphrase para compatibilidad Yoroi/Daedalus)
    let master_key = Bip32PrivateKey::from_bip39_entropy(&entropy, &[]);
    println!("🔍 Master key generada con EMURGO CSL");

    // Derivar staking key: m/1852'/1815'/0'/2/0
    let staking_key = master_key
        .derive(harden(1852))  // purpose
        .derive(harden(1815))  // coin_type
        .derive(harden(0))     // account
        .derive(2)             // role (staking)
        .derive(0);            // index

    let staking_pub = staking_key.to_public();
    let staking_hash = staking_pub.to_raw_key().hash();
    let staking_cred = Credential::from_keyhash(&staking_hash);

    // Generar direcciones para el número solicitado
    for index in 0u32..count {
        let payment_key = master_key
            .derive(harden(1852))  // purpose
            .derive(harden(1815))  // coin_type
            .derive(harden(0))     // account
            .derive(0)             // role (external)
            .derive(index);        // index

        let payment_pub = payment_key.to_public();
        let payment_hash = payment_pub.to_raw_key().hash();
        let payment_cred = Credential::from_keyhash(&payment_hash);

        // Crear base address (payment + staking)
        let base_addr = BaseAddress::new(
            NetworkInfo::mainnet().network_id(),
            &payment_cred,
            &staking_cred
        );

        let address_str = base_addr.to_address().to_bech32(None)
            .map_err(|e| SCypherError::crypto(format!("Address encoding failed: {:?}", e)))?;

        println!("🔍 Index {} address: {}", index, address_str);

        addresses.push(Address {
            address_type: format!("Cardano Shelley (Index {})", index),
            path: format!("m/1852'/1815'/0'/0/{}", index),
            address: address_str,
        });
    }

    Ok(addresses)
}

/// Helper para hardened derivation
fn harden(index: u32) -> u32 {
    index | 0x80_00_00_00
}

// =============================================================================
// IMPLEMENTACIÓN SOLANA OFICIAL - PHANTOM COMPATIBLE
// =============================================================================

/// Derivar direcciones Solana compatible con Phantom Wallet
/// NOTA: Phantom no soporta BIP39 passphrase oficialmente
fn derive_solana_from_mnemonic_direct(
    mnemonic_phrase: &str,
    _passphrase: Option<&str>, // Ignorado intencionalmente
    count: u32,
) -> Result<Vec<Address>> {
    use bip39_crate::{Mnemonic, Language};

    let mut addresses = Vec::new();

    println!("🚀 SOLANA PHANTOM COMPATIBLE - BIP32-Ed25519 (sin passphrase)");

    // Generar seed BIP39 (exactamente como Phantom, sin passphrase)
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
        .map_err(|e| SCypherError::crypto(format!("Invalid mnemonic: {}", e)))?;

    let seed = mnemonic.to_seed("");
    println!("🔍 Seed: {} bytes", seed.len());

    for index in 0u32..count {
        let derivation_path = if index == 0 {
            "m/44'/501'/0'/0'".to_string()
        } else {
            format!("m/44'/501'/{}'/0'", index)
        };

        println!("🔍 Derivando path: {}", derivation_path);

        // Implementar derivePath(path, seed) manualmente
        let derived_key = manual_derive_path(&derivation_path, &seed)?;

        // Crear keypair Ed25519
        let signing_key = SolanaSigningKey::from_bytes(&derived_key);
        let verifying_key = signing_key.verifying_key();
        let address_str = bs58::encode(verifying_key.as_bytes()).into_string();

        println!("🔍 Index {} address: {}", index, address_str);

        addresses.push(Address {
            address_type: format!("Solana Phantom (Index {})", index),
            path: derivation_path,
            address: address_str,
        });
    }

    Ok(addresses)
}

/// Implementación manual de derivePath - Compatible con ed25519-hd-key JavaScript
fn manual_derive_path(path: &str, seed: &[u8]) -> Result<[u8; 32]> {
    // Crear master key usando "ed25519 seed" como en BIP32-Ed25519
    let mut mac = Hmac::<Sha512>::new_from_slice(b"ed25519 seed")
        .map_err(|e| SCypherError::crypto(format!("Master key HMAC failed: {}", e)))?;

    mac.update(seed);
    let master_key_data = mac.finalize().into_bytes();

    // Split master key (32 bytes left = private key, 32 bytes right = chain code)
    let mut master_private_key = [0u8; 32];
    let mut master_chain_code = [0u8; 32];
    master_private_key.copy_from_slice(&master_key_data[0..32]);
    master_chain_code.copy_from_slice(&master_key_data[32..64]);

    // Parsear path y derivar jerárquicamente
    let path_components = parse_derivation_path_simple(path)?;

    let mut current_private_key = master_private_key;
    let mut current_chain_code = master_chain_code;

    for (i, &component) in path_components.iter().enumerate() {
        println!("🔍 Derivando componente {}: 0x{:08x}", i, component);

        // Crear HMAC para derivación del componente
        let mut child_mac = Hmac::<Sha512>::new_from_slice(&current_chain_code)
            .map_err(|e| SCypherError::crypto(format!("Child derivation HMAC failed: {}", e)))?;

        // Para hardened derivation (siempre en Ed25519)
        child_mac.update(&[0x00]);
        child_mac.update(&current_private_key);
        child_mac.update(&component.to_be_bytes());

        let child_data = child_mac.finalize().into_bytes();

        // Actualizar keys para siguiente iteración
        current_private_key.copy_from_slice(&child_data[0..32]);
        current_chain_code.copy_from_slice(&child_data[32..64]);
    }

    Ok(current_private_key)
}

/// Parsear derivation path: "m/44'/501'/0'/0'" -> [0x8000002C, 0x800001F5, 0x80000000, 0x80000000]
fn parse_derivation_path_simple(path: &str) -> Result<Vec<u32>> {
    let mut components = Vec::new();

    let path_clean = path.strip_prefix("m/")
        .ok_or_else(|| SCypherError::crypto("Invalid path format".to_string()))?;

    for component in path_clean.split('/') {
        if component.is_empty() {
            continue;
        }

        let (num_str, is_hardened) = if component.ends_with('\'') {
            (component.trim_end_matches('\''), true)
        } else {
            (component, false)
        };

        let mut num: u32 = num_str.parse()
            .map_err(|e| SCypherError::crypto(format!("Invalid path component: {}", e)))?;

        if is_hardened {
            num |= 0x80000000;
        }

        components.push(num);
    }

    Ok(components)
}

// =============================================================================
// IMPLEMENTACIONES BITCOIN (SOPORTA PASSPHRASE OFICIALMENTE)
// =============================================================================

/// Derivar direcciones Bitcoin (Legacy, SegWit, Nested SegWit)
/// Bitcoin soporta BIP39 passphrase oficialmente en hardware wallets
fn derive_bitcoin_addresses(master_key: &XPrv, count: u32) -> Result<Vec<Address>> {
    use bitcoin::Network;

    let mut addresses = Vec::new();
    let secp = bitcoin::secp256k1::Secp256k1::new();

    // Generar direcciones para cada índice solicitado
    for index in 0u32..count {
        // P2PKH (Legacy) - m/44'/0'/0'/0/index
        let path = DerivationPath::from_str(&format!("m/44'/0'/0'/0/{}", index))
            .map_err(|e| SCypherError::crypto(format!("Invalid derivation path: {}", e)))?;

        let mut current_key = master_key.clone();
        for child_number in path.as_ref() {
            current_key = current_key.derive_child(*child_number)
                .map_err(|e| SCypherError::crypto(format!("Bitcoin derivation failed: {}", e)))?;
        }

        let private_key = bitcoin::PrivateKey::new(
            bitcoin::secp256k1::SecretKey::from_slice(current_key.private_key().to_bytes().as_slice())
                .map_err(|e| SCypherError::crypto(format!("Invalid private key: {}", e)))?,
            Network::Bitcoin
        );

        let public_key = private_key.public_key(&secp);

        // P2PKH (Legacy)
        let p2pkh_address = bitcoin::Address::p2pkh(&public_key, Network::Bitcoin);
        addresses.push(Address {
            address_type: format!("Legacy P2PKH (Index {})", index),
            path: format!("m/44'/0'/0'/0/{}", index),
            address: p2pkh_address.to_string(),
        });

        // Solo para el primer índice, agregar también SegWit
        if index == 0 {
            // P2WPKH (Native SegWit) - m/84'/0'/0'/0/0
            let segwit_path = DerivationPath::from_str("m/84'/0'/0'/0/0")
                .map_err(|e| SCypherError::crypto(format!("Invalid segwit path: {}", e)))?;

            let mut segwit_key = master_key.clone();
            for child_number in segwit_path.as_ref() {
                segwit_key = segwit_key.derive_child(*child_number)
                    .map_err(|e| SCypherError::crypto(format!("SegWit derivation failed: {}", e)))?;
            }

            let segwit_private = bitcoin::PrivateKey::new(
                bitcoin::secp256k1::SecretKey::from_slice(segwit_key.private_key().to_bytes().as_slice())
                    .map_err(|e| SCypherError::crypto(format!("Invalid segwit private key: {}", e)))?,
                Network::Bitcoin
            );

            let segwit_public = segwit_private.public_key(&secp);
            let p2wpkh_address = bitcoin::Address::p2wpkh(&segwit_public, Network::Bitcoin)
                .map_err(|e| SCypherError::crypto(format!("P2WPKH address creation failed: {}", e)))?;

            addresses.push(Address {
                address_type: "Native SegWit (P2WPKH)".to_string(),
                path: "m/84'/0'/0'/0/0".to_string(),
                address: p2wpkh_address.to_string(),
            });

            // P2SH-P2WPKH (Nested SegWit) - m/49'/0'/0'/0/0
            let nested_path = DerivationPath::from_str("m/49'/0'/0'/0/0")
                .map_err(|e| SCypherError::crypto(format!("Invalid nested path: {}", e)))?;

            let mut nested_key = master_key.clone();
            for child_number in nested_path.as_ref() {
                nested_key = nested_key.derive_child(*child_number)
                    .map_err(|e| SCypherError::crypto(format!("Nested SegWit derivation failed: {}", e)))?;
            }

            let nested_private = bitcoin::PrivateKey::new(
                bitcoin::secp256k1::SecretKey::from_slice(nested_key.private_key().to_bytes().as_slice())
                    .map_err(|e| SCypherError::crypto(format!("Invalid nested private key: {}", e)))?,
                Network::Bitcoin
            );

            let nested_public = nested_private.public_key(&secp);
            let p2shwpkh_address = bitcoin::Address::p2shwpkh(&nested_public, Network::Bitcoin)
                .map_err(|e| SCypherError::crypto(format!("P2SH-P2WPKH address creation failed: {}", e)))?;

            addresses.push(Address {
                address_type: "Nested SegWit (P2SH-P2WPKH)".to_string(),
                path: "m/49'/0'/0'/0/0".to_string(),
                address: p2shwpkh_address.to_string(),
            });
        }
    }

    Ok(addresses)
}

// =============================================================================
// IMPLEMENTACIONES ETHEREUM Y REDES EVM (SOPORTAN PASSPHRASE OFICIALMENTE)
// =============================================================================

/// Derivar direcciones Ethereum
/// Ethereum soporta BIP39 passphrase oficialmente en hardware wallets
fn derive_ethereum_addresses(master_key: &XPrv, count: u32) -> Result<Vec<Address>> {
    let mut addresses = Vec::new();

    for index in 0u32..count {
        // Ethereum standard - m/44'/60'/0'/0/index
        let path = DerivationPath::from_str(&format!("m/44'/60'/0'/0/{}", index))
            .map_err(|e| SCypherError::crypto(format!("Invalid Ethereum path: {}", e)))?;

        let mut current_key = master_key.clone();
        for child_number in path.as_ref() {
            current_key = current_key.derive_child(*child_number)
                .map_err(|e| SCypherError::crypto(format!("Ethereum derivation failed: {}", e)))?;
        }

        let public_key_point = current_key.public_key();
        let public_key_compressed = public_key_point.to_bytes();

        // Para Ethereum necesitamos la versión no comprimida
        let secp = secp256k1::Secp256k1::new();
        let pk = secp256k1::PublicKey::from_slice(&public_key_compressed)
            .map_err(|e| SCypherError::crypto(format!("Invalid public key: {}", e)))?;
        let uncompressed = pk.serialize_uncompressed();

        // Usar solo la parte X,Y (sin el prefijo 0x04)
        let xy_coords = &uncompressed[1..];

        // Hash con Keccak256
        let mut hasher = Keccak::v256();
        hasher.update(xy_coords);
        let mut hash = [0u8; 32];
        hasher.finalize(&mut hash);

        // Tomar los últimos 20 bytes como dirección
        let address_bytes = &hash[12..];

        // Aplicar EIP-55 checksum encoding (formato estándar de la industria)
        let address = to_eip55_checksum_address(&address_bytes);

        addresses.push(Address {
            address_type: format!("Ethereum (Index {})", index),
            path: format!("m/44'/60'/0'/0/{}", index),
            address,
        });
    }

    Ok(addresses)
}

/// Implementar EIP-55 checksum encoding para direcciones Ethereum
/// Este es el formato estándar usado por MetaMask, Phantom, Ledger, etc.
fn to_eip55_checksum_address(address_bytes: &[u8]) -> String {
    let address_hex = hex::encode(address_bytes);

    // Hash de la dirección en minúsculas (sin 0x) usando Keccak256
    let mut hasher = Keccak::v256();
    hasher.update(address_hex.as_bytes());
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);

    let hash_hex = hex::encode(hash);

    // Aplicar EIP-55: mayúscula si el dígito del hash >= 8
    let mut checksum_address = String::with_capacity(42);
    checksum_address.push_str("0x");

    for (i, c) in address_hex.chars().enumerate() {
        if c.is_ascii_digit() {
            // Los números siempre permanecen iguales
            checksum_address.push(c);
        } else {
            // Para letras a-f, usar mayúscula si el hex del hash en esa posición >= 8
            let hash_char = hash_hex.chars().nth(i).unwrap_or('0');
            if hash_char >= '8' {
                checksum_address.push(c.to_ascii_uppercase());
            } else {
                checksum_address.push(c);
            }
        }
    }

    checksum_address
}

/// BSC addresses (usa mismas direcciones que Ethereum)
/// BSC soporta BIP39 passphrase por herencia de Ethereum
fn derive_bsc_addresses(master_key: &XPrv, count: u32) -> Result<Vec<Address>> {
    let eth_addresses = derive_ethereum_addresses(master_key, count)?;
    let mut bsc_addresses = Vec::new();

    for addr in eth_addresses {
        bsc_addresses.push(Address {
            address_type: addr.address_type.replace("Ethereum", "BSC"),
            path: addr.path,
            address: addr.address,
        });
    }

    Ok(bsc_addresses)
}

/// Polygon addresses (usa mismas direcciones que Ethereum)
/// Polygon soporta BIP39 passphrase por herencia de Ethereum
fn derive_polygon_addresses(master_key: &XPrv, count: u32) -> Result<Vec<Address>> {
    let eth_addresses = derive_ethereum_addresses(master_key, count)?;
    let mut polygon_addresses = Vec::new();

    for addr in eth_addresses {
        polygon_addresses.push(Address {
            address_type: addr.address_type.replace("Ethereum", "Polygon"),
            path: addr.path,
            address: addr.address,
        });
    }

    Ok(polygon_addresses)
}

// =============================================================================
// IMPLEMENTACIÓN ERGO (USA SU PROPIO ESTÁNDAR)
// =============================================================================

/// Derivar direcciones Ergo usando ergo-lib
/// NOTA: Ergo soporta passphrase (verificado con wallet SATERGO)
fn derive_ergo_addresses(
    seed_phrase: &str,
    passphrase: Option<&str>, // Ahora SÍ usamos passphrase
    count: u32,
) -> Result<Vec<Address>> {
    let mut addresses = Vec::new();

    // Crear seed usando ergo-lib (con passphrase para compatibilidad SATERGO)
    let seed = ErgoMnemonic::to_seed(seed_phrase, passphrase.unwrap_or(""));

    // Derivar master key usando ergo-lib
    let master_key = ExtSecretKey::derive_master(seed)
        .map_err(|e| SCypherError::crypto(format!("Ergo master key derivation failed: {}", e)))?;

    // Account index 0 (hardened) - m/44'/429'/0'
    let account = ChildIndexHardened::from_31_bit(0)
        .map_err(|e| SCypherError::crypto(format!("Invalid Ergo account index: {}", e)))?;

    // Derivar direcciones para el número solicitado
    for index in 0u32..count {
        // Construir path de derivación: m/44'/429'/0'/0/index
        let path = ErgoDerivationPath::new(
            account,
            vec![ChildIndexNormal::normal(index)
                .map_err(|e| SCypherError::crypto(format!("Invalid Ergo address index {}: {}", index, e)))?],
        );

        // Derivar la key para el path dado
        let derived_key = master_key.derive(path)
            .map_err(|e| SCypherError::crypto(format!("Ergo key derivation failed for index {}: {}", index, e)))?;

        // Convertir la public key derivada a una address
        let ext_pub_key = derived_key.public_key()
            .map_err(|e| SCypherError::crypto(format!("Ergo public key extraction failed for index {}: {}", index, e)))?;

        let ergo_address: ErgoAddress = ext_pub_key.into();

        // Codificar la address con prefijo Mainnet
        let encoded_address = AddressEncoder::encode_address_as_string(
            NetworkPrefix::Mainnet,
            &ergo_address
        );

        addresses.push(Address {
            address_type: format!("Ergo P2PK (Index {})", index),
            path: format!("m/44'/429'/0'/0/{}", index),
            address: encoded_address,
        });
    }

    Ok(addresses)
}

// =============================================================================
// IMPLEMENTACIÓN TRON (SOPORTA PASSPHRASE OFICIALMENTE)
// =============================================================================

/// Derivar direcciones TRON usando BIP44 estándar
/// TRON soporta BIP39 passphrase oficialmente
/// Path: m/44'/195'/0'/0/index (195 = TRON coin type oficial)
fn derive_tron_addresses(master_key: &XPrv, count: u32) -> Result<Vec<Address>> {
    let mut addresses = Vec::new();

    println!("🔶 TRON Address Derivation - BIP44 m/44'/195'/0'/0/index");

    // Generar direcciones para el número solicitado
    for index in 0u32..count {
        // TRON BIP44 derivation path oficial
        let path_str = format!("m/44'/195'/0'/0/{}", index);
        let path = DerivationPath::from_str(&path_str)
            .map_err(|e| SCypherError::crypto(format!("Invalid TRON path {}: {}", path_str, e)))?;

        // Derivar la clave privada siguiendo el path BIP44
        let mut current_key = master_key.clone();
        for child_number in path.as_ref() {
            current_key = current_key.derive_child(*child_number)
                .map_err(|e| SCypherError::crypto(format!("TRON derivation failed at {}: {}", path_str, e)))?;
        }

        // Extraer public key en formato secp256k1
        let public_key_point = current_key.public_key();
        let public_key_compressed = public_key_point.to_bytes();

        // Convertir a formato no comprimido (requerido por TRON)
        let secp = secp256k1::Secp256k1::new();
        let pk = secp256k1::PublicKey::from_slice(&public_key_compressed)
            .map_err(|e| SCypherError::crypto(format!("Invalid TRON public key for index {}: {}", index, e)))?;

        // Serializar en formato no comprimido (65 bytes: 0x04 + 32 bytes X + 32 bytes Y)
        let uncompressed = pk.serialize_uncompressed();

        // TRON usa solo las coordenadas X,Y (64 bytes), sin el prefijo 0x04
        let xy_coords = &uncompressed[1..]; // 64 bytes

        println!("🔍 Index {} - Public key coords: {} bytes", index, xy_coords.len());

        // Aplicar Keccak256 hash (SHA3) a las coordenadas públicas
        let mut hasher = Keccak::v256();
        hasher.update(xy_coords);
        let mut keccak_hash = [0u8; 32];
        hasher.finalize(&mut keccak_hash);

        // Tomar los últimos 20 bytes del hash Keccak256
        let address_bytes = &keccak_hash[12..]; // 20 bytes

        // Agregar prefijo TRON mainnet (0x41) para formar dirección completa
        let mut tron_address = vec![0x41];
        tron_address.extend_from_slice(address_bytes);

        println!("🔍 Index {} - Address with prefix: {}", index, hex::encode(&tron_address));

        // Aplicar TRON Base58Check encoding
        let tron_address_base58 = tron_base58_encode(&tron_address)?;

        println!("🔍 Index {} - Final TRON address: {}", index, tron_address_base58);

        // Verificar que la dirección comience con 'T'
        if !tron_address_base58.starts_with('T') {
            return Err(SCypherError::crypto(format!("Invalid TRON address format for index {}: {}", index, tron_address_base58)));
        }

        addresses.push(Address {
            address_type: format!("TRON (Index {})", index),
            path: path_str,
            address: tron_address_base58,
        });
    }

    Ok(addresses)
}

/// TRON Base58Check encoding específico
/// Aplica doble SHA256 para checksum + Base58 encoding
fn tron_base58_encode(input: &[u8]) -> Result<String> {
    // Primer SHA256 del input
    let hash1 = Sha256::digest(input);

    // Segundo SHA256 del resultado anterior
    let hash2 = Sha256::digest(&hash1);

    // Tomar los primeros 4 bytes como checksum
    let checksum = &hash2[0..4];

    // Crear dirección completa: address + checksum
    let mut address_with_checksum = input.to_vec();
    address_with_checksum.extend_from_slice(checksum);

    // Codificar en Base58 estándar
    let base58_address = bs58::encode(address_with_checksum).into_string();

    Ok(base58_address)
}

// =============================================================================
// IMPLEMENTACIONES OTRAS REDES (SOPORTAN PASSPHRASE OFICIALMENTE)
// =============================================================================

/// Derivar direcciones Dogecoin
/// Dogecoin soporta BIP39 passphrase por herencia de Bitcoin
fn derive_dogecoin_addresses(master_key: &XPrv, count: u32) -> Result<Vec<Address>> {
    use bitcoin::Network;

    let mut addresses = Vec::new();

    for index in 0u32..count {
        // Dogecoin coin type: 3' - m/44'/3'/0'/0/index
        let path = DerivationPath::from_str(&format!("m/44'/3'/0'/0/{}", index))
            .map_err(|e| SCypherError::crypto(format!("Invalid Dogecoin path: {}", e)))?;

        let mut current_key = master_key.clone();
        for child_number in path.as_ref() {
            current_key = current_key.derive_child(*child_number)
                .map_err(|e| SCypherError::crypto(format!("Dogecoin derivation failed: {}", e)))?;
        }

        let secp = bitcoin::secp256k1::Secp256k1::new();
        let private_key = bitcoin::PrivateKey::new(
            bitcoin::secp256k1::SecretKey::from_slice(current_key.private_key().to_bytes().as_slice())
                .map_err(|e| SCypherError::crypto(format!("Invalid Dogecoin private key: {}", e)))?,
            Network::Bitcoin
        );

        let public_key = private_key.public_key(&secp);
        let compressed_pubkey = public_key.to_bytes();
        let sha256_hash = Sha256::digest(&compressed_pubkey);
        let ripemd_hash = Ripemd160::digest(&sha256_hash);

        // Dogecoin version byte is 0x1e (30)
        let mut address_bytes = vec![0x1e];
        address_bytes.extend_from_slice(&ripemd_hash);

        // Checksum
        let checksum_hash = Sha256::digest(&Sha256::digest(&address_bytes));
        address_bytes.extend_from_slice(&checksum_hash[0..4]);

        let dogecoin_address = bs58::encode(address_bytes).into_string();

        addresses.push(Address {
            address_type: format!("Dogecoin P2PKH (Index {})", index),
            path: format!("m/44'/3'/0'/0/{}", index),
            address: dogecoin_address,
        });
    }

    Ok(addresses)
}

/// Derivar direcciones Litecoin
/// Litecoin soporta BIP39 passphrase por herencia de Bitcoin
fn derive_litecoin_addresses(master_key: &XPrv, count: u32) -> Result<Vec<Address>> {
    use bitcoin::Network;

    let mut addresses = Vec::new();

    for index in 0u32..count {
        // Litecoin coin type: 2' - m/44'/2'/0'/0/index
        let path = DerivationPath::from_str(&format!("m/44'/2'/0'/0/{}", index))
            .map_err(|e| SCypherError::crypto(format!("Invalid Litecoin path: {}", e)))?;

        let mut current_key = master_key.clone();
        for child_number in path.as_ref() {
            current_key = current_key.derive_child(*child_number)
                .map_err(|e| SCypherError::crypto(format!("Litecoin derivation failed: {}", e)))?;
        }

        let secp = bitcoin::secp256k1::Secp256k1::new();
        let private_key = bitcoin::PrivateKey::new(
            bitcoin::secp256k1::SecretKey::from_slice(current_key.private_key().to_bytes().as_slice())
                .map_err(|e| SCypherError::crypto(format!("Invalid Litecoin private key: {}", e)))?,
            Network::Bitcoin
        );

        let public_key = private_key.public_key(&secp);
        let compressed_pubkey = public_key.to_bytes();
        let sha256_hash = Sha256::digest(&compressed_pubkey);
        let ripemd_hash = Ripemd160::digest(&sha256_hash);

        // Litecoin P2PKH version byte is 0x30 (48)
        let mut address_bytes = vec![0x30];
        address_bytes.extend_from_slice(&ripemd_hash);

        let checksum_hash = Sha256::digest(&Sha256::digest(&address_bytes));
        address_bytes.extend_from_slice(&checksum_hash[0..4]);

        let litecoin_address = bs58::encode(address_bytes).into_string();

        addresses.push(Address {
            address_type: format!("Litecoin P2PKH (Index {})", index),
            path: format!("m/44'/2'/0'/0/{}", index),
            address: litecoin_address,
        });
    }

    Ok(addresses)
}

// =============================================================================
// TESTING Y VALIDACIÓN CON TEST VECTORS OFICIALES
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use bip39_crate::{Mnemonic, Language};

    // =============================================================================
    // TEST VECTORS OFICIALES BIP39 - Mnemonic estándar de prueba
    // =============================================================================

    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // =============================================================================
    // TEST VECTORS BITCOIN - Ian Coleman BIP39 Tool
    // =============================================================================

    #[test]
    fn test_bitcoin_official_test_vectors() {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("");
        let master_key = XPrv::new(&seed).unwrap();

        let addresses = derive_bitcoin_addresses(&master_key, 1).unwrap();

        // Direcciones verificadas con Ian Coleman BIP39 tool
        let expected_legacy = "1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA";
        let expected_segwit = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";
        let expected_nested = "37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf";

        // Verificar dirección Legacy (siempre primera)
        assert_eq!(addresses[0].address, expected_legacy);

        // Verificar SegWit y Nested SegWit (solo para index 0)
        let segwit_addr = addresses.iter().find(|addr| addr.address_type.contains("Native SegWit"));
        if let Some(addr) = segwit_addr {
            assert_eq!(addr.address, expected_segwit);
        }

        let nested_addr = addresses.iter().find(|addr| addr.address_type.contains("Nested SegWit"));
        if let Some(addr) = nested_addr {
            assert_eq!(addr.address, expected_nested);
        }

        println!("✅ Bitcoin official test vectors passed:");
        println!("   Legacy:      {}", expected_legacy);
        println!("   SegWit:      {}", expected_segwit);
        println!("   Nested:      {}", expected_nested);
    }

    #[test]
    fn test_bitcoin_with_bip39_passphrase() {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("test");
        let master_key = XPrv::new(&seed).unwrap();

        let addresses = derive_bitcoin_addresses(&master_key, 1).unwrap();

        // Direcciones verificadas con Ian Coleman BIP39 tool usando passphrase "test"
        let expected_legacy = "1GG6E1WqKKhjBqtmEaKUKYefKgiDR4Wff6";
        let expected_segwit = "bc1q0h0u48k0hx0m9uhrpzpjsh4h9v2z5jhkvs9w94";
        let expected_nested = "381WJsTSJpmFW4TcSaWZhXqUL6WqBTUqYQ";

        // Verificar dirección Legacy
        assert_eq!(addresses[0].address, expected_legacy);

        // Verificar SegWit y Nested SegWit
        let segwit_addr = addresses.iter().find(|addr| addr.address_type.contains("Native SegWit"));
        if let Some(addr) = segwit_addr {
            assert_eq!(addr.address, expected_segwit);
        }

        let nested_addr = addresses.iter().find(|addr| addr.address_type.contains("Nested SegWit"));
        if let Some(addr) = nested_addr {
            assert_eq!(addr.address, expected_nested);
        }

        println!("✅ Bitcoin BIP39 passphrase test vectors passed:");
        println!("   Legacy:      {}", expected_legacy);
        println!("   SegWit:      {}", expected_segwit);
        println!("   Nested:      {}", expected_nested);
    }

    // =============================================================================
    // TEST VECTORS ETHEREUM - Ian Coleman BIP39 Tool
    // =============================================================================

    #[test]
    fn test_ethereum_official_test_vector() {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("");
        let master_key = XPrv::new(&seed).unwrap();

        let addresses = derive_ethereum_addresses(&master_key, 1).unwrap();

        // Dirección verificada con MetaMask y Phantom (formato EIP-55)
        let expected_address = "0x9858EfFD232B4033E47d90003D41EC34EcaEda94";

        assert_eq!(addresses[0].address, expected_address);
        assert_eq!(addresses[0].path, "m/44'/60'/0'/0/0");
        println!("✅ Ethereum official test vector passed: {}", addresses[0].address);
    }

    #[test]
    fn test_ethereum_with_bip39_passphrase() {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("test");
        let master_key = XPrv::new(&seed).unwrap();

        let addresses = derive_ethereum_addresses(&master_key, 1).unwrap();

        // Dirección verificada con Ian Coleman BIP39 tool usando passphrase "test"
        // Formato EIP-55 estándar compatible con todas las wallets
        let expected_address = "0xB560762fa35eFD20DF74b2cdEeB49D7A975fF99b";

        assert_eq!(addresses[0].address, expected_address);
        assert_eq!(addresses[0].path, "m/44'/60'/0'/0/0");
        println!("✅ Ethereum BIP39 passphrase test vector passed: {}", addresses[0].address);
    }

    // =============================================================================
    // TEST VECTORS TRON - Ian Coleman BIP39 Tool
    // =============================================================================

    #[test]
    fn test_tron_official_test_vector() {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("");
        let master_key = XPrv::new(&seed).unwrap();

        let addresses = derive_tron_addresses(&master_key, 1).unwrap();

        // Dirección verificada con Ian Coleman BIP39 tool
        let expected_address = "TUEZSdKsoDHQMeZwihtdoBiN46zxhGWYdH";

        assert_eq!(addresses[0].address, expected_address);
        assert_eq!(addresses[0].path, "m/44'/195'/0'/0/0");
        println!("✅ TRON official test vector passed: {}", addresses[0].address);
    }

    #[test]
    fn test_tron_with_bip39_passphrase() {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("test");
        let master_key = XPrv::new(&seed).unwrap();

        let addresses = derive_tron_addresses(&master_key, 1).unwrap();

        // Dirección verificada con Ian Coleman BIP39 tool usando passphrase "test"
        let expected_address = "THuKukbDjhaKnRNboYmZyUJjYP9jQzqtWj";

        assert_eq!(addresses[0].address, expected_address);
        assert_eq!(addresses[0].path, "m/44'/195'/0'/0/0");
        println!("✅ TRON BIP39 passphrase test vector passed: {}", addresses[0].address);
    }

    // =============================================================================
    // TEST VECTORS DOGECOIN - Ian Coleman BIP39 Tool
    // =============================================================================

    #[test]
    fn test_dogecoin_official_test_vector() {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("");
        let master_key = XPrv::new(&seed).unwrap();

        let addresses = derive_dogecoin_addresses(&master_key, 1).unwrap();

        // Dirección verificada con Ian Coleman BIP39 tool
        let expected_address = "DBus3bamQjgJULBJtYXpEzDWQRwF5iwxgC";

        assert_eq!(addresses[0].address, expected_address);
        assert_eq!(addresses[0].path, "m/44'/3'/0'/0/0");
        println!("✅ Dogecoin official test vector passed: {}", addresses[0].address);
    }

    #[test]
    fn test_dogecoin_with_bip39_passphrase() {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("test");
        let master_key = XPrv::new(&seed).unwrap();

        let addresses = derive_dogecoin_addresses(&master_key, 1).unwrap();

        // Dirección verificada con Ian Coleman BIP39 tool usando passphrase "test"
        let expected_address = "DMjZienrvG6ygQ64oDUemeaaKw3NHHjcZb";

        assert_eq!(addresses[0].address, expected_address);
        assert_eq!(addresses[0].path, "m/44'/3'/0'/0/0");
        println!("✅ Dogecoin BIP39 passphrase test vector passed: {}", addresses[0].address);
    }

    // =============================================================================
    // TEST VECTORS LITECOIN - Ian Coleman BIP39 Tool
    // =============================================================================

    #[test]
    fn test_litecoin_official_test_vector() {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("");
        let master_key = XPrv::new(&seed).unwrap();

        let addresses = derive_litecoin_addresses(&master_key, 1).unwrap();

        // Dirección verificada con Ian Coleman BIP39 tool
        let expected_address = "LUWPbpM43E2p7ZSh8cyTBEkvpHmr3cB8Ez";

        assert_eq!(addresses[0].address, expected_address);
        assert_eq!(addresses[0].path, "m/44'/2'/0'/0/0");
        println!("✅ Litecoin official test vector passed: {}", addresses[0].address);
    }

    #[test]
    fn test_litecoin_with_bip39_passphrase() {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("test");
        let master_key = XPrv::new(&seed).unwrap();

        let addresses = derive_litecoin_addresses(&master_key, 1).unwrap();

        // Dirección verificada con Ian Coleman BIP39 tool usando passphrase "test"
        let expected_address = "Lc78DL6zHtfPzsPV6WkWhCmfsFmP3MRXCd";

        assert_eq!(addresses[0].address, expected_address);
        assert_eq!(addresses[0].path, "m/44'/2'/0'/0/0");
        println!("✅ Litecoin BIP39 passphrase test vector passed: {}", addresses[0].address);
    }

    // =============================================================================
    // TEST VECTORS BSC/POLYGON - Ian Coleman BIP39 Tool (same as Ethereum)
    // =============================================================================

    #[test]
    fn test_bsc_polygon_official_test_vectors() {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("");
        let master_key = XPrv::new(&seed).unwrap();

        let bsc_addresses = derive_bsc_addresses(&master_key, 1).unwrap();
        let polygon_addresses = derive_polygon_addresses(&master_key, 1).unwrap();

        // Misma dirección que Ethereum (compatible EVM) en formato EIP-55
        let expected_address = "0x9858EfFD232B4033E47d90003D41EC34EcaEda94";

        assert_eq!(bsc_addresses[0].address, expected_address);
        assert_eq!(polygon_addresses[0].address, expected_address);

        println!("✅ BSC official test vector passed: {}", bsc_addresses[0].address);
        println!("✅ Polygon official test vector passed: {}", polygon_addresses[0].address);
    }

    #[test]
    fn test_bsc_polygon_with_bip39_passphrase() {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("test");
        let master_key = XPrv::new(&seed).unwrap();

        let bsc_addresses = derive_bsc_addresses(&master_key, 1).unwrap();
        let polygon_addresses = derive_polygon_addresses(&master_key, 1).unwrap();

        // Misma dirección que Ethereum con passphrase (compatible EVM) en formato EIP-55
        let expected_address = "0xB560762fa35eFD20DF74b2cdEeB49D7A975fF99b";

        assert_eq!(bsc_addresses[0].address, expected_address);
        assert_eq!(polygon_addresses[0].address, expected_address);

        println!("✅ BSC BIP39 passphrase test vector passed: {}", bsc_addresses[0].address);
        println!("✅ Polygon BIP39 passphrase test vector passed: {}", polygon_addresses[0].address);
    }

    // =============================================================================
    // TEST VECTORS ERGO - SATERGO Wallet
    // =============================================================================

    #[test]
    fn test_ergo_satergo_test_vectors() {
        // Test sin passphrase
        let addresses_no_pass = derive_ergo_addresses(TEST_MNEMONIC, None, 1).unwrap();
        let expected_no_pass = "9fv2n41gttbUx8oqqhexi68qPfoETFPxnLEEbTfaTk4SmY2knYC";

        assert_eq!(addresses_no_pass[0].address, expected_no_pass);
        println!("✅ Ergo SATERGO test vector (no passphrase) passed: {}", addresses_no_pass[0].address);

        // Test con passphrase "test"
        let addresses_with_pass = derive_ergo_addresses(TEST_MNEMONIC, Some("test"), 1).unwrap();
        let expected_with_pass = "9hqHAeSrCtq8p5WP8tPokBBeiC1uh6Vp42eRwvoNfaQYT1kaa6X";

        assert_eq!(addresses_with_pass[0].address, expected_with_pass);
        println!("✅ Ergo SATERGO test vector (with passphrase 'test') passed: {}", addresses_with_pass[0].address);
    }

    // =============================================================================
    // TEST VECTORS CARDANO - Eternl Wallet
    // =============================================================================

    #[test]
    fn test_cardano_eternl_test_vector() {
        let addresses = derive_cardano_addresses_official(TEST_MNEMONIC, None, 1).unwrap();

        // Dirección verificada con Eternl wallet
        let expected_address = "addr1qy8ac7qqy0vtulyl7wntmsxc6wex80gvcyjy33qffrhm7sh927ysx5sftuw0dlft05dz3c7revpf7jx0xnlcjz3g69mq4afdhv";

        assert_eq!(addresses[0].address, expected_address);
        assert_eq!(addresses[0].path, "m/1852'/1815'/0'/0/0");
        println!("✅ Cardano Eternl test vector passed: {}", addresses[0].address);
    }

    // =============================================================================
    // TEST VECTORS SOLANA - Phantom Wallet
    // =============================================================================

    #[test]
    fn test_solana_phantom_test_vector() {
        let addresses = derive_solana_from_mnemonic_direct(TEST_MNEMONIC, None, 1).unwrap();

        // Dirección verificada con Phantom wallet
        let expected_address = "HAgk14JpMQLgt6rVgv7cBQFJWFto5Dqxi472uT3DKpqk";

        assert_eq!(addresses[0].address, expected_address);
        assert_eq!(addresses[0].path, "m/44'/501'/0'/0'");
        println!("✅ Solana Phantom test vector passed: {}", addresses[0].address);
    }

    // =============================================================================
    // TESTS DE FUNCIONALIDAD GENERAL
    // =============================================================================

    #[test]
    fn test_passphrase_support_detection() {
        // Test que la detección de soporte de passphrase sea correcta
        assert!(network_supports_passphrase("bitcoin"));
        assert!(network_supports_passphrase("ethereum"));
        assert!(network_supports_passphrase("tron"));
        assert!(network_supports_passphrase("litecoin"));
        assert!(network_supports_passphrase("dogecoin"));
        assert!(network_supports_passphrase("bsc"));
        assert!(network_supports_passphrase("polygon"));
        assert!(network_supports_passphrase("ergo"));

        assert!(!network_supports_passphrase("cardano"));
        assert!(!network_supports_passphrase("solana"));

        println!("✅ Passphrase support detection test passed");
    }

    #[test]
    fn test_all_networks_standard_seed() {
        // Test integral que verifica que todas las redes generen direcciones válidas
        let mut network_configs = std::collections::HashMap::new();

        let networks = ["bitcoin", "ethereum", "tron", "litecoin", "dogecoin",
                       "bsc", "polygon", "cardano", "solana", "ergo"];

        for network in &networks {
            network_configs.insert(network.to_string(), NetworkConfig {
                count: 1,
                use_passphrase: false,
            });
        }

        let result = derive_addresses_with_config(TEST_MNEMONIC, None, network_configs).unwrap();

        // Verificar que todas las redes generaron direcciones
        assert!(!result.bitcoin.is_empty());
        assert!(!result.ethereum.is_empty());
        assert!(!result.tron.is_empty());
        assert!(!result.litecoin.is_empty());
        assert!(!result.dogecoin.is_empty());
        assert!(!result.bsc.is_empty());
        assert!(!result.polygon.is_empty());
        assert!(!result.cardano.is_empty());
        assert!(!result.solana.is_empty());
        assert!(!result.ergo.is_empty());

        println!("✅ All networks standard seed test passed");
        println!("   Bitcoin:  {}", result.bitcoin[0].address);
        println!("   Ethereum: {}", result.ethereum[0].address);
        println!("   TRON:     {}", result.tron[0].address);
        println!("   Litecoin: {}", result.litecoin[0].address);
        println!("   Dogecoin: {}", result.dogecoin[0].address);
        println!("   BSC:      {}", result.bsc[0].address);
        println!("   Polygon:  {}", result.polygon[0].address);
        println!("   Cardano:  {}...", &result.cardano[0].address[..20]);
        println!("   Solana:   {}", result.solana[0].address);
        println!("   Ergo:     {}", result.ergo[0].address);
    }

    #[test]
    fn test_multiple_addresses_generation() {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, TEST_MNEMONIC).unwrap();
        let seed = mnemonic.to_seed("");
        let master_key = XPrv::new(&seed).unwrap();

        // Test con múltiples direcciones
        let ethereum_addresses = derive_ethereum_addresses(&master_key, 5).unwrap();
        assert_eq!(ethereum_addresses.len(), 5);

        let tron_addresses = derive_tron_addresses(&master_key, 3).unwrap();
        assert_eq!(tron_addresses.len(), 3);

        // Verificar que las direcciones sean únicas
        let mut unique_addresses = std::collections::HashSet::new();
        for addr in &ethereum_addresses {
            assert!(unique_addresses.insert(&addr.address), "Duplicate address found: {}", addr.address);
        }

        println!("✅ Multiple addresses generation test passed");
        println!("   Generated {} Ethereum addresses", ethereum_addresses.len());
        println!("   Generated {} TRON addresses", tron_addresses.len());
    }

    #[test]
    fn test_passphrase_differences() {
        // Test para redes que soportan passphrase
        let mut config = std::collections::HashMap::new();
        config.insert("ethereum".to_string(), NetworkConfig { count: 1, use_passphrase: true });
        config.insert("ergo".to_string(), NetworkConfig { count: 1, use_passphrase: true });

        let result_no_pass = derive_addresses_with_config(TEST_MNEMONIC, None, config.clone()).unwrap();
        let result_with_pass = derive_addresses_with_config(TEST_MNEMONIC, Some("test"), config).unwrap();

        // Las direcciones deben ser diferentes
        assert_ne!(result_no_pass.ethereum[0].address, result_with_pass.ethereum[0].address);
        assert_ne!(result_no_pass.ergo[0].address, result_with_pass.ergo[0].address);

        println!("✅ Passphrase differences test passed");
        println!("   Ethereum without passphrase: {}", result_no_pass.ethereum[0].address);
        println!("   Ethereum with passphrase:    {}", result_with_pass.ethereum[0].address);
        println!("   Ergo without passphrase:     {}", result_no_pass.ergo[0].address);
        println!("   Ergo with passphrase:        {}", result_with_pass.ergo[0].address);
    }

    #[test]
    fn test_bip39_passphrase_comprehensive_validation() {
        // Test completo que valida todas las redes que soportan passphrase
        println!("🔐 BIP39 Passphrase Comprehensive Validation");

        // Definir las redes que soportan passphrase oficialmente
        let passphrase_networks = ["bitcoin", "ethereum", "tron", "litecoin", "dogecoin", "bsc", "polygon", "ergo"];

        for network in &passphrase_networks {
            let mut config = std::collections::HashMap::new();
            config.insert(network.to_string(), NetworkConfig {
                count: 1,
                use_passphrase: true
            });

            let result_no_pass = derive_addresses_with_config(TEST_MNEMONIC, None, config.clone()).unwrap();
            let result_with_pass = derive_addresses_with_config(TEST_MNEMONIC, Some("test"), config).unwrap();

            // Verificar que las direcciones sean diferentes con passphrase
            let addr_no_pass = match network {
                &"bitcoin" => &result_no_pass.bitcoin[0].address,
                &"ethereum" => &result_no_pass.ethereum[0].address,
                &"tron" => &result_no_pass.tron[0].address,
                &"litecoin" => &result_no_pass.litecoin[0].address,
                &"dogecoin" => &result_no_pass.dogecoin[0].address,
                &"bsc" => &result_no_pass.bsc[0].address,
                &"polygon" => &result_no_pass.polygon[0].address,
                &"ergo" => &result_no_pass.ergo[0].address,
                _ => panic!("Network not supported"),
            };

            let addr_with_pass = match network {
                &"bitcoin" => &result_with_pass.bitcoin[0].address,
                &"ethereum" => &result_with_pass.ethereum[0].address,
                &"tron" => &result_with_pass.tron[0].address,
                &"litecoin" => &result_with_pass.litecoin[0].address,
                &"dogecoin" => &result_with_pass.dogecoin[0].address,
                &"bsc" => &result_with_pass.bsc[0].address,
                &"polygon" => &result_with_pass.polygon[0].address,
                &"ergo" => &result_with_pass.ergo[0].address,
                _ => panic!("Network not supported"),
            };

            assert_ne!(addr_no_pass, addr_with_pass,
                "Network {} should generate different addresses with passphrase", network);

            println!("   ✅ {} - Passphrase creates different addresses", network.to_uppercase());
        }

        println!("✅ BIP39 Passphrase comprehensive validation passed");
    }
}
