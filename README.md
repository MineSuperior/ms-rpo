# MineSuperior | Resource Pack Optimizer

## What is this?

This is a command line interface tool used by MineSuperior to optimize the file-size and compression of a Minecraft (Java Edition) resource pack.

## Why does this exist?

Minecraft resource packs are often space inefficient and poorly compressed.

This tool aims to solve that problem by optimizing file sizes and compression of the archive.

## How does it work?

This tool works by performing the following steps:

1. Copy all files from the input directory into a temporary directory.

2. Remove unnecessary files from the temporary directory.

    - Remove `*.old`, `*.md` files.

3. Compress / minify files in the temporary directory.

    - Minify json-like `*.json`, `*.mcmeta` files.

    - Minify yaml-like `*.yaml`, `*.yml` files.

    - Minify open gl shader library `*.fsh`, `*.vsh` files.

    - Compress png-like `*.png` files.

4. Optionally, create a zip archive of the temporary directory.

5. Output the processed files (or zip archive) to the output directory.

6. Cleanup of the temporary directory.

## How do I use it?

### Prerequisites

- [Rust](https://www.rust-lang.org/).

- Cargo (included with Rust).

- A basic understanding of the command line / terminal.

### Compile

1. Clone this repository.

2. Run `cargo build --release` in the root directory of this project.

3. The compiled binary will be located at `./target/release/ms-rpo`.

### Optional Steps

1. Rename the compiled binary to `ms-rpo` (consistent with examples).

2. Add the compiled binary to your `PATH`.

3. Reload your shell / terminal to apply the changes.

### Run

1. First, read the warnings in the [caution](#caution) section below.

2. Run `ms-rpo --help` to see the available options.

3. Run `ms-rpo` with the desired options.

### Example

Note: This only works if you added the compiled binary to your `PATH`.

```bash
ms-rpo -i "./test/input" -o "./test/output" -z "optimized-resource-pack.zip" --no-confirm
```

## Common Flags

| Flag            | Value                                   | Description                                              |
|-----------------|-----------------------------------------|----------------------------------------------------------|
| `-h` `--help`   |                                         | Show the help message.                                   |
| `-i` `--input`  | A path to a folder                      | The input directory.                                     |
| `-o` `--output` | A path to a folder                      | The output directory.                                    |
| `-z` `--zip`    | If provided, a file name with extension | Optionally, output as a zip file with the provided name. |
| `--no-confirm`  |                                         | Skip confirmation prompts.                               |

## Caution

- :warning: The `--no-confirm` flag will not prompt for confirmation before performing destructive operations.

- :warning: By default, the output directory is deleted every time the program is run.

## License

This project is licensed under the [MIT License](./LICENSE.md).
