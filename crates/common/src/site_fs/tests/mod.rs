use builder_command::Encoding;
use camino_fs::{Utf8PathBuf, Utf8PathExt};
use icu_locid::langid;

use crate::site_fs::{
    checksum_from, parse_site, Asset, AssetEncodings, AssetPath, SiteFile, TranslatedAssetPath,
};

fn create_tmp_dir(path: &str) -> Utf8PathBuf {
    let path = Utf8PathBuf::from(path);
    if path.exists() {
        path.rm().unwrap();
    }
    path.mkdirs().unwrap();
    path
}

#[test]
fn single_uncompressed() {
    let tmp_dir = create_tmp_dir("src/tests/tmp/single_uncompressed");

    let font_path = AssetPath {
        subdir: "".into(),
        name_ext: SiteFile::new("font", "woff2"),
        checksum: None,
    }
    .absolute_path(&tmp_dir);
    println!("font_path: {font_path}");

    AssetEncodings::uncompressed()
        .write(&font_path, "font content".as_bytes())
        .unwrap();

    let assets = parse_site(&tmp_dir).unwrap();
    assert_eq!(
        assets,
        [Asset {
            sub_dir: None,
            name: "font".to_string(),
            hash: None,
            ext: "woff2".to_string(),
            encodings: AssetEncodings::uncompressed(),
            translations: Default::default(),
        }]
    );
}

#[test]
fn single_hashed() {
    let tmp_dir = create_tmp_dir("src/tests/tmp/single_hashed");

    let hash = checksum_from("content".as_bytes());

    let font_path = AssetPath {
        subdir: "".into(),
        name_ext: SiteFile::new("font", "woff2"),
        checksum: Some(hash.clone()),
    }
    .absolute_path(&tmp_dir);
    println!("font_path: {font_path}");

    AssetEncodings::uncompressed()
        .write(&font_path, "font content".as_bytes())
        .unwrap();

    let assets = parse_site(&tmp_dir).unwrap();
    assert_eq!(
        assets,
        [Asset {
            sub_dir: None,
            name: "font".to_string(),
            hash: Some(hash),
            ext: "woff2".to_string(),
            encodings: AssetEncodings::uncompressed(),
            translations: Default::default(),
        }]
    );
}

#[test]
fn single_compressed() {
    let tmp_dir = create_tmp_dir("src/tests/tmp/single_compressed");

    let font_path = AssetPath {
        subdir: "".into(),
        name_ext: SiteFile::new("font", "woff2"),
        checksum: None,
    }
    .absolute_path(&tmp_dir);

    AssetEncodings::all()
        .write(&font_path, "font content".as_bytes())
        .unwrap();

    let assets = parse_site(&tmp_dir).unwrap();
    assert_eq!(
        assets,
        [Asset {
            sub_dir: None,
            name: "font".to_string(),
            hash: None,
            ext: "woff2".to_string(),
            encodings: AssetEncodings::all(),
            translations: Default::default(),
        }]
    );
}

#[test]
fn single_in_subfolder() {
    let tmp_dir = create_tmp_dir("src/tests/tmp/single_in_subfolder");

    let font_path = AssetPath {
        subdir: "fonts".into(),
        name_ext: SiteFile::new("font", "woff2"),
        checksum: None,
    }
    .absolute_path(&tmp_dir);

    tmp_dir.join("fonts").mkdir().unwrap();

    AssetEncodings::uncompressed()
        .write(&font_path, "font content".as_bytes())
        .unwrap();

    let assets = parse_site(&tmp_dir).unwrap();
    assert_eq!(
        assets,
        [Asset {
            sub_dir: Some("fonts".into()),
            name: "font".to_string(),
            hash: None,
            ext: "woff2".to_string(),
            encodings: AssetEncodings::uncompressed(),
            translations: Default::default(),
        }]
    );
}

#[test]
fn single_translated() {
    let tmp_dir = create_tmp_dir("src/tests/tmp/single_translated");

    let mut path = TranslatedAssetPath {
        site_file: SiteFile::new("image", "svg"),
        lang: "en".into(),
        checksum: None,
    };
    let en_path = path.absolute_path(&tmp_dir);
    println!("en_path: {en_path}");
    en_path.parent().unwrap().mkdir().unwrap();

    path.lang = "de".into();
    let de_path = path.absolute_path(&tmp_dir);

    AssetEncodings::uncompressed()
        .write(&en_path, "content".as_bytes())
        .unwrap();

    AssetEncodings::uncompressed()
        .write(&de_path, "content".as_bytes())
        .unwrap();

    let assets = parse_site(&tmp_dir).unwrap();
    assert_eq!(
        assets,
        [Asset {
            sub_dir: None,
            name: "image".to_string(),
            hash: None,
            ext: "svg".to_string(),
            encodings: AssetEncodings::uncompressed(),
            translations: vec![langid!("de"), langid!("en")],
        }]
    );
}

