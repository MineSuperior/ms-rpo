//------------------------------------------------------------//
//                 Copyright (C) MineSuperior                 //
//------------------------------------------------------------//

// configure the clippy linter

#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
#![allow(clippy::println_empty_string)]

//------------------------------------------------------------//

// import standard library modules

use std::io;
use std::io::Write;
use std::fs;
use std::path;
use std::process;

//------------------------------------------------------------//

// import third-party modules

use sha1::Sha1; // sha1 hashing

use rayon::prelude::*; // parallel iterators

use tempdir::TempDir; // temporary directories

//------------------------------------------------------------//

#[allow(dead_code)] // `Folders` is currently unused
enum TraverseDirLookFor {
    All,
    Files,
    Folders,
}

//------------------------------------------------------------//

fn exit_program(
    exit_message: &str,
) -> ! {
    println!("\nExiting Program...\n\n{}", exit_message);
    process::exit(0);
}

fn ask_user_to_confirm(
    should_ask_user_to_confirm: bool,
    prompt: &str,
) -> bool {
    if !should_ask_user_to_confirm {
        return true;
    }

    let response: bool;

    loop {
        println!("{}\nContinue? (Y)es (N)o", prompt);

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        response = match input.trim().to_lowercase().as_str() {
            "y" | "yes" => true,
            "n" | "no" => false,
            _ => {
                continue;
            },
        };

        break;
    }

    return response;
}

fn traverse_dir(
    dir: &path::Path,
    look_for: &TraverseDirLookFor,
    predicate: Option<&dyn Fn(&path::Path) -> bool>,
) -> Vec<path::PathBuf> {
    let mut items: Vec<path::PathBuf> = Vec::new();

    let dir_items = fs::read_dir(dir).expect("Failed to read directory");
    for dir_item in dir_items {
        let item = dir_item.expect("Failed to read item").path();

        if let Some(predicate) = predicate {
            if !predicate(&item) {
                continue;
            }
        }

        match look_for {
            TraverseDirLookFor::All => {
                items.push(item.clone());
            },
            TraverseDirLookFor::Files => {
                if item.is_file() {
                    items.push(item.clone());
                }
            },
            TraverseDirLookFor::Folders => {
                if item.is_dir() {
                    items.push(item.clone());
                }
            },
        }

        if item.is_dir() {
            items.extend(traverse_dir(&item, look_for, predicate));
        }
    }

    return items;
}

fn clone_dir(
    input_dir: &path::Path,
    output_dir: &path::Path,
    predicate: Option<&dyn Fn(&path::Path) -> bool>,
) {
    let traversed_items = traverse_dir(
        &path::PathBuf::from(&input_dir),
        &TraverseDirLookFor::All,
        predicate,
    );

    let operation_start = std::time::Instant::now();

    traversed_items.par_iter().for_each(
        |file| {
            if file.is_dir() {
                // include the directory itself (useful for empty directories)

                fs::create_dir_all(
                    output_dir.join(
                        file.strip_prefix(input_dir).expect("Failed to strip prefix")
                    )
                ).expect("Failed to create directory");

                return;
            }

            let relative_file_path_from_input_dir = file
                .strip_prefix(input_dir).expect("Failed to strip prefix")
                .to_str().expect("Failed to convert relative file path to string");

            let output_file_path = output_dir.join(relative_file_path_from_input_dir);

            // if it does not exist, create it
            fs::create_dir_all(
                output_file_path.parent().expect("Failed to get parent directory")
            ).expect("Failed to create parent directory");

            // println!("Cloning {}", file.to_str().expect("Failed to convert file to string"));

            fs::copy(file, output_file_path).expect("Failed to copy file");
        }
    );

    let operation_end = operation_start.elapsed();

    println!("Cloned {} directory items in {:.2?}", traversed_items.len(), operation_end);
}

fn empty_dir(
    dir: &path::Path,
) {
    let operation_start = std::time::Instant::now();

    let dir_items = fs::read_dir(dir).expect("Failed to read directory");

    let dir_items_vec = dir_items.map(
        |dir_item| return dir_item.expect("Failed to read item").path()
    ).collect::<Vec<path::PathBuf>>();

    dir_items_vec.par_iter().for_each(
        |file| {
            if file.is_dir() {
                fs::remove_dir_all(file).expect("Failed to remove directory");
            }

            if file.is_file() {
                fs::remove_file(file).expect("Failed to remove file");
            }
        }
    );

    let operation_end = operation_start.elapsed();

    println!("Emptied {} directory items in {:.2?}", dir_items_vec.len(), operation_end);
}

