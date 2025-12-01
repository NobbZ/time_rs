// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use eyre::{eyre, Context, Ok, Result};
use figment::{
    providers::{Format, Json, Toml, Yaml},
    Figment,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub data_dir: Option<PathBuf>,

    #[serde(skip)]
    figment: Figment,
}

impl Config {
    fn load_figment(paths: Vec<PathBuf>) -> Result<Figment> {
        let figment = Figment::new();

        paths
            .iter()
            .map(|p| p.join("**").join("*.*"))
            .map(|p| {
                p.to_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| eyre!("converting path {:?} to string", p))
            })
            .flat_map(|s| {
                let s = s?;
                glob::glob(&s).wrap_err_with(|| format!("creating glob from {}", s))
            })
            .flatten()
            .map(|f| {
                let file = f?;
                let ext = file.extension().and_then(|s| s.to_str());

                match ext {
                    Some("yaml" | "yml") => Ok(Figment::from(Yaml::file(file).nested())),
                    Some("toml") => Ok(Figment::from(Toml::file(file).nested())),
                    Some("json") => Ok(Figment::from(Json::file(file).nested())),
                    Some(ext) => Err(eyre!("unknown extension {}", ext)),
                    None => Err(eyre!("no extension found")),
                }
            })
            .try_fold(figment, |acc, additional| Ok(acc.merge(additional?)))
    }

    pub fn load(paths: Vec<PathBuf>) -> Result<Self> {
        let figment = Self::load_figment(paths).wrap_err("loading config")?;

        figment.try_into()
    }

    pub fn add_data_dir<P>(&mut self, path: P) -> Result<()>
    where
        P: AsRef<Path> + Debug,
    {
        let figment = self.figment.clone().merge((
            "data_dir",
            path.as_ref()
                .to_str()
                .ok_or_else(|| eyre!("Can not convert {:?} to string", path))?,
        ));

        *self = figment.clone().try_into()?;

        Ok(())
    }
}

impl TryFrom<Figment> for Config {
    type Error = eyre::Report;

    fn try_from(figment: Figment) -> Result<Self> {
        Ok(Config {
            figment: figment.clone(),
            ..figment.extract().wrap_err("extracting config")?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_fs::{prelude::*, TempDir};
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

    fn figment(name: &str, content: &str) -> (TempDir, PathBuf, Figment, String) {
        let tmp = assert_fs::TempDir::new().unwrap();

        tmp.child(name).write_str(content).unwrap();

        let path = tmp.to_owned();

        (
            tmp,
            path.clone(),
            Config::load_figment(vec![path]).unwrap(),
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
    #[case(figment("random_name.toml", TOML), "TOML file")]
    #[case(figment("random_name.yaml", YAML), "YAML file")]
    #[case(figment("random_name.yaml", JSON), "YAML file")]
    #[case(figment("random_name.json", JSON), "JSON file")]
    fn the_template(
        #[case] figment_data: (PathBuf, PathBuf, Figment, String),
        #[case] md_name: &str,
    ) {
    }

    #[apply(the_template)]
    fn count_metadata(
        #[case] figment_data: (TempDir, PathBuf, Figment, String),
        #[case] _md_name: &str,
    ) {
        let (_tmp, _path, figment, _name) = figment_data;

        assert_eq!(2, figment.metadata().count());
    }

    #[apply(the_template)]
    fn metadata_name(
        #[case] figment_data: (TempDir, PathBuf, Figment, String),
        #[case] md_name: &str,
    ) {
        let (_tmp, _path, figment, _name) = figment_data;

        let md = dbg!(figment.metadata().next().unwrap());

        assert_eq!(md_name, md.name);
    }

    #[apply(the_template)]
    fn metadata_filename(
        #[case] figment_data: (TempDir, PathBuf, Figment, String),
        #[case] _md_name: &str,
    ) {
        let (_tmp, path, figment, name) = figment_data;

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
    fn config_data_dir(
        #[case] figment_data: (TempDir, PathBuf, Figment, String),
        #[case] _md_name: &str,
    ) {
        let (_tmp, _path, figment, _name) = figment_data;

        let cfg: Config = figment.extract().unwrap();

        assert_eq!(Some(PathBuf::from("/tmp")), cfg.data_dir);
    }

    #[apply(the_template)]
    fn load_config_data_dir(
        #[case] figment_data: (TempDir, PathBuf, Figment, String),
        #[case] _md_name: &str,
    ) {
        let cfg = Config::load(vec![figment_data.0.path().to_owned()]).unwrap();

        assert_eq!(Some(PathBuf::from("/tmp")), cfg.data_dir)
    }

    #[test]
    fn test_add_data_dir() {
        let mut config = Config::load(vec![]).unwrap();
        let path = PathBuf::from("/new/data/dir");
        config.add_data_dir(path.clone()).unwrap();
        assert_eq!(config.data_dir, Some(path));
    }
}
