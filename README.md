# Embedding-generation-proto

A Rust-service to generate sentence embeddings using [All-MiniLM-L6-V2](https://huggingface.co/rawsh/multi-qa-MiniLM-distill-onnx-L6-cos-v1/blob/main/onnx/model_quantized.onnx).

## Installation Instructions

To run this repository, you will need to have the Rust toolchain and ONNX Runtime installed. Please follow the steps below to set up your environment:

1. **Install ONNX Runtime:** Before proceeding, make sure you have ONNX Runtime installed on your system. You can find the installation instructions at [onnxruntime.ai/docs/install](https://onnxruntime.ai/docs/install/). If you are using macOS and have Homebrew installed, you can install ONNX Runtime using the following command:
```
brew install onnxruntime
```

3. **Set up Environment Variables:** This project relies on the `ort` Rust crate, which requires two environment variables to be set:
- `ORT_STRATEGY`: Set this variable to `system`.
- `ORT_LIB_LOCATION`: Set this variable to the installation path of ONNX Runtime. To determine the installation path, execute the following command in your macOS terminal:
  ```
  brew --prefix onnxruntime
  ```
  The output should resemble `/opt/homebrew/opt/onnxruntime`.

On macOS, you can set these environment variables by executing the following commands in your terminal:
```
export ORT_STRATEGY=system
export ORT_LIB_LOCATION=/opt/homebrew/opt/onnxruntime
```

3. **Run the Project:** Once the environment variables are configured, navigate to the project's directory and execute:
```
cargo run --release
```

This command will build and run the project with optimizations enabled(Highly recommended).
