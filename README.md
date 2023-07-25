<div align="center">
  <img alt="OpenSauced logo" src="https://i.ibb.co/7jPXt0Z/logo1-92f1a87f.png" width="300px">
  <h1>🍕 RepoQuery 🍕</h1>
<p align="center">
  <img src="https://img.shields.io/github/languages/code-size/open-sauced/repo-query" alt="GitHub code size in bytes">
  <img src="https://img.shields.io/github/commit-activity/w/open-sauced/repo-query" alt="GitHub commit activity">
  <a href="https://discord.gg/U2peSNf23P">
    <img src="https://img.shields.io/discord/714698561081704529.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2" alt="Discord">
  </a>
  <a href="https://twitter.com/saucedopen">
    <img src="https://img.shields.io/twitter/follow/saucedopen?label=Follow&style=social" alt="Twitter">
  </a>
</p>

<img alt="Open Sauced" src="https://user-images.githubusercontent.com/46051506/255266242-0b94b779-3020-455e-8a26-d3ed8e292214.gif" width="600px">


<p><h3>A REST service to answer user-queries about public GitHub repositories</h3></p>

</div>

## 🔎 The Project
RepoQuery is an early-beta project, that uses recursive [OpenAI function calling](https://platform.openai.com/docs/api-reference/chat/create#chat/create-functions) paired with semantic search using [All-MiniLM-L6-V2](https://huggingface.co/rawsh/multi-qa-MiniLM-distill-onnx-L6-cos-v1/blob/main/onnx/model_quantized.onnx) to index and answer user queries about public GitHub repositories.

##  📬 Service Endpoints

> **Note:**
Since the service returns responses as SSEs, a REST client like Postman is recommended. Download it [here](https://www.postman.com/downloads/). The Postman web client doesn't support requests to `localhost`.

[![Run in Postman](https://run.pstmn.io/button.svg)](https://app.getpostman.com/run-collection/18073744-276b793e-f5ec-418f-ba0a-9dff94af543e?action=collection%2Ffork&source=rip_markdown&collection-url=entityId%3D18073744-276b793e-f5ec-418f-ba0a-9dff94af543e%26entityType%3Dcollection%26workspaceId%3D8d8a1363-ad0a-45ad-b036-ef6a37e44ef8)

### 1. `POST /embed`
To generate and store embeddings for a GitHub repository.

#### Parameters
The parameters are passed as a JSON object in the request body:

- `owner` (string, required): The owner of the repository.
- `name` (string, required): The name of the repository.
- `branch` (string, required): The name of the branch.

#### Response
The request is processed by the server and responses are sent as [Server-sent events(SSE)](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events). The event stream will contain the following events with optional data. https://github.com/open-sauced/repo-query/blob/f2f415a4fa9c02d4530624fd7bac2105eea1a77c/src/routes/events.rs#L14-L20

#### Example
```bash
curl --location 'localhost:3000/embed' \
--header 'Content-Type: application/json' \
--data '{
    "owner": "open-sauced",
    "name": "ai",
    "branch": "beta"
}'
```

### 2. `POST /query`
To perform a query on the API with a specific question related to a repository.

#### Parameters
The parameters are passed as a JSON object in the request body:

- `query` (string, required): The question or query you want to ask.
- `repository` (object, required): Information about the repository for which you want to get the answer.
  - `owner` (string, required): The owner of the repository.
  - `name` (string, required): The name of the repository.
  - `branch` (string, required): The name of the branch.

#### Response
The request is processed by the server and responses are sent as [Server-sent events(SSE)](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events). The event stream will contain the following events with optional data. https://github.com/open-sauced/repo-query/blob/f2f415a4fa9c02d4530624fd7bac2105eea1a77c/src/routes/events.rs#L22-L29


#### Example
```bash
curl --location 'localhost:3000/query' \
--header 'Content-Type: application/json' \
--data '{
    "query": "How is the PR description being generated using AI?",
    "repository": {
        "owner": "open-sauced",
        "name": "ai",
        "branch": "beta"
    }
}'
```

### 3. `GET /collection`
To check if a repository has been indexed.

#### Parameters
- `owner` (string, required): The owner of the repository.
- `name` (string, required): The name of the repository.
- `branch` (string, required): The name of the branch.

#### Response
This endpoint returns an `OK` status code if the repository has been indexed by the service.

#### Example
```bash
curl --location 'localhost:3000/embed?owner=open-sauced&name=ai&branch=beta'
```

## 🧪 Running Locally

To run the project locally, there are a few prerequisites:

- The [Rust toolchain](https://www.rust-lang.org/learn/get-started)
- The [Onnx Runtime](https://onnxruntime.ai/docs/install/). Will be downloaded and installed automatically when building the project.
- [Docker](https://docs.docker.com/engine/install/) to run the [QdrantDB](https://qdrant.tech/) instance.

Once, the above requirements are satisfied, you can run the project like so:

### Environment variables

The project requires the following environment variables to be set.
* [`OPENAI_API_KEY`](https://platform.openai.com/account/api-keys). To authenticate requests to OpenAI. 

### Database setup

Start Docker and run the following commands to spin-up a Docker container with a QdrantDB image.
```
docker pull qdrant/qdrant
docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant
```
The database dashboard will be accessible at [localhost:6333/dashboard](http://localhost:6333/dashboard), the project communicates with the DB on port `6334`.

### Running the project

Run the following command to install the dependencies and run the project on port `3000`.

```
cargo run --release
```
This command will build and run the project with optimizations enabled(Highly recommended).

## Attributions

https://sbert.net for https://huggingface.co/sentence-transformers/multi-qa-MiniLM-L6-cos-v1.
```
@inproceedings{reimers-2020-multilingual-sentence-bert,
  title = "Making Monolingual Sentence Embeddings Multilingual using Knowledge Distillation",
  author = "Reimers, Nils and Gurevych, Iryna",
  booktitle = "Proceedings of the 2020 Conference on Empirical Methods in Natural Language Processing",
  month = "11",
  year = "2020",
  publisher = "Association for Computational Linguistics",
  url = "https://arxiv.org/abs/2004.09813",
}
```

## 🤝 Contributing

We encourage you to contribute to OpenSauced! Please check out the [Contributing guide](https://docs.opensauced.pizza/contributing/introduction-to-contributing/) for guidelines about how to proceed.

We have a commit utility called [@open-sauced/conventional-commit](https://github.com/open-sauced/conventional-commit) that helps you write your commits in a way that is easy to understand and process by others.

## 🍕 Community

Got Questions? Join the conversation in our [Discord](https://discord.gg/U2peSNf23P).  
Find Open Sauced videos and release overviews on our [YouTube Channel](https://www.youtube.com/channel/UCklWxKrTti61ZCROE1e5-MQ).

## ⚖️ LICENSE

MIT © [Open Sauced](LICENSE)
