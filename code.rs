/// This is a generated file. Do not edit. It is updated depending on the build profile used (i.e. dev, release).
/// Instead it should be included with an include! macro: `include!("../gen/module_path.rs");`
pub mod Assets {
    #![allow(dead_code)]
    pub const POLYGLOT_URL: &str = "797nNevRHR8=polyglot.woff2";
    pub const POLYGLOT_ENC: [&str; 3] = ["gz", "", "br"];
    pub const STYLE_URL: &str = "IzXCtcAPvPs=style.css";
    pub const STYLE_ENC: [&str; 3] = ["gz", "", "br"];
    pub const APPLE_STORE_URL: &str = "N7UZwX_xY8E=apple_store.svg";
    pub const APPLE_STORE_ENC: [&str; 3] = ["", "br", "gz"];
    pub const APPLE_STORE_LANGS: [&str; 126] = [
        "ar", "ar", "ar", "az", "az", "az", "bg", "bg", "bg", "cs", "cs", "cs", "da", "da", "da",
        "de", "de", "de", "el", "el", "el", "en", "en", "en", "es", "es", "es", "es-MX", "es-MX",
        "es-MX", "et", "et", "et", "fi", "fi", "fi", "fil", "fil", "fil", "fr", "fr", "fr",
        "fr-CA", "fr-CA", "fr-CA", "he", "he", "he", "hu", "hu", "hu", "id", "id", "id", "it",
        "it", "it", "ja", "ja", "ja", "ko", "ko", "ko", "lt", "lt", "lt", "lv", "lv", "lv", "ms",
        "ms", "ms", "mt", "mt", "mt", "nl", "nl", "nl", "nn", "nn", "nn", "pl", "pl", "pl", "pt",
        "pt", "pt", "pt-BR", "pt-BR", "pt-BR", "ro", "ro", "ro", "ru", "ru", "ru", "sk", "sk",
        "sk", "sl", "sl", "sl", "sv", "sv", "sv", "th", "th", "th", "tr", "tr", "tr", "ur", "ur",
        "ur", "vi", "vi", "vi", "zh-CN", "zh-CN", "zh-CN", "zh-HK", "zh-HK", "zh-HK", "zh-TW",
        "zh-TW", "zh-TW",
    ];
    pub const GOOGLE_PLAY_URL: &str = "x6r8PGoxrOM=google_play.svg";
    pub const GOOGLE_PLAY_ENC: [&str; 3] = ["", "br", "gz"];
    pub const GOOGLE_PLAY_LANGS: [&str; 231] = [
        "af", "af", "af", "am", "am", "am", "ar", "ar", "ar", "az", "az", "az", "be", "be", "be",
        "bg", "bg", "bg", "bn", "bn", "bn", "bs", "bs", "bs", "ca", "ca", "ca", "cs", "cs", "cs",
        "da", "da", "da", "de", "de", "de", "el", "el", "el", "en", "en", "en", "es", "es", "es",
        "es-418", "es-418", "es-418", "et", "et", "et", "eu", "eu", "eu", "fa", "fa", "fa", "fi",
        "fi", "fi", "fil", "fil", "fil", "fr", "fr", "fr", "fr-CA", "fr-CA", "fr-CA", "gl", "gl",
        "gl", "gu", "gu", "gu", "hi", "hi", "hi", "hr", "hr", "hr", "hu", "hu", "hu", "hy", "hy",
        "hy", "id", "id", "id", "is", "is", "is", "it", "it", "it", "iw", "iw", "iw", "ja", "ja",
        "ja", "ka", "ka", "ka", "kk", "kk", "kk", "km", "km", "km", "kn", "kn", "kn", "ko", "ko",
        "ko", "ky", "ky", "ky", "lo", "lo", "lo", "lt", "lt", "lt", "lv", "lv", "lv", "mk", "mk",
        "mk", "ml", "ml", "ml", "mn", "mn", "mn", "mr", "mr", "mr", "ms", "ms", "ms", "my", "my",
        "my", "ne", "ne", "ne", "nl", "nl", "nl", "no", "no", "no", "pa", "pa", "pa", "pl", "pl",
        "pl", "pt", "pt", "pt", "pt-BR", "pt-BR", "pt-BR", "ro", "ro", "ro", "ru", "ru", "ru",
        "si", "si", "si", "sk", "sk", "sk", "sl", "sl", "sl", "sq", "sq", "sq", "sr", "sr", "sr",
        "sv", "sv", "sv", "sw", "sw", "sw", "ta", "ta", "ta", "te", "te", "te", "th", "th", "th",
        "tr", "tr", "tr", "ua", "ua", "ua", "ur", "ur", "ur", "uz", "uz", "uz", "vi", "vi", "vi",
        "zh-CH", "zh-CH", "zh-CH", "zh-HK", "zh-HK", "zh-HK", "zh-TW", "zh-TW", "zh-TW", "zu",
        "zu", "zu",
    ];

    pub struct AssetOptions {
        pub langs: Option<&'static [&'static str]>,
        pub encodings: Option<&'static [&'static str]>,
    }

    pub fn localisations_and_compressions_for_url(url: &str) -> Option<AssetOptions> {
        match url {
            "797nNevRHR8=polyglot.woff2" => Some(AssetOptions {
                langs: None,
                encodings: Some(&POLYGLOT_ENC),
            }),
            "IzXCtcAPvPs=style.css" => Some(AssetOptions {
                langs: None,
                encodings: Some(&STYLE_ENC),
            }),
            "N7UZwX_xY8E=apple_store.svg" => Some(AssetOptions {
                langs: Some(&APPLE_STORE_LANGS),
                encodings: Some(&APPLE_STORE_ENC),
            }),
            "x6r8PGoxrOM=google_play.svg" => Some(AssetOptions {
                langs: Some(&GOOGLE_PLAY_LANGS),
                encodings: Some(&GOOGLE_PLAY_ENC),
            }),
            _ => None,
        }
    }
}