//------------------------------------------------------------//

fn minify_json_files(
    input_dir: &path::Path,
    output_dir: &path::Path,
) {
    let traversed_items = traverse_dir(
        &path::PathBuf::from(&input_dir),
        &TraverseDirLookFor::Files,
        None,
    );

    let json_like_files = traversed_items.par_iter().filter(
        // check if the file ends with `.json` or `.mcmeta`
        |file| {
            let s = file.to_str().expect("Failed to convert file to string");
            return s.ends_with(".json") || s.ends_with(".mcmeta");
        }
    ).collect::<Vec<&path::PathBuf>>();

    let operation_start = std::time::Instant::now();

    json_like_files.par_iter().for_each(
        |file| {
            if file.is_dir() {
                return;
            }

            let file_contents = fs::read_to_string(file).expect("Failed to read file");

            let parsed_json = serde_json::from_str::<serde_json::Value>(&file_contents).expect("Failed to parse json");

            let minified_file_contents = serde_json::to_string(&parsed_json).expect("Failed to convert json to string");

            let relative_file_path_from_input_dir = file
                .strip_prefix(input_dir).expect("Failed to strip prefix")
                .to_str().expect("Failed to convert relative file path to string");

            let output_file_path = output_dir.join(relative_file_path_from_input_dir);

            // println!("Minifying {}", file.to_str().expect("Failed to convert file to string"));

            // if it does not exist, create it
            fs::create_dir_all(
                output_file_path.parent().expect("Failed to get parent directory")
            ).expect("Failed to create parent directory");

            fs::write(output_file_path, minified_file_contents).expect("Failed to write file");
        }
    );

    let operation_end = operation_start.elapsed();

    println!("Minified {} json-like files in {:.2?}", json_like_files.len(), operation_end);
}

fn minify_yaml_files(
    input_dir: &path::Path,
    output_dir: &path::Path,
) {
    let traversed_items = traverse_dir(
        &path::PathBuf::from(&input_dir),
        &TraverseDirLookFor::Files,
        None,
    );

    let yaml_like_files = traversed_items.par_iter().filter(
        // check if the file ends with `.yml` or `.yaml`
        |file| {
            let s = file.to_str().expect("Failed to convert file to string");
            return s.ends_with(".yml") || s.ends_with(".yaml");
        }
    ).collect::<Vec<&path::PathBuf>>();

    let operation_start = std::time::Instant::now();

    yaml_like_files.par_iter().for_each(
        |file| {
            if file.is_dir() {
                return;
            }

            let file_contents = fs::read_to_string(file).expect("Failed to read file");

            let parsed_yaml_as_json = serde_yaml::from_str::<serde_json::Value>(&file_contents).expect("Failed to parse yaml");

            let minified_file_contents = serde_json::to_string(&parsed_yaml_as_json).expect("Failed to convert yaml to json");

            let relative_file_path_from_input_dir = file
                .strip_prefix(input_dir).expect("Failed to strip prefix")
                .to_str().expect("Failed to convert relative file path to string");

            let output_file_path = output_dir.join(relative_file_path_from_input_dir);

            // println!("Minifying {}", file.to_str().expect("Failed to convert file to string"));

            // if it does not exist, create it
            fs::create_dir_all(
                output_file_path.parent().expect("Failed to get parent directory")
            ).expect("Failed to create parent directory");

            fs::write(output_file_path, minified_file_contents).expect("Failed to write file");
        }
    );

    let operation_end = operation_start.elapsed();

    println!("Minified {} yaml-like files in {:.2?}", yaml_like_files.len(), operation_end);
}

