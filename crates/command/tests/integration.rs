use builder_command::{BuilderCmd, WasmCmd};

#[test]
fn test_roundtrip() {
    let cmd = BuilderCmd::new().add_wasm(WasmCmd::new("my-package"));
    let s = toml::to_string(&cmd).unwrap();
    let _cmd2: BuilderCmd = toml::from_str(&s).unwrap();
}
