// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use figment::{
    providers::{Format, Json, Toml, Yaml},
    Figment,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub data_dir: PathBuf,
}

impl Config {
    fn load_figment(paths: Vec<PathBuf>) -> Figment {
        let figment = Figment::new();

        paths.iter().fold(figment, |fig: Figment, p| {
            glob::glob(p.join("**").join("*.*").to_str().unwrap())
                .unwrap()
                .fold(fig, |fig, f| {
                    match dbg!(f.as_ref()).unwrap().extension().unwrap().to_str() {
                        Some("yaml") | Some("yml") => fig.merge(Yaml::file(f.unwrap()).nested()),
                        Some("toml") => fig.merge(Toml::file(f.unwrap()).nested()),
                        Some("json") => fig.merge(Json::file(f.unwrap()).nested()),
                        _ => {
                            println!(
                                "Unknown filetype in configuration at {file}",
                                file = f.unwrap().display()
                            );
                            fig
                        }
                    }
                })
        })
    }

    #[cfg(not(tarpaulin_include))]
    pub fn load(paths: Vec<PathBuf>) -> Self {
        Self::load_figment(paths).extract().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_fs::prelude::*;
    use rstest::rstest;
    use rstest_reuse::*;

    const TOML: &str = r#"
    [default]
    data_dir = "/tmp"
    "#;

    const YAML: &str = r#"
    default:
      data_dir: /tmp
    "#;

    const JSON: &str = r#"
    {"default": {"data_dir": "/tmp"}}
    "#;

    fn figment(name: &str, content: &str) -> (PathBuf, Figment, String) {
        let tmp = assert_fs::TempDir::new().unwrap();

        tmp.child(name).write_str(content).unwrap();

        let path = tmp.path();

        (
            path.to_owned(),
            Config::load_figment(vec![path.to_owned()]),
            name.to_owned(),
        )
    }

    #[template]
    #[rstest]
    #[case(figment("config.toml", TOML), "TOML file")]
    #[case(figment("config.yaml", YAML), "YAML file")]
    #[case(figment("config.yaml", JSON), "YAML file")]
    #[case(figment("config.json", JSON), "JSON file")]
    #[case(figment("config.yml", YAML), "YAML file")]
    #[case(figment("config.yml", JSON), "YAML file")]
    #[case(figment("folder/config.toml", TOML), "TOML file")]
    #[case(figment("folder/config.yaml", YAML), "YAML file")]
    #[case(figment("folder/config.yaml", JSON), "YAML file")]
    #[case(figment("folder/config.json", JSON), "JSON file")]
    #[case(figment("folder/config.yml", YAML), "YAML file")]
    #[case(figment("folder/config.yml", JSON), "YAML file")]
    fn the_template(#[case] figment_data: (PathBuf, Figment, String), #[case] md_name: &str) {}

    #[apply(the_template)]
    fn count_metadata(#[case] figment_data: (PathBuf, Figment, String), #[case] _md_name: &str) {
        let (_path, figment, _name) = figment_data;

        assert_eq!(1, figment.metadata().count());
    }

    #[apply(the_template)]
    fn metadata_name(#[case] figment_data: (PathBuf, Figment, String), #[case] md_name: &str) {
        let (_path, figment, _name) = figment_data;

        let md = dbg!(figment.metadata().next().unwrap());

        assert_eq!(md_name, md.name);
    }

    #[apply(the_template)]
    fn metadata_filename(#[case] figment_data: (PathBuf, Figment, String), #[case] _md_name: &str) {
        let (path, figment, name) = figment_data;

        let md = dbg!(figment.metadata().next().unwrap());

        let src_name = md
            .source
            .as_ref()
            .and_then(|s| s.file_path())
            .and_then(|p| p.strip_prefix(path).ok())
            .and_then(|p| p.to_str())
            .unwrap();

        assert_eq!(name, src_name);
    }

    #[apply(the_template)]
    fn config_data_dir(#[case] figment_data: (PathBuf, Figment, String), #[case] _md_name: &str) {
        let (_path, figment, _name) = figment_data;

        let cfg: Config = figment.extract().unwrap();

        assert_eq!(PathBuf::from("/tmp"), cfg.data_dir);
    }

    //     assert_eq!(
    //         PathBuf::from("/tmp"),
    //         figment.extract::<Config>().unwrap().data_dir
    //     )
    // }
}
