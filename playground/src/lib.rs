include!("../gen/mobile.rs");
include!("../gen/web.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_assets() {
        let f = mobile::localisations_and_compressions_for_url("wrong");
        assert!(f.is_none());
        let url = mobile::MAIN_SCSS_URL;
        assert_eq!(url, "/main.scss");
    }

    #[test]
    fn web_assets() {
        let f = web::localisations_and_compressions_for_url("wrong");
        assert!(f.is_none());
        let url = web::MAIN_SCSS_URL;
        assert!(url.ends_with("main.scss"));
    }
}
