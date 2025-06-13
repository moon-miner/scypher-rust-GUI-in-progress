// src/cli/display.rs - Pantallas visuales y banners

use std::io::{self, Write};

/// Versión de SCypher para mostrar en el banner
const VERSION: &str = "3.0";

/// Colores ANSI para tema amber/terminal retro
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const PRIMARY: &str = "\x1b[38;5;214m";      // Amber primary
    pub const BRIGHT: &str = "\x1b[1;38;5;220m";     // Bright amber
    pub const DIM: &str = "\x1b[38;5;172m";          // Dark orange
    pub const WARNING: &str = "\x1b[38;5;228m";      // Warm yellow
    pub const ERROR: &str = "\x1b[38;5;124m";        // Brick red
    pub const FRAME: &str = "\x1b[38;5;240m";        // Dark gray
    pub const SUCCESS: &str = "\x1b[1;32m";          // Green
}

/// Limpiar pantalla usando múltiples métodos para compatibilidad total
pub fn clear_screen() {
    // Detectar tipo de terminal para mejor compatibilidad
    let term_type = std::env::var("TERM").unwrap_or_default();
    let is_windows = cfg!(target_os = "windows");

    if is_windows {
        // En Windows, usar comando cls
        let _ = std::process::Command::new("cls").status();
    } else if term_type.contains("xterm") || term_type.contains("screen") {
        // Terminales compatibles con ANSI
        print!("\x1b[2J\x1b[H");
        io::stdout().flush().unwrap_or(());
    } else {
        // Fallback: comando clear estándar
        let _ = std::process::Command::new("clear").status();
    }

    // Fallback final: llenar con líneas vacías si los comandos fallan
    for _ in 0..3 {
        println!();
    }
}

/// Mostrar banner principal de SCypher con ASCII art
pub fn show_banner() {
    println!("{}SCypher v{}{} {}- XOR-based BIP39 Seed Cipher{}",
             colors::BRIGHT, VERSION, colors::RESET, colors::DIM, colors::RESET);
    println!("{}                        Rust Implementation{}", colors::DIM, colors::RESET);
    println!();

    // ASCII art del logo (preservado del script Bash original)
    println!("{}                                  000000000", colors::PRIMARY);
    println!("                              000000000000000000");
    println!("                            000000          000000");
    println!("                           000                  000");
    println!("                          000     0000000000     000");
    println!("                         000      0000000000      000");
    println!("                         00        0000           000");
    println!("                        000          0000          000");
    println!("                        000          0000          000");
    println!("                         000       0000            00");
    println!("                         000      0000000000      000");
    println!("                          000     0000000000     000");
    println!("                           000                  000");
    println!("                            000000          000000");
    println!("                              000000000000000000");
    println!("                                   000000000{}", colors::RESET);
    println!();
}

/// Mostrar texto de licencia y disclaimer
pub fn show_license_text() {
    let license_text = r#"
License:
This project is released under the MIT License. You are free to:
- Use the software commercially
- Modify the source code
- Distribute the software
- Use it privately

Disclaimer:
THIS SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED.
IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

The developers assume no responsibility for:
- Loss of funds or assets
- Incorrect usage of the software
- Modifications made by third parties
- Security implications of usage in specific contexts
- Possible malfunction of the software
"#;

    clear_screen();
    println!("{}", license_text);
    println!();
    print!("Press enter to continue...");
    io::stdout().flush().unwrap_or(());

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
}

