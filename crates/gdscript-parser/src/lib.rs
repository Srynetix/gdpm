use std::path::{Path, PathBuf};

use gdpm_io::IoAdapter;
use parsers::new_span;

pub(crate) mod ast;
pub(crate) mod parsers;

#[cfg(test)]
mod tests;

pub(crate) mod types;

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Path does not exist: {0}")]
    MissingPath(PathBuf),
    #[error("Error: {0}")]
    Custom(String),
    #[error("Parse error on file {0}: {1}")]
    ParseError(PathBuf, String),
}

#[derive(Debug)]
pub struct GdScriptParser;

impl GdScriptParser {
    pub fn parse_path(io: &dyn IoAdapter, path: impl AsRef<Path>) -> Result<(), ParserError> {
        let path = path.as_ref();
        if path.is_dir() {
            Self::parse_dir(io, path)
        } else if path.is_file() {
            Self::parse_file(path)
        } else {
            Err(ParserError::MissingPath(path.to_owned()))
        }
    }

    pub fn parse_dir(io: &dyn IoAdapter, path: impl AsRef<Path>) -> Result<(), ParserError> {
        let mut file_map = vec![];

        for file in io
            .find_files_in_dir(path.as_ref(), ".gd")
            .map_err(|_| ParserError::MissingPath(path.as_ref().to_owned()))?
        {
            let contents = std::fs::read_to_string(&file).unwrap();

            match parsers::parse_file(new_span(&contents)) {
                Ok(_) => {
                    file_map.push((file.clone(), "OK".into()));
                }
                Err(e) => {
                    file_map.push((file.clone(), format!("ERROR: {:#?}", e)));
                }
            }
        }

        for (file_name, status) in file_map {
            println!("{}:{}", file_name.as_os_str().to_string_lossy(), status);
        }
        Ok(())
    }

    pub fn parse_file(path: impl AsRef<Path>) -> Result<(), ParserError> {
        let contents = std::fs::read_to_string(path.as_ref()).unwrap();

        match parsers::parse_file(new_span(&contents)) {
            Ok((_, p)) => {
                println!("{:#?}", p);
                Ok(())
            }
            Err(e) => Err(ParserError::ParseError(
                path.as_ref().to_path_buf(),
                format!("{:#?}", e),
            )),
        }
    }
}
