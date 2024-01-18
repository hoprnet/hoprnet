use crate::utils::HelperErrors;
use clap::{Parser, ValueHint};
use log::{log, Level};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Parser)]
#[derive(Default)]
pub struct LocalIdentityFromDirectoryArgs {
    #[clap(
        help = "Path to the directory that stores identity files",
        long,
        short = 'd',
        value_hint = ValueHint::DirPath,
    )]
    pub identity_directory: Option<String>,

    #[clap(
        help = "Only use identity files with prefix",
        long,
        short = 'x',
        default_value = None
    )]
    pub identity_prefix: Option<String>,
}

#[derive(Debug, Clone, Parser)]
#[derive(Default)]
pub struct LocalIdentityArgs {
    #[clap(help = "Get identity file(s) from a directory", flatten)]
    pub identity_from_directory: Option<LocalIdentityFromDirectoryArgs>,

    #[clap(
        long,
        help = "The path to an identity file",
        value_hint = ValueHint::FilePath,
        name = "identity_from_path"
    )]
    pub identity_from_path: Option<PathBuf>,
}



impl LocalIdentityFromDirectoryArgs {
    /// read files from given directory or file path
    pub fn get_files(self) -> Result<Vec<PathBuf>, HelperErrors> {
        let LocalIdentityFromDirectoryArgs {
            identity_directory,
            identity_prefix,
        } = self;
        let id_dir = identity_directory.unwrap();

        log!(target: "identity_reader", Level::Debug, "Reading dir {}", &id_dir);

        // early return if failed in reading identity directory
        let directory = fs::read_dir(Path::new(&id_dir))?;
        // read all the files from the directory that contains
        // 1) "id" in its name
        // 2) the provided idetity_prefix
        let files: Vec<PathBuf> = directory
            .into_iter() // read all the files from the directory
            .filter(|r| r.is_ok()) // Get rid of Err variants for Result<DirEntry>
            .map(|r| r.unwrap().path()) // Read all the files from the given directory
            .filter(|r| r.is_file()) // Filter out folders
            .filter(|r| r.to_str().unwrap().contains("id")) // file name should contain "id"
            .filter(|r| match &identity_prefix {
                Some(id_prf) => r.file_stem().unwrap().to_str().unwrap().starts_with(id_prf.as_str()),
                _ => true,
            })
            .collect();
        log!(target: "identity_reader", Level::Info, "{} path read from dir", &files.len());
        Ok(files)
    }
}



impl LocalIdentityArgs {
    /// read files from given directory or file path
    pub fn get_files(self) -> Vec<PathBuf> {
        let LocalIdentityArgs {
            identity_from_directory,
            identity_from_path,
        } = self;
        log!(target: "identity_reader", Level::Info, "Read from dir {}, path {}", &identity_from_directory.is_some(), &identity_from_path.is_some());

        let mut files: Vec<PathBuf> = Vec::new();
        if let Some(id_dir_args) = identity_from_directory {
            files = id_dir_args.get_files().unwrap();
        };
        if let Some(id_path) = identity_from_path {
            log!(target: "identity_reader", Level::Info, "Reading path {}", &id_path.as_path().display().to_string());
            if id_path.exists() {
                files.push(id_path);
                log!(target: "identity_reader", Level::Info, "path read from path");
            } else {
                log!(target: "identity_reader", Level::Error, "Path {} does not exist.", &id_path.as_path().display().to_string());
            }
        }
        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revert_get_dir_from_non_existing_dir() {
        let path = "./tmp_non_exist";
        let dir_args = LocalIdentityFromDirectoryArgs {
            identity_directory: Some(path.to_string()),
            identity_prefix: None,
        };

        match dir_args.get_files() {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn pass_get_empty_dir_from_existing_dir() {
        let path = "./tmp_exist_1";
        create_file(path, None, 0);
        let dir_args = LocalIdentityFromDirectoryArgs {
            identity_directory: Some(path.to_string()),
            identity_prefix: None,
        };

        match dir_args.get_files() {
            Ok(vp) => assert!(vp.is_empty()),
            Err(_) => assert!(false),
        }
        remove_file(path);
    }

    #[test]
    fn pass_get_dir_from_existing_dir() {
        let path = "./tmp_exist_2";
        create_file(path, None, 4);
        let dir_args = LocalIdentityFromDirectoryArgs {
            identity_directory: Some(path.to_string()),
            identity_prefix: None,
        };

        match dir_args.get_files() {
            Ok(vp) => assert_eq!(4, vp.len()),
            Err(_) => assert!(false),
        }
        remove_file(path);
    }

    #[test]
    fn pass_get_path_from_existing_path() {
        let path = "./tmp_exist_3";
        create_file(path, None, 4);
        let id_path = PathBuf::from(format!("{path}/fileid1"));
        let path_args: LocalIdentityArgs = LocalIdentityArgs {
            identity_from_directory: None,
            identity_from_path: Some(id_path),
        };

        let vp = path_args.get_files();
        assert_eq!(1, vp.len());
        remove_file(path);
    }

    fn create_file(dir_name: &str, prefix: Option<String>, num: u32) {
        // create dir if not exist
        fs::create_dir_all(dir_name).unwrap();

        if num > 0 {
            for _n in 1..=num {
                let file_name = match prefix {
                    Some(ref file_prefix) => format!("{file_prefix}{_n}"),
                    None => format!("fileid{_n}"),
                };

                let file_path = PathBuf::from(dir_name).join(file_name);
                fs::write(&file_path, "Hello").unwrap();
            }
        }
    }

    fn remove_file(dir: &str) {
        fs::remove_dir_all(dir).unwrap();
    }
}