#[test]
fn single_hashed_translated() {
    let tmp_dir = create_tmp_dir("src/tests/tmp/single_hashed_translated");

    let hash = checksum_from("content".as_bytes());
    let mut path = TranslatedAssetPath {
        site_file: SiteFile::new("image", "svg"),
        lang: "en".into(),
        checksum: Some(hash.clone()),
    };
    let en_path = path.absolute_path(&tmp_dir);
    println!("en_path: {en_path}");
    en_path.parent().unwrap().mkdir().unwrap();

    path.lang = "de".into();
    let de_path = path.absolute_path(&tmp_dir);

    AssetEncodings::uncompressed()
        .write(&en_path, "content".as_bytes())
        .unwrap();

    AssetEncodings::uncompressed()
        .write(&de_path, "content".as_bytes())
        .unwrap();

    let assets = parse_site(&tmp_dir).unwrap();
    assert_eq!(
        assets,
        [Asset {
            sub_dir: None,
            name: "image".to_string(),
            hash: Some(hash),
            ext: "svg".to_string(),
            encodings: AssetEncodings::uncompressed(),
            translations: vec![langid!("de"), langid!("en")],
        }]
    );
}

#[test]
fn multiple() {
    let tmp_dir = create_tmp_dir("src/tests/tmp/multiple");

    let font_path = AssetPath {
        subdir: "fonts".into(),
        name_ext: SiteFile::new("font", "woff2"),
        checksum: None,
    }
    .absolute_path(&tmp_dir);

    let img_hash = checksum_from("image content".as_bytes());
    let image_path = AssetPath {
        subdir: "".into(),
        name_ext: SiteFile::new("image", "svg"),
        checksum: Some(img_hash.clone()),
    }
    .absolute_path(&tmp_dir);

    AssetEncodings::uncompressed()
        .write(&font_path, "font content".as_bytes())
        .unwrap();

    AssetEncodings::from_iter([Encoding::Gzip])
        .write(&image_path, "image content".as_bytes())
        .unwrap();

    // a translated svg image with hash

    let tr_hash = checksum_from("svg content".as_bytes());

    let mut translated_img = TranslatedAssetPath {
        site_file: SiteFile::new("tr_image", "svg"),
        lang: "en".into(),
        checksum: Some(tr_hash.clone()),
    };
    let en_path = translated_img.absolute_path(&tmp_dir);

    translated_img.lang = "fr".into();
    let fr_path = translated_img.absolute_path(&tmp_dir);

    AssetEncodings::all()
        .write(&en_path, "svg content".as_bytes())
        .unwrap();
    AssetEncodings::all()
        .write(&fr_path, "svg content".as_bytes())
        .unwrap();

    let assets = parse_site(&tmp_dir).unwrap();
    assert_eq!(assets.len(), 3);
    assert_eq!(assets[0].to_url(), "/fonts/font.woff2");
    assert_eq!(
        assets[0],
        Asset {
            sub_dir: Some("fonts".into()),
            name: "font".to_string(),
            hash: None,
            ext: "woff2".to_string(),
            encodings: AssetEncodings::uncompressed(),
            translations: Default::default(),
        }
    );

    assert_eq!(assets[1].to_url(), "/image.P0SY9_UGSLM=.svg");
    assert_eq!(
        assets[1],
        Asset {
            sub_dir: None,
            name: "image".to_string(),
            hash: Some(img_hash.clone()),
            ext: "svg".to_string(),
            encodings: AssetEncodings::from_iter([Encoding::Gzip]),
            translations: Default::default(),
        },
    );

    assert_eq!(assets[2].to_url(), "/tr_image.TvSNVuI-jOs=.svg");
    assert_eq!(
        assets[2],
        Asset {
            sub_dir: None,
            name: "tr_image".to_string(),
            hash: Some(tr_hash),
            ext: "svg".to_string(),
            encodings: AssetEncodings::all(),
            translations: vec![langid!("en"), langid!("fr")],
        }
    );
}
