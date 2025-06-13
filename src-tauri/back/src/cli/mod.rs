// src/cli/mod.rs - Módulo CLI principal

pub mod input;
pub mod output;
pub mod display;
pub mod menu;

// Re-exportar funciones principales para fácil acceso
pub use input::{
    read_seed_interactive,
    read_seed_from_file,
    read_password_secure,
};

pub use output::{
    output_result,
    save_to_file,
};

pub use menu::{
    run_interactive_menu,
    handle_post_processing_menu,
    handle_menu_error,
    MenuState,
};

pub use display::{
    clear_screen,
    show_banner,
    colors,
};
