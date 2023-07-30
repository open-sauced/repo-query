#![allow(unused_must_use)]
use crate::{
    embeddings::{Embeddings, EmbeddingsModel},
    prelude::*,
    utils::functions::clean_chunks,
};
use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{io::Read, time::Instant, ops::RangeInclusive, collections::HashSet};

#[derive(Debug, Default, Serialize)]
pub struct File {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Default, Serialize)]
pub struct FileChunk {
    pub path: String,
    pub chunk_content: String,
}

impl ToString for FileChunk {
    fn to_string(&self) -> String {
        format!(
            "File path: {}\nFile content: {}",
            &self.path, &self.chunk_content
        )
    }
}

#[derive(Debug, Clone)]
pub struct FileEmbeddings {
    pub path: String,
    pub content: String,
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
    pub file_paths: HashSet<String>,
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

pub fn embed_repo<M: EmbeddingsModel + Send + Sync>(
    repository: &Repository,
    files: Vec<File>,
    model: &M,
    chunk_limit: RangeInclusive<usize>
) -> Result<RepositoryEmbeddings> {
    let now = Instant::now();
    let chunked_files = chunk_files(files, chunk_limit);
    let file_embeddings: Vec<FileEmbeddings> = chunked_files
        .into_par_iter()
        .filter_map(|file_chunk| {
            let embed_content = file_chunk.to_string();
            let embeddings = model.embed(&embed_content).unwrap();
            Some(FileEmbeddings {
                path: file_chunk.path,
                content: file_chunk.chunk_content,
                embeddings,
            })
        })
        .collect();
    println!("Time taken to embed: {}", now.elapsed().as_secs());
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
                //Fails for non UTF-8 files
                match file.read_to_string(&mut content) {
                    Ok(_) => Some(File {
                        path: file_path,
                        content,
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

fn chunk_files(files: Vec<File>, chunk_limit: RangeInclusive<usize>) -> Vec<FileChunk> {
    let files: Vec<FileChunk> = files
        .par_iter()
        .flat_map(|file| {
            let splitter = text_splitter::TextSplitter::default().with_trim_chunks(true);
            let chunks: Vec<&str> = splitter
                .chunks(&file.content, chunk_limit.clone())
                .collect();
            let cleaned_chunks: Vec<String> = clean_chunks(chunks);
            cleaned_chunks.into_par_iter().map(|chunk| FileChunk {
                path: file.path.clone(),
                chunk_content: chunk,
            })
        })
        .collect();
    files
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

const ALLOWED_LICENSES: &[&str] = &[
    "0bsd",
    "apache-2.0",
    "bsd-2-clause",
    "bsd-3-clause",
    "bsd-3-clause-clear",
    "bsd-4-clause",
    "isc",
    "mit",
    "unlicense",
    "wtfpl",
    "zlib",
];

#[derive(Serialize, Debug, Default)]
pub struct LicenseFetchResponse {
    pub permissible: bool,
    pub error: Option<Value>,
}

pub async fn fetch_license_info(repository: &Repository) -> Result<LicenseFetchResponse> {
    let Repository { owner, name, .. } = repository;
    let url = format!("https://api.github.com/repos/{owner}/{name}/license");

    //User-agent reference: https://docs.github.com/en/rest/overview/resources-in-the-rest-api?apiVersion=2022-11-28#user-agent-required
    let client = reqwest::Client::builder()
        .user_agent("open-sauced")
        .build()
        .unwrap();

    let response = client.get(url).send().await?;
    match response.error_for_status() {
        Ok(response) => {
            let response_json = response.json::<Value>().await?;
            let license_key = response_json["license"]["key"].as_str().unwrap_or_default();
            let permissible: bool = ALLOWED_LICENSES.iter().any(|k| k.eq(&license_key));

            Ok(LicenseFetchResponse {
                permissible,
                error: if permissible {
                    None
                } else {
                    Some(json! {{
                        "message": "Impermissible repository license",
                        "license": {
                            "name": response_json["license"]["name"],
                            "url": response_json["html_url"]
                        }
                    }})
                },
            })
        }
        Err(_) => Err(anyhow::anyhow!("Unable to fetch repository license")),
    }
}

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

    #[tokio::test]
    async fn test_is_indexing_allowed() {
        // Permissible
        let repository = Repository {
            owner: "open-sauced".to_string(),
            name: "ai".to_string(),
            branch: "beta".to_string(),
        };

        let license_info = fetch_license_info(&repository).await.unwrap_or_default();
        assert_eq!(license_info.permissible, true);

        //Permissible
        let repository = Repository {
            owner: "facebook".to_string(),
            name: "react".to_string(),
            branch: "main".to_string(),
        };

        let license_info = fetch_license_info(&repository).await.unwrap_or_default();
        assert_eq!(license_info.permissible, true);

        //Impermissible
        let repository = Repository {
            owner: "open-sauced".to_string(),
            name: "guestbook".to_string(),
            branch: "main".to_string(),
        };

        let license_info = fetch_license_info(&repository).await.unwrap_or_default();
        assert_eq!(license_info.permissible, false);
    }
}
