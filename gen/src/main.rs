use std::{fs::File, io::Write};

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
    let config = Config::from_disk().unwrap_or_default();
    // config.set_libpath(vec![&"example", &"native", &"target", &"release"]);
    // config.set_rust_entry("example/native/src/");
    // config.set_dart_out("example/lib/flusty.dart");
    // config.save();
    let module = Module::builder(&["flusty"])
        .path(config.rust_entry().display().to_string())
        .name("lib".to_string())
        .data()
        .expect("Failed to parse module")
        .build();
    dbg!(&module);
    let mut dart_builder = DartFileBuilder::new();
    dart_builder.set_lib_name("libnative");
    dart_builder.set_lib_path(
        &config
            .libpath()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
    );
    dart_builder.set_module_name("Flusty");
    for f in module.functions {
        dart_builder.add_fn(&f).expect("Failed to add function");
    }
    let file_path = config.dart_out();
    dbg!(&file_path);
    // create the directory if it doesn't exist
    if let Some(dir) = file_path.parent() {
        std::fs::create_dir_all(dir).expect("Failed to create directory");
    }
    let mut file = File::create(file_path).expect("Failed to create file");
    file.write_all(dart_builder.build().unwrap().as_bytes())
        .expect("Failed to write to file");
}
