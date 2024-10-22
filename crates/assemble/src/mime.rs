pub fn mime_from_ext(ext: &str) -> &'static str {
    if ext.ends_with("js") {
        "application/javascript"
    } else if ext.ends_with("css") {
        "text/css"
    } else if ext.ends_with("wasm") {
        "application/wasm"
    } else if ext.ends_with("svg") {
        "image/svg+xml"
    } else if ext.ends_with("woff2") {
        "font/woff2"
    } else {
        panic!("Missing mapping file ext '{ext}' -> mime type. Please add it to mime.rs")
    }
}