fn minify_open_gl_sl_files(
    input_dir: &path::Path,
    output_dir: &path::Path,
) {
    let traversed_items = traverse_dir(
        &path::PathBuf::from(&input_dir),
        &TraverseDirLookFor::Files,
        None,
    );

    let open_gl_sl_like_files = traversed_items.par_iter().filter(
        // check if the file ends with `.vsh` or `.fsh`
        |file| {
            let s = file.to_str().expect("Failed to convert file to string");
            return s.ends_with(".vsh") || s.ends_with(".fsh");
        }
    ).collect::<Vec<&path::PathBuf>>();

    let operation_start = std::time::Instant::now();

    open_gl_sl_like_files.par_iter().for_each(
        |file| {
            if file.is_dir() {
                return;
            }

            let file_contents = fs::read_to_string(file).expect("Failed to read file");

            let minified_file_contents: String = file_contents.lines().map(
                |mut line| {
                    // remove comments from lines (including comments at the end of lines)
                    if let Some(index) = line.find("//") {
                        line = line[..index].trim();
                    } else {
                        line = line.trim();
                    }

                    return line;
                }
            ).filter(
                |line| {
                    // remove empty lines
                    if line.is_empty() {
                        return false;
                    }

                    return true;
                }
            ).collect::<Vec<&str>>().join("\n");

            let relative_file_path_from_input_dir = file
                .strip_prefix(input_dir).expect("Failed to strip prefix")
                .to_str().expect("Failed to convert relative file path to string");

            let output_file_path = output_dir.join(relative_file_path_from_input_dir);

            // println!("Minifying {}", file.to_str().expect("Failed to convert file to string"));

            // if it does not exist, create it
            fs::create_dir_all(
                output_file_path.parent().expect("Failed to get parent directory")
            ).expect("Failed to create parent directory");

            fs::write(output_file_path, minified_file_contents).expect("Failed to write file");
        }
    );

    let operation_end = operation_start.elapsed();

    println!("Minified {} open_gl_sl-like files in {:.2?}", open_gl_sl_like_files.len(), operation_end);
}

/**
 * Compresses all png-like files in a directory and outputs them to another directory.
 */
fn compress_png_files(
    input_dir: &path::Path,
    output_dir: &path::Path,
) {
    let traversed_items = traverse_dir(
        &path::PathBuf::from(&input_dir),
        &TraverseDirLookFor::Files,
        None,
    );

    let png_like_files = traversed_items.par_iter().filter(
        // check if the file ends with `.png`
        |file| {
            let s = file.to_str().expect("Failed to convert file to string");
            return s.ends_with(".png");
        }
    ).collect::<Vec<&path::PathBuf>>();

    let operation_start = std::time::Instant::now();

    // this cannot be `par_iter` because `oxipng` can spawn too many threads and lock up the master process
    png_like_files.iter().for_each(
        |file| {
            if file.is_dir() {
                return;
            }

            let relative_file_path_from_input_dir = file
                .strip_prefix(input_dir).expect("Failed to strip prefix")
                .to_str().expect("Failed to convert relative file path to string");

            let output_file_path = output_dir.join(relative_file_path_from_input_dir);

            // if it does not exist, create it
            fs::create_dir_all(
                output_file_path.parent().expect("Failed to get parent directory")
            ).expect("Failed to create parent directory");

            // println!("Optimizing {}", file.to_str().expect("Failed to convert file to string"));

            oxipng::optimize(
                &oxipng::InFile::Path(file.to_path_buf()),
                &oxipng::OutFile::Path(Some(output_file_path.to_path_buf())),
                &oxipng::Options::max_compression(),
            ).expect("Failed to optimize png");
        }
    );

    let operation_end = operation_start.elapsed();

    println!("Compressed {} png-like files in {:.2?}", png_like_files.len(), operation_end);
}

/**
 * Zips up a directory into a specified zip file.
 */
