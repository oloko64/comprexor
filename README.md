# Comprexor

Comprexor is a Rust library for compressing and decompressing files and folders. It uses the popular GZip implementation.

## Usage

### Compressing

You can use the same function to compress a file or a folder. The output will be a `.tar.gz` file.

In the following example we are compressing a folder called `some-folder-or-file` and saving the output to `output.tar.gz`.

```rust
use comprexor::{CompressionLevel, Compressor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let compressor = Compressor::new("./some-folder-or-file", "./output.tar.gz");
    let compress_info = compressor.compress(CompressionLevel::Maximum)?;

    dbg!(&compress_info.input_size_formatted());
    dbg!(&compress_info.output_size_formatted());
    dbg!(&compress_info.ratio_formatted(5));
}
```

This will create a file called `output.tar.gz` in the current directory.

### Extracting

You can use the same function to extract a file or a folder. The output will be a folder or a file depending on the input.

In the following example we are decompressing a file called `some-folder-or-file.tar.gz`, which was created in the previous example, and saving the output to `output`.

```rust
use comprexor::Extractor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let extractor = Extractor::new("./some-folder-or-file.tar.gz", "./output");
    let extract_info = extractor.extract()?;

    dbg!(&extract_info.input_size_formatted());
    dbg!(&extract_info.output_size_formatted());
    dbg!(&extract_info.ratio_formatted(5));
}
```

This will create a folder called `output` in the current directory, which contains the decompressed files.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
