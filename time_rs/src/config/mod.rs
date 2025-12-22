// SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Defines how to interact with configuration

use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use figment::{
    providers::{Format, Json, Toml, Yaml},
    Figment,
};
use serde::Deserialize;
use tokio::task;

pub use crate::config::error::Error;

pub mod error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Deserialize)]
/// Describes the program configuration
pub struct Config {
    /// The location to store the actual data
    pub data_dir: Option<PathBuf>,

    #[serde(skip)]
    figment: Figment,
}

impl Config {
    async fn load_figment(paths: Vec<PathBuf>) -> Result<Figment> {
        let figment = Figment::new();

        task::spawn_blocking(move || {
            paths
                .iter()
                .map(|p| p.join("**").join("*.*"))
                .map(|p| {
                    p.to_str()
                        .map(ToString::to_string)
                        .ok_or_else(|| Error::PathStringConversion(p.clone()))
                })
                .flat_map(|s| {
                    let s = s?;
                    glob::glob(&s).map_err(|e| Error::PatternError(s.clone(), e))
                })
                .flatten()
                .map(|f| {
                    let file = f?;
                    let ext = file.extension().and_then(|s| s.to_str());

                    match ext {
                        Some("yaml" | "yml") => Ok(Figment::from(Yaml::file(file).nested())),
                        Some("toml") => Ok(Figment::from(Toml::file(file).nested())),
                        Some("json") => Ok(Figment::from(Json::file(file).nested())),
                        Some(ext) => Err(Error::UnknownExtension(file.clone(), ext.to_owned())),
                        None => Err(Error::NoExtension(file.clone())),
                    }
                })
                .try_fold(figment, |acc, additional| Ok(acc.merge(additional?)))
        })
        .await
        .map_err(Error::JoinError)?
    }

    #[allow(clippy::missing_errors_doc)]
    /// Load the configuration from the given possible paths.
    pub async fn load(paths: Vec<PathBuf>) -> Result<Self> {
        let figment = Self::load_figment(paths)
            .await
            .map_err(|e| Error::LoadingConfig(Box::new(e)))?;

        figment.try_into()
    }

    #[allow(clippy::missing_errors_doc)]
    /// Sets the [`Self::data_dir`]
    pub fn add_data_dir<P>(&mut self, path: P) -> Result<()>
    where
        P: AsRef<Path> + Debug,
    {
        let figment = self.figment.clone().merge((
            "data_dir",
            path.as_ref()
                .to_str()
                .ok_or_else(|| Error::PathStringConversion(path.as_ref().to_owned()))?,
        ));

        *self = figment.try_into()?;

        Ok(())
    }
}

impl TryFrom<Figment> for Config {
    type Error = Error;

    fn try_from(figment: Figment) -> Result<Self> {
        Ok(Self {
            figment: figment.clone(),
            ..figment.extract().map_err(Box::new)?
        })
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use super::*;

    use assert_fs::{prelude::*, TempDir};
    use rstest::rstest;
    use rstest_reuse::*;

    const TOML: &str = r#"
    [default]
    data_dir = "/tmp"
    "#;

    const YAML: &str = "
    default:
      data_dir: /tmp
    ";

    const JSON: &str = r#"
    {"default": {"data_dir": "/tmp"}}
    "#;

    async fn figment(name: &str, content: &str) -> (TempDir, PathBuf, Figment, String) {
        let tmp = assert_fs::TempDir::new().unwrap();

        tmp.child(name).write_str(content).unwrap();

        let path = tmp.to_owned();

        (
            tmp,
            path.clone(),
            Config::load_figment(vec![path]).await.unwrap(),
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
    #[tokio::test]
    #[allow(clippy::used_underscore_binding)]
    async fn count_metadata(
        #[case] figment_data: impl Future<Output = (TempDir, PathBuf, Figment, String)>,
        #[case] _md_name: &str,
    ) {
        let (_tmp, _path, figment, _name) = figment_data.await;

        // 1. the read config file
        // 2. the implied defaults
        assert_eq!(2, figment.metadata().count());
    }

    #[apply(the_template)]
    #[tokio::test]
    async fn metadata_name(
        #[case] figment_data: impl Future<Output = (TempDir, PathBuf, Figment, String)>,
        #[case] md_name: &str,
    ) {
        let (_tmp, _path, figment, _name) = figment_data.await;

        let md = figment.metadata().next().unwrap();

        assert_eq!(md_name, md.name);
    }

    #[apply(the_template)]
    #[tokio::test]
    #[allow(clippy::used_underscore_binding)]
    async fn metadata_filename(
        #[case] figment_data: impl Future<Output = (TempDir, PathBuf, Figment, String)>,
        #[case] _md_name: &str,
    ) {
        let (_tmp, path, figment, name) = figment_data.await;

        let md = figment.metadata().next().unwrap();

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
    #[tokio::test]
    #[allow(clippy::used_underscore_binding)]
    async fn config_data_dir(
        #[case] figment_data: impl Future<Output = (TempDir, PathBuf, Figment, String)>,
        #[case] _md_name: &str,
    ) {
        let (_tmp, _path, figment, _name) = figment_data.await;

        let cfg: Config = figment.extract().unwrap();

        assert_eq!(Some(PathBuf::from("/tmp")), cfg.data_dir);
    }

    #[apply(the_template)]
    #[tokio::test]
    #[allow(clippy::used_underscore_binding)]
    async fn load_config_data_dir(
        #[case] figment_data: impl Future<Output = (TempDir, PathBuf, Figment, String)>,
        #[case] _md_name: &str,
    ) {
        let cfg = Config::load(vec![figment_data.await.0.path().to_owned()])
            .await
            .unwrap();

        assert_eq!(Some(PathBuf::from("/tmp")), cfg.data_dir);
    }

    #[tokio::test]
    async fn test_add_data_dir() {
        let mut config = Config::load(vec![]).await.unwrap();
        let path = PathBuf::from("/new/data/dir");
        config.add_data_dir(path.clone()).unwrap();
        assert_eq!(config.data_dir, Some(path));
    }

    #[tokio::test]
    async fn test_unknown_file_extension() {
        let tmpdir = TempDir::new().unwrap();
        tmpdir.child("foo.txt").touch().unwrap();

        let config = Config::load(vec![tmpdir.to_path_buf()]).await;

        assert!(config.is_err());
        let err = config.unwrap_err();

        match err {
            Error::LoadingConfig(ref inner) => match &**inner {
                Error::UnknownExtension(_, ref ext) => assert_eq!(ext, "txt"),
                err => panic!("Error::UnknownExtension was expected, got {err:?}"),
            },
            err => panic!("Error::LoadingConfig was expected, got {err:?}"),
        }
    }
}