fn zip_dir(
    input_dir: &path::Path,
    output_zip_file_path: &path::Path,
) {
    let zip_file = std::fs::File::create(output_zip_file_path).expect("Failed to create zip file");
    let mut zip_writer = zip::ZipWriter::new(zip_file);

    let traversed_items = traverse_dir(
        &path::PathBuf::from(&input_dir),
        &TraverseDirLookFor::All,
        Some(&|file| return file != output_zip_file_path)
    );

    let operation_start = std::time::Instant::now();

    traversed_items.iter().for_each(
        |item| {
            if !item.is_file() {
                return;
            }

            let relative_file_path_from_input_dir = item
                .strip_prefix(input_dir).expect("Failed to strip prefix")
                .to_str().expect("Failed to convert relative file path to string");

            // println!("Zipping {}", file.to_str().expect("Failed to convert file to string"));

            zip_writer.start_file(
                relative_file_path_from_input_dir,
                zip::write::FileOptions::default()
            ).expect("Failed to start file");

            let file_contents = fs::read(item).expect("Failed to read file");
            zip_writer.write_all(&file_contents).expect("Failed to write file");
        }
    );

    zip_writer.finish().expect("Failed to finish zip");

    let operation_end = operation_start.elapsed();

    println!("Zipped {} files in {:.2?}", traversed_items.len(), operation_end);

    let mut sha1_hasher = Sha1::new();
    let zip_file = std::fs::File::open(output_zip_file_path).expect("Failed to open zip file");

    loop {
        let mut buffer = [0; 1024];
        let bytes_read = io::Read::read(&mut &zip_file, &mut buffer).expect("Failed to read zip file");

        if bytes_read == 0 {
            break;
        }

        sha1_hasher.update(&buffer[..bytes_read]);
    }

    let hash = sha1_hasher.digest().to_string();

    println!("Zip file SHA-1 hash: {}", hash);
}

//------------------------------------------------------------//

fn get_command_line_args() -> clap::ArgMatches {
    let matched_args = clap::Command::new("ms-rpo")
        .author("MineSuperior")
        .arg(
            clap::Arg::new("input_path")
                .short('i')
                .long("input-path")
                .help("The directory to read from")
                .value_name("INPUT_PATH")
                .value_hint(clap::ValueHint::DirPath)
                .value_parser(clap::value_parser!(path::PathBuf))
                .required(true)
        )
        .arg(
            clap::Arg::new("output_path")
                .short('o')
                .long("output-path")
                .help("The directory to output to")
                .value_name("OUTPUT_PATH")
                .value_hint(clap::ValueHint::DirPath)
                .value_parser(clap::value_parser!(path::PathBuf))
                .required(true)
        )
        .arg(
            clap::Arg::new("zip")
                .short('z')
                .long("zip")
                .help("Compresses the output files into a .zip file with an optionally specified name")
                .value_name("ZIP_NAME")
                .value_hint(clap::ValueHint::FilePath)
                .value_parser(clap::value_parser!(path::PathBuf))
                .default_missing_value("output.zip")
                .num_args(0..=1)
                .required(false)
        )
        .arg(
            clap::Arg::new("no_confirm")
                .long("no-confirm")
                .help("Bypass confirmation prompts")
                .action(clap::ArgAction::Set)
                .default_missing_value("false")
                .default_value("true")
                .value_parser(clap::value_parser!(bool))
                .num_args(0)
                .required(false)
        )
        .get_matches();

    return matched_args;
}

//------------------------------------------------------------//

