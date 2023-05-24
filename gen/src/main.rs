use config::Config;
use flusty_parse::rust::module::Module;
use simplelog::{
    ColorChoice, Config as LogConfig, LevelFilter, TermLogger, TerminalMode,
};

pub mod config;
pub mod conversion;
pub mod dart;

fn main() {
    TermLogger::init(
        LevelFilter::Info,
        LogConfig::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
    let config = Config::from_disk().unwrap();
    let module = Module::builder(&["flusty"])
        .path(config.rust_entry().display().to_string())
        .name("lib".to_string())
        .data()
        .expect("Failed to parse module")
        .build();
}
