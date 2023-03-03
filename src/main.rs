#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;

#[derive(Parser)]
#[command(name = "MyApp")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create an empty Git repository or reinitialize an existing one
    Init { directory: Option<String> },
    /// Provide content or type and size information for repository objects
    #[command(name = "cat-file")]
    CatFile {
        /// Pretty-print the contents of <object> based on its type.
        ///  
        /// ## <type>
        ///
        /// Typically this matches the real type of <object> but asking for a type that can be trivially
        /// dereferenced from the given <object> is also permitted. An example is to ask for a "tree"
        /// with <object> being a commit object that contains it, or to ask for a "blob" with <object>
        /// being a tag object that points at it.

        #[arg(
            short = 'p', 
            value_name = "TYPE", 
            require_equals = true,
            num_args = 0..=1, 
            default_value_t = PrettyPrint::Auto,
            default_missing_value = "none",
            value_enum, 
        )]
        object_type: PrettyPrint,

        /// The name of the object to show.
        object: String,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum PrettyPrint {
    None,
    Auto,
    Blob,
    Tree,
    Commit,
}

fn main() {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    match &cli.command {
        Some(Commands::Init { directory }) => run_init(directory).unwrap(),
        Some(Commands::CatFile {
            object,
            object_type,
        }) => run_cat_file(object, object_type).unwrap(),
        None => {}
    }
}

fn run_init(directory: &Option<String>) -> Result<(), io::Error> {
    if let Some(d) = directory {
        env::set_current_dir(d).unwrap()
    }

    if !Path::new(".git").exists() {
        fs::create_dir(".git")?;
        fs::create_dir(".git/objects")?;
        fs::create_dir(".git/refs")?;
        fs::write(".git/HEAD", "ref: refs/heads/master\n")?;
    }

    Ok(())
}

fn run_cat_file(object: &String, _object_type: &PrettyPrint) -> Result<(), io::Error> {
    let prefix = object.get(0..2).unwrap();
    let rest = object.get(2..).unwrap();
    let file_path = format!(".git/objects/{}/{}", prefix, rest);

    let path = Path::new(&file_path);
    if path.exists() {
        let contents = BufReader::new(File::open(path).unwrap());
        let mut decoder = ZlibDecoder::new(contents);
        let mut s = String::new();
        decoder.read_to_string(&mut s)?;

        let obj = GitObject::parse(s).unwrap();

        match obj {
            GitObject::Blob {
                contents,
                byte_size: _,
            } => println!("{}", contents),
        };
    }
    Ok(())
}

#[derive(Debug)]
enum GitObject {
    Blob { byte_size: String, contents: String },
}

impl GitObject {
    fn parse(input: String) -> Result<GitObject, String> {
        let split = input.split_once("\0");
        match split {
            Some((header, contents)) => match header.split_once(" ") {
                Some(("blob", byte_size)) => Ok(GitObject::Blob {
                    contents: contents.to_string(),
                    byte_size: byte_size.to_string(),
                }),
                _ => Err("failed to parse object: unknown type".to_string())
            },
            _ => Err("failed to parse object: unknown type".to_string()),
        }
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}

#[test]
fn test_prefix() {
    let s = "0052b8a91ee9ed9bee642188cc0cf67487ab201c";
    let actual = s.get(0..2);
    assert_eq!(actual.unwrap(), "00");
}

#[test]
fn test_rest() {
    let s = "0052b8a91ee9ed9bee642188cc0cf67487ab201c";
    let actual = s.get(2..);
    assert_eq!(actual.unwrap(), "52b8a91ee9ed9bee642188cc0cf67487ab201c");
}
