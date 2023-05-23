use flusty_build::rust::module::Module;

fn main() {
    let mut builder = Module::builder(&["test"]);
    let res = builder
        .name("test_fn".to_string())
        .path("test_files/".to_string())
        .data()
        .unwrap()
        .build();
    dbg!(res);
}