/// Mostrar explicación detallada del proceso XOR
pub fn show_cipher_details() {
    let details_text = r#"
How SCypher v3.0 Works (XOR-Based Encryption):

SCypher uses XOR encryption while maintaining BIP39 compatibility through
intelligent checksum recalculation.

1. Core Concept - XOR Encryption:
   - XOR (exclusive OR) is a reversible binary operation
   - When you XOR data twice with the same key, you get back the original
   - Formula: (data XOR key) XOR key = data

2. The Process:
   Encryption/Decryption (same operation due to XOR symmetry):
   - Your seed phrase is converted to binary (11 bits per word)
   - Your password generates a keystream using Argon2id key derivation
   - The keystream can be strengthened with iterations
   - Binary seed XOR keystream = transformed binary
   - Transformed binary gets a recalculated BIP39 checksum
   - Result is converted back to valid BIP39 words

3. Security Features:
   - Argon2id provides memory-hard key derivation
   - Iterations add computational cost for attackers
   - XOR provides perfect secrecy with a strong keystream
   - Output is always a valid BIP39 phrase with correct checksum
   - Memory-secure operations with automatic cleanup

4. Checksum Handling:
   - BIP39 phrases include a checksum for error detection
   - After XOR transformation, we recalculate the checksum
   - This ensures compatibility with all BIP39-compliant wallets
   - The adjustment is deterministic and doesn't compromise security

5. Key Improvements over v2.0:
   - Rust implementation for memory safety
   - Argon2id instead of SHAKE-256 for key derivation
   - Enhanced security protections
   - Better error handling and user experience
   - Cross-platform compatibility

6. Usage Notes:
   - Always use a strong, unique password
   - More iterations = more security but slower processing
   - Test with non-critical phrases first
   - Keep secure backups of original seeds
   - Remember both password AND iteration count

Technical Note:
The XOR cipher achieves 'perfect secrecy' when the keystream is as long as the
message and cryptographically secure. Argon2id provides the secure pseudo-randomness
needed for this application while adding resistance to hardware attacks.
"#;

    clear_screen();
    println!("{}", details_text);
    println!();
    print!("Press enter to continue...");
    io::stdout().flush().unwrap_or(());

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
}

/// Mostrar ejemplos de uso
pub fn show_usage_examples() {
    clear_screen();
    println!("{}Usage Examples{}", colors::BRIGHT, colors::RESET);
    println!("{}=============={}", colors::FRAME, colors::RESET);
    println!();
    println!("{}Interactive Mode (Menu):{}", colors::PRIMARY, colors::RESET);
    println!("  ./scypher-rust                    # Shows this menu");
    println!();
    println!("{}Command Line Mode:{}", colors::PRIMARY, colors::RESET);
    println!("  ./scypher-rust -o output.txt      # Encrypt/decrypt and save to file");
    println!("  ./scypher-rust -f input.txt       # Read from file");
    println!("  ./scypher-rust -i 10 -m 262144    # Custom security parameters");
    println!();
    println!("{}Options:{}", colors::PRIMARY, colors::RESET);
    println!("  -o, --output FILE   Save output to file");
    println!("  -f, --file FILE     Read seed phrase from file");
    println!("  -i, --iterations N  Argon2id iterations (default: 5)");
    println!("  -m, --memory KB     Argon2id memory cost (default: 131072)");
    println!("  -d, --decrypt       Decryption mode (same as encrypt due to XOR)");
    println!("  -h, --help          Show help");
    println!();
    println!("{}Security Recommendations:{}", colors::WARNING, colors::RESET);
    println!("  - Use strong, unique passwords");
    println!("  - Higher iterations = more security");
    println!("  - Test with non-critical phrases first");
    println!("  - Keep secure backups");
    println!();

    print!("Press enter to continue...");
    io::stdout().flush().unwrap_or(());

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
}

/// Mostrar información de compatibilidad del sistema
pub fn show_compatibility_info() {
    clear_screen();
    println!("{}System Compatibility{}", colors::BRIGHT, colors::RESET);
    println!("{}==================={}", colors::FRAME, colors::RESET);
    println!();
    println!("{}Dependencies:{}", colors::PRIMARY, colors::RESET);
    println!("- Rust 1.70 or higher");
    println!("- Standard system libraries");
    println!();
    println!("{}Supported Platforms:{}", colors::PRIMARY, colors::RESET);
    println!("- Linux (all distributions)");
    println!("- macOS 10.15+");
    println!("- Windows 10+ (native or WSL)");
    println!("- FreeBSD and other Unix-like systems");
    println!();
    println!("{}Installation:{}", colors::PRIMARY, colors::RESET);
    println!("1. Install Rust: https://rustup.rs/");
    println!("2. Clone repository");
    println!("3. Run: cargo build --release");
    println!("4. Binary located at: target/release/scypher-rust");
    println!();
    println!("{}Security Features:{}", colors::SUCCESS, colors::RESET);
    println!("- Memory-safe operations");
    println!("- Automatic cleanup of sensitive data");
    println!("- No external network dependencies");
    println!("- Cross-platform secure random generation");
    println!();

    print!("Press enter to continue...");
    io::stdout().flush().unwrap_or(());

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
}

/// Función utilitaria para leer entrada del usuario
pub fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap_or(());

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
    input.trim().to_string()
}

/// Función utilitaria para pausar y esperar enter
pub fn wait_for_enter() {
    print!("Press enter to continue...");
    io::stdout().flush().unwrap_or(());

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
}
