
pub mod mobile {
    #![allow(dead_code)]
    pub const MAIN_SCSS_URL: &str = "/main.scss";
    pub const MAIN_SCSS_ENC: [&str; 1] = ["identity"];


    pub struct AssetOptions {
        pub langs: Option<&'static [&'static str]>,
        pub encodings: Option<&'static [&'static str]>,
    }

    pub fn localisations_and_compressions_for_url(url: &str) -> Option<AssetOptions> {
       match url {
            "/main.scss" => Some(AssetOptions {
                langs: None,
                encodings: Some(&MAIN_SCSS_ENC),
            }),
            _ => None,
        }
    }
}
