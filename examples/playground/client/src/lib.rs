// This would normally have the below inclueds, but
// when building on CI, the builder is not available, sot
// the builder is skipped in the build.rs file, and thus
// these files are not generated

// include!("../gen/mobile.rs");
// include!("../gen/web.rs");

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

pub fn str() -> &'static str {
    "client"
}