fn main() {
    let matched_args = get_command_line_args();

    println!(""); // empty line

    let input_dir = matched_args.get_one::<path::PathBuf>("input_path").expect("Failed to get input_path");
    println!("input_dir: {}", input_dir.to_str().expect("Failed to convert input_dir to string"));

    let output_dir = matched_args.get_one::<path::PathBuf>("output_path").expect("Failed to get output_path");
    println!("output_dir: {}", output_dir.to_str().expect("Failed to convert output_dir to string"));

    let zip_name = matched_args.get_one::<path::PathBuf>("zip");
    match zip_name {
        Some(zip_name) => {
            println!("zip_name: {}", zip_name.to_str().expect("Failed to convert zip_name to string"));
        },
        None => {
            println!("zip_name: None");
        },
    }

    let should_ask_user_to_confirm = !matched_args.contains_id("no_confirm");
    println!("should_ask_user_to_confirm: {}", should_ask_user_to_confirm);

    println!(""); // empty line

    if !input_dir.exists() || !input_dir.is_dir() {
        exit_program("Input directory does not exist or is not a directory");
    }

    if !output_dir.exists() || !output_dir.is_dir() {
        exit_program("Output directory does not exist");
    }

    // ensure that input_dir is not the same as output_dir
    if input_dir == output_dir {
        exit_program("Input directory is the same as output directory");
    }

    // ensure output_dir is not a subdirectory or a descendant of input_dir
    if output_dir.starts_with(input_dir) {
        exit_program("Output directory is a subdirectory or a descendant of input directory");
    }

    // ensure that the output directory is empty
    let output_dir_contains_items = output_dir.read_dir().expect("Failed to read output directory").next().is_some();
    if output_dir_contains_items {
        let user_confirmed = ask_user_to_confirm(
            should_ask_user_to_confirm,
            format!(
                "Output directory is not empty.\nContinuing will delete all files in:\n{}",
                output_dir.to_str().expect("Failed to convert output_dir to string")
            ).as_str()
        );

        if !user_confirmed {
            exit_program("User did not confirm to continue");
        }

        empty_dir(output_dir);
    }

    // create a temporary directory to work on the files inside of the programs running directory
    let temp_dir = TempDir::new("ms-rpo").expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.path();

    {
        let user_confirmed = ask_user_to_confirm(
            should_ask_user_to_confirm,
            format!(
                "Clone all files in {} into {}",
                input_dir.to_str().expect("Failed to convert input_dir to string"),
                temp_dir_path.to_str().expect("Failed to convert temp_dir to string")
            ).as_str()
        );

        if !user_confirmed {
            exit_program("User did not confirm to continue");
        }

        // clone all input_dir files into the temporary directory
        clone_dir(
            input_dir,
            temp_dir_path,
            // remove `.md` and `.old` files
            // TODO: make this configurable
            Some(
                &|file| {
                    let s = file.to_str().expect("Failed to convert file to string");
                    return !s.ends_with(".md") && !s.ends_with(".old");
                }
            )
        );
    }

    {
        let user_confirmed = ask_user_to_confirm(
            should_ask_user_to_confirm,
            "Minify all .json and .mcmeta files"
        );

        if !user_confirmed {
            exit_program("User did not confirm to continue");
        }

        // modify files in-place (output_dir is the same as input_dir)
        minify_json_files(temp_dir_path, temp_dir_path);
    }

    {
        let user_confirmed = ask_user_to_confirm(
            should_ask_user_to_confirm,
            "Minify all .yml and .yaml files"
        );

        if !user_confirmed {
            exit_program("User did not confirm to continue");
        }

        // modify files in-place (output_dir is the same as input_dir)
        minify_yaml_files(temp_dir_path, temp_dir_path);
    }

    {
        let user_confirmed = ask_user_to_confirm(
            should_ask_user_to_confirm,
            "Minify all .vsh and .fsh files"
        );

        if !user_confirmed {
            exit_program("User did not confirm to continue");
        }

        // modify files in-place (output_dir is the same as input_dir)
        minify_open_gl_sl_files(temp_dir_path, temp_dir_path);
    }

    {
        let user_confirmed = ask_user_to_confirm(
            should_ask_user_to_confirm,
            "Compress all .png files"
        );

        if !user_confirmed {
            exit_program("User did not confirm to continue");
        }

        // modify files in-place (output_dir is the same as input_dir)
        compress_png_files(temp_dir_path, temp_dir_path);
    }

    match zip_name {
        Some(zip_name) => {
            let zip_file_path = output_dir.join(zip_name);

            let user_confirmed = ask_user_to_confirm(
                should_ask_user_to_confirm,
                format!(
                    "Zip all files and output to {}",
                    &zip_file_path.to_str().expect("Failed to convert zip_file_path to string")
                ).as_str()
            );

            if !user_confirmed {
                exit_program("User did not confirm to continue");
            }

            zip_dir(temp_dir_path, &zip_file_path);
        },
        None => {
            // copy all files from the temporary directory to the output directory
            clone_dir(temp_dir_path, output_dir, None);
        },
    }

    {
        let user_confirmed = ask_user_to_confirm(
            should_ask_user_to_confirm,
            format!(
                "Delete temporary directory {}",
                temp_dir_path.to_str().expect("Failed to convert temp_dir to string")
            ).as_str()
        );

        if !user_confirmed {
            exit_program("User did not confirm to continue");
        }

        println!("Deleting temporary directory {}...", temp_dir_path.to_str().expect("Failed to convert temp_dir to string"));
        temp_dir.close().expect("Failed to remove temporary directory");
    }

    println!(""); // empty line

    println!("Exiting...");
}
