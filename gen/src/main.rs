use config::Config;
use dart::DartFileBuilder;
use flusty_parse::rust::module::Module;
use simplelog::{
    ColorChoice, Config as LogConfig, LevelFilter, TermLogger, TerminalMode,
};

pub mod config;
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
    dbg!(&module);
    let mut dart_builder = DartFileBuilder::new();
    dart_builder.set_lib_name("flusty");
    dart_builder.add_lib_path("flusty");
    dart_builder.set_module_name("flusty");
    for f in module.functions {
        dart_builder.add_fn(&f).expect("Failed to add function");
    }
    println!("{}", dart_builder.build().unwrap());
}
