// =============================================================================
// DERIVACIÓN DE DIRECCIONES MULTI-BLOCKCHAIN
// Implementación criptográficamente segura para Bitcoin, Ethereum, Cardano, Solana, Ergo y más
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
}

/// Derivar direcciones para múltiples redes desde una seed phrase
pub fn derive_addresses(
    seed_phrase: &str,
    passphrase: Option<&str>,
    networks: &[String],
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
    };

    // Derivar direcciones para cada red solicitada
    for network in networks {
        match network.as_str() {
            "bitcoin" => {
                address_set.bitcoin = derive_bitcoin_addresses(&master_key)?;
            }
            "ethereum" => {
                address_set.ethereum = derive_ethereum_addresses(&master_key)?;
            }
            "ergo" => {
                address_set.ergo = derive_ergo_addresses(seed_phrase, passphrase)?;
            }
            "bsc" => {
                address_set.bsc = derive_bsc_addresses(&master_key)?;
            }
            "polygon" => {
                address_set.polygon = derive_polygon_addresses(&master_key)?;
            }
            "cardano" => {
                address_set.cardano = derive_cardano_addresses_official(seed_phrase, passphrase)?;
            }
            "dogecoin" => {
                address_set.dogecoin = derive_dogecoin_addresses(&master_key)?;
            }
            "litecoin" => {
                address_set.litecoin = derive_litecoin_addresses(&master_key)?;
            }
            "solana" => {
                address_set.solana = derive_solana_from_mnemonic_direct(seed_phrase, passphrase)?;
            }
            _ => return Err(SCypherError::crypto(format!("Unsupported network: {}", network))),
        }
    }

    Ok(address_set)
}

// =============================================================================
// IMPLEMENTACIÓN CARDANO OFICIAL - EMURGO CSL
// =============================================================================

