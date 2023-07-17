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

#[derive(Serialize)]
pub struct RepositoryFilePaths {
    pub repo_id: String,
    pub file_paths: Vec<String>
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
    repository: Repository,
    model: &M,
) -> Result<RepositoryEmbeddings> {
    let time = std::time::Instant::now();
    let files: Vec<File> = fetch_repo_files(repository.clone()).await?;
    println!("Time to fetch files: {:?}", time.elapsed());
    let time = std::time::Instant::now();
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
    println!("Time to embed files: {:?}", time.elapsed());
    Ok(RepositoryEmbeddings {
        repo_id: repository.to_string(),
        file_embeddings,
    })
}

async fn fetch_repo_files(repository: Repository) -> Result<Vec<File>> {
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
            if file.is_file() {
                let mut content = String::new();
                let length = content.len();
                //Fails for non UTF-8 files
                match file.read_to_string(&mut content) {
                    Ok(_) => Some(File {
                        path: file.name().split_once("/").unwrap().1.to_string(),
                        content: content,
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

pub async fn fetch_file_content(repository: Repository, path: &str) -> Result<String> {
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
