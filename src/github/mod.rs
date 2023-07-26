#![allow(unused_must_use)]
use crate::{
    embeddings::{Embeddings, EmbeddingsModel},
    prelude::*,
};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Debug, Default, Serialize)]
pub struct File {
    pub path: String,
    pub content: String,
    pub length: usize,
}

impl ToString for File {
    fn to_string(&self) -> String {
        format!(
            "File path: {}\nFile length: {} bytes\nFile content: {}",
            &self.path, &self.length, &self.content
        )
    }
}

#[derive(Debug, Clone)]
pub struct FileEmbeddings {
    pub path: String,
    pub embeddings: Embeddings,
}

#[derive(Debug)]
pub struct RepositoryEmbeddings {
    pub repo_id: String,
    pub file_embeddings: Vec<FileEmbeddings>,
}

#[derive(Serialize, Debug)]
pub struct RepositoryFilePaths {
    pub repo_id: String,
    pub file_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub owner: String,
    pub name: String,
    pub branch: String,
}

impl ToString for Repository {
    fn to_string(&self) -> String {
        format!("{}-{}-{}", &self.owner, &self.name, &self.branch)
    }
}

pub async fn embed_repo<M: EmbeddingsModel + Send + Sync>(
    repository: &Repository,
    files: Vec<File>,
    model: &M,
) -> Result<RepositoryEmbeddings> {
    let file_embeddings: Vec<FileEmbeddings> = files
        .into_par_iter()
        .filter_map(|file| {
            let embed_content = file.to_string();
            let embeddings = model.embed(&embed_content).unwrap();
            Some(FileEmbeddings {
                path: file.path,
                embeddings,
            })
        })
        .collect();

    Ok(RepositoryEmbeddings {
        repo_id: repository.to_string(),
        file_embeddings,
    })
}

pub async fn fetch_repo_files(repository: &Repository) -> Result<Vec<File>> {
    let Repository {
        owner,
        name,
        branch,
    } = repository;

    let url = format!("https://github.com/{owner}/{name}/archive/{branch}.zip");
    let response = reqwest::get(url).await?.bytes().await?;

    let reader = std::io::Cursor::new(response);
    let mut archive = zip::ZipArchive::new(reader)?;

    let files: Vec<File> = (0..archive.len())
        .filter_map(|file| {
            let mut file = archive.by_index(file).unwrap();
            let file_path = file.name().split_once('/').unwrap().1.to_string();

            if file.is_file() && should_index(&file_path) {
                let mut content = String::new();
                let length = content.len();
                //Fails for non UTF-8 files
                match file.read_to_string(&mut content) {
                    Ok(_) => Some(File {
                        path: file_path,
                        content,
                        length,
                    }),
                    Err(_) => None,
                }
            } else {
                None
            }
        })
        .collect();
    Ok(files)
}

pub async fn fetch_file_content(repository: &Repository, path: &str) -> Result<String> {
    let Repository {
        owner: repo_owner,
        name: repo_name,
        branch: repo_branch,
    } = repository;
    let url =
        format!("https://raw.githubusercontent.com/{repo_owner}/{repo_name}/{repo_branch}/{path}");
    let response = reqwest::get(url).await?;
    if response.status() == reqwest::StatusCode::OK {
        let content = response.text().await?;
        Ok(content)
    } else {
        Err(anyhow::anyhow!("Unable to fetch file content"))
    }
}

const IGNORED_EXTENSIONS: &[&str] = &[
    "bpg", "eps", "pcx", "ppm", "tga", "tiff", "wmf", "xpm", "svg", "ttf", "woff2", "fnt", "fon",
    "otf", "pdf", "ps", "dot", "docx", "dotx", "xls", "xlsx", "xlt", "lock", "odt", "ott", "ods",
    "ots", "dvi", "pcl", "mod", "jar", "pyc", "war", "ear", "bz2", "xz", "rpm", "coff", "obj",
    "dll", "class", "log",
];

const IGNORED_DIRECTORIES: &[&str] = &[
    "vendor",
    "dist",
    "build",
    "target",
    "bin",
    "obj",
    "node_modules",
    "debug",
];

pub fn should_index(path: &str) -> bool {
    !(IGNORED_EXTENSIONS.iter().any(|ext| path.ends_with(ext))
        || IGNORED_DIRECTORIES.iter().any(|dir| path.contains(dir)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_repo_files() {
        let repository = Repository {
            owner: "open-sauced".to_string(),
            name: "ai".to_string(),
            branch: "beta".to_string(),
        };

        let result = fetch_repo_files(&repository).await;

        // Assert that the function returns a Result containing a vector of File
        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(!files.is_empty());
    }

    #[tokio::test]
    async fn test_fetch_file_content() {
        let repository = Repository {
            owner: "open-sauced".to_string(),
            name: "ai".to_string(),
            branch: "beta".to_string(),
        };
        let path = "package.json";

        let result = fetch_file_content(&repository, path).await;

        // Assert that the function returns a Result containing the file content
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(!content.is_empty());

        let path = "Some_Invalid_File.example";

        let result = fetch_file_content(&repository, path).await;

        //Assert that the function returns Err for an invalid file path
        assert!(result.is_err());
    }

    #[test]
    fn test_should_index() {
        // Test with ignored extensions
        for ext in IGNORED_EXTENSIONS {
            let path = format!("path/to/file.{}", ext);
            assert!(!should_index(&path));
        }

        // Test with ignored directories
        for dir in IGNORED_DIRECTORIES {
            let path = format!("path/to/{}/file.txt", dir);
            assert!(!should_index(&path));
        }

        // Test with valid path
        let path = "path/to/file.tsx";
        assert!(should_index(path));
    }
}
