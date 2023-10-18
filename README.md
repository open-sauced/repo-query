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
  
![263793091-872fbaf0-c33f-4c4f-a86e-5bd7f5b4dad7](https://github.com/open-sauced/repo-query/assets/46051506/82d1bf17-9d0e-42f8-b346-c402d269cbfa)



<p><h3>A REST service to answer user-queries about public GitHub repositories</h3></p>

</div>

## 🔎 The Project
RepoQuery is an early-beta project, that uses recursive [OpenAI function calling](https://platform.openai.com/docs/api-reference/chat/create#chat/create-functions) paired with semantic search using [sentence-transformers/all-MiniLM-L6-v2](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2) to index and answer user queries about public GitHub repositories.

## 📬 Service Endpoints

> **Note:**
> Since the service returns responses as SSEs, a REST client like Postman is recommended. Download it [here](https://www.postman.com/downloads/). The Postman web client doesn't support requests to `localhost`.

[![Run in Postman](https://run.pstmn.io/button.svg)](https://app.getpostman.com/run-collection/18073744-276b793e-f5ec-418f-ba0a-9dff94af543e?action=collection%2Ffork&source=rip_markdown&collection-url=entityId%3D18073744-276b793e-f5ec-418f-ba0a-9dff94af543e%26entityType%3Dcollection%26workspaceId%3D8d8a1363-ad0a-45ad-b036-ef6a37e44ef8)

| Endpoint             | Method | Description                                   |
|----------------------|--------|-----------------------------------------------|
| `/`                  | GET    | Redirects to the configured [redirect URL](https://github.com/open-sauced/repo-query/blob/afc4d19068e7c84a2566dae9598f1500f1191705/src/constants.rs#L12).          |
| `/embed`             | POST   | Generate and store embeddings for a GitHub repository.          |
| `/query`             | POST   | Perform a query on the API with a specific question related to a repository. |
| `/collection`        | GET    | Check if a repository has been indexed.      |

### 1. `/embed`

#### Parameters

The parameters are passed as a JSON object in the request body:

- `owner` (string, required): The owner of the repository.
- `name` (string, required): The name of the repository.
- `branch` (string, required): The name of the branch.

#### Response

The request is processed by the server and responses are sent as [Server-sent events(SSE)](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events). The event stream will contain [events](https://github.com/open-sauced/repo-query/blob/afc4d19068e7c84a2566dae9598f1500f1191705/src/routes/events.rs#L14-L21) with optional data.

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

### 2. `/query`

#### Parameters

The parameters are passed as a JSON object in the request body:

- `query` (string, required): The question or query you want to ask.
- `repository` (object, required): Information about the repository for which you want to get the answer.
  - `owner` (string, required): The owner of the repository.
  - `name` (string, required): The name of the repository.
  - `branch` (string, required): The name of the branch.

#### Response

The request is processed by the server and responses are sent as [Server-sent events(SSE)](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events). The event stream will contain [events](https://github.com/open-sauced/repo-query/blob/afc4d19068e7c84a2566dae9598f1500f1191705/src/routes/events.rs#L23-L32) with optional data.

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

### 3. `/collection`

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
- `make` for easy automation and development workflow

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

## 🐳 Docker container

The repo-query engine can also be run locally via a docker container and
includes all the necessary dependencies.

To build the container tagged as `open-sauced-repo-query:latest`, run:

```
make local-image
```

Then, you can start the repo-query service with:
```
docker run --env-file ./.env -p 3000:3000 open-sauced-repo-query
```

There's also a `docker-compose.yaml` file that can be used to start
both qdrant and the repo-query engine together.

To build the image and then start the services, run:

```
make up
```

## Attributions

[sentence-transformers/all-MiniLM-L6-v2](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2).
```
@inproceedings{reimers-2019-sentence-bert,
  title = "Sentence-BERT: Sentence Embeddings using Siamese BERT-Networks",
  author = "Reimers, Nils and Gurevych, Iryna",
  booktitle = "Proceedings of the 2019 Conference on Empirical Methods in Natural Language Processing",
  month = "11",
  year = "2019",
  publisher = "Association for Computational Linguistics",
  url = "https://arxiv.org/abs/1908.10084",
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