/// Derivar direcciones Cardano usando EMURGO CSL (biblioteca oficial)
fn derive_cardano_addresses_official(
    mnemonic_phrase: &str,
    passphrase: Option<&str>,
) -> Result<Vec<Address>> {
    use bip39_crate::{Mnemonic, Language};

    let mut addresses = Vec::new();
    let passphrase_str = passphrase.unwrap_or("");

    println!("🔧 CARDANO OFICIAL - EMURGO CSL Implementation");

    // Conversión correcta de mnemonic a entropy
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
        .map_err(|e| SCypherError::crypto(format!("Invalid mnemonic: {}", e)))?;

    let entropy = mnemonic.to_entropy();
    println!("🔍 Entropy correcta: {}", hex::encode(&entropy));

    // Generar master key usando EMURGO CSL
    let master_key = Bip32PrivateKey::from_bip39_entropy(&entropy, passphrase_str.as_bytes());
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

    // Generar direcciones para índices 0, 1, 2
    for index in 0u32..3u32 {
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
/// Implementa BIP32-Ed25519 exactamente como JavaScript ed25519-hd-key
fn derive_solana_from_mnemonic_direct(
    mnemonic_phrase: &str,
    passphrase: Option<&str>,
) -> Result<Vec<Address>> {
    use bip39_crate::{Mnemonic, Language};

    let mut addresses = Vec::new();

    println!("🚀 SOLANA PHANTOM COMPATIBLE - BIP32-Ed25519");

    // Generar seed BIP39 (exactamente como Phantom)
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
        .map_err(|e| SCypherError::crypto(format!("Invalid mnemonic: {}", e)))?;

    let seed = mnemonic.to_seed(passphrase.unwrap_or(""));
    println!("🔍 Seed: {} bytes", seed.len());

    for index in 0u32..3u32 {
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
// IMPLEMENTACIONES BITCOIN
// =============================================================================

/// Derivar direcciones Bitcoin (Legacy, SegWit, Nested SegWit)
fn derive_bitcoin_addresses(master_key: &XPrv) -> Result<Vec<Address>> {
    use bitcoin::Network;

    let mut addresses = Vec::new();
    let secp = bitcoin::secp256k1::Secp256k1::new();

    // P2PKH (Legacy) - m/44'/0'/0'/0/0
    let path = DerivationPath::from_str("m/44'/0'/0'/0/0")
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
        address_type: "Legacy (P2PKH)".to_string(),
        path: "m/44'/0'/0'/0/0".to_string(),
        address: p2pkh_address.to_string(),
    });

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

    Ok(addresses)
}

// =============================================================================
// IMPLEMENTACIONES ETHEREUM Y REDES EVM
// =============================================================================

/// Derivar direcciones Ethereum
fn derive_ethereum_addresses(master_key: &XPrv) -> Result<Vec<Address>> {
    use tiny_keccak::{Hasher, Keccak};

    let mut addresses = Vec::new();

    // Ethereum standard - m/44'/60'/0'/0/0
    let path = DerivationPath::from_str("m/44'/60'/0'/0/0")
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
    let address = format!("0x{}", hex::encode(address_bytes));

    addresses.push(Address {
        address_type: "Ethereum (Standard)".to_string(),
        path: "m/44'/60'/0'/0/0".to_string(),
        address,
    });

    // Dirección adicional para m/44'/60'/0'/0/1
    let path_1 = DerivationPath::from_str("m/44'/60'/0'/0/1")
        .map_err(|e| SCypherError::crypto(format!("Invalid Ethereum path 1: {}", e)))?;

    let mut current_key_1 = master_key.clone();
    for child_number in path_1.as_ref() {
        current_key_1 = current_key_1.derive_child(*child_number)
            .map_err(|e| SCypherError::crypto(format!("Ethereum derivation 1 failed: {}", e)))?;
    }

    let public_key_1 = current_key_1.public_key();
    let public_key_compressed_1 = public_key_1.to_bytes();

    let pk_1 = secp256k1::PublicKey::from_slice(&public_key_compressed_1)
        .map_err(|e| SCypherError::crypto(format!("Invalid public key 1: {}", e)))?;
    let uncompressed_1 = pk_1.serialize_uncompressed();
    let xy_coords_1 = &uncompressed_1[1..];

    let mut hasher_1 = Keccak::v256();
    hasher_1.update(xy_coords_1);
    let mut hash_1 = [0u8; 32];
    hasher_1.finalize(&mut hash_1);

    let address_bytes_1 = &hash_1[12..];
    let address_1 = format!("0x{}", hex::encode(address_bytes_1));

    addresses.push(Address {
        address_type: "Ethereum (Index 1)".to_string(),
        path: "m/44'/60'/0'/0/1".to_string(),
        address: address_1,
    });

    Ok(addresses)
}

/// BSC addresses (usa mismas direcciones que Ethereum)
fn derive_bsc_addresses(master_key: &XPrv) -> Result<Vec<Address>> {
    let eth_addresses = derive_ethereum_addresses(master_key)?;
    let mut bsc_addresses = Vec::new();

    for addr in eth_addresses {
        bsc_addresses.push(Address {
            address_type: format!("BSC {}", addr.address_type.replace("Ethereum", "")),
            path: addr.path,
            address: addr.address,
        });
    }

    Ok(bsc_addresses)
}

/// Polygon addresses (usa mismas direcciones que Ethereum)
fn derive_polygon_addresses(master_key: &XPrv) -> Result<Vec<Address>> {
    let eth_addresses = derive_ethereum_addresses(master_key)?;
    let mut polygon_addresses = Vec::new();

    for addr in eth_addresses {
        polygon_addresses.push(Address {
            address_type: format!("Polygon {}", addr.address_type.replace("Ethereum", "")),
            path: addr.path,
            address: addr.address,
        });
    }

    Ok(polygon_addresses)
}

// =============================================================================
// IMPLEMENTACIÓN ERGO
// =============================================================================

/// Derivar direcciones Ergo usando ergo-lib
fn derive_ergo_addresses(
    seed_phrase: &str,
    passphrase: Option<&str>,
) -> Result<Vec<Address>> {
    let mut addresses = Vec::new();

    // Crear seed usando ergo-lib
    let seed = ErgoMnemonic::to_seed(seed_phrase, passphrase.unwrap_or(""));

    // Derivar master key usando ergo-lib
    let master_key = ExtSecretKey::derive_master(seed)
        .map_err(|e| SCypherError::crypto(format!("Ergo master key derivation failed: {}", e)))?;

    // Account index 0 (hardened) - m/44'/429'/0'
    let account = ChildIndexHardened::from_31_bit(0)
        .map_err(|e| SCypherError::crypto(format!("Invalid Ergo account index: {}", e)))?;

    // Derivar las primeras 3 direcciones (índices 0, 1, 2)
    for index in 0u32..3u32 {
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
// IMPLEMENTACIONES OTRAS REDES
// =============================================================================

/// Derivar direcciones Dogecoin
fn derive_dogecoin_addresses(master_key: &XPrv) -> Result<Vec<Address>> {
    use bitcoin::Network;

    let mut addresses = Vec::new();

    // Dogecoin coin type: 3' - m/44'/3'/0'/0/0
    let path = DerivationPath::from_str("m/44'/3'/0'/0/0")
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
        address_type: "Dogecoin P2PKH".to_string(),
        path: "m/44'/3'/0'/0/0".to_string(),
        address: dogecoin_address,
    });

    Ok(addresses)
}

/// Derivar direcciones Litecoin
fn derive_litecoin_addresses(master_key: &XPrv) -> Result<Vec<Address>> {
    use bitcoin::Network;

    let mut addresses = Vec::new();

    // Litecoin coin type: 2' - m/44'/2'/0'/0/0
    let path = DerivationPath::from_str("m/44'/2'/0'/0/0")
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
        address_type: "Litecoin P2PKH".to_string(),
        path: "m/44'/2'/0'/0/0".to_string(),
        address: litecoin_address,
    });

    Ok(addresses)
}
