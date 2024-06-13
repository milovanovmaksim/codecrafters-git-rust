pub mod commands;
pub mod objects;

use anyhow::Context;
use clap::{Parser, Subcommand};
use commands::ls_tree;
use commands::{cat_file, hash_object, write_tree};
use std::fs;
#[allow(unused_imports)]
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use crate::commands::commit_tree::{self, write_commit};
use crate::commands::write_tree::write_tree_for;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,
        file: PathBuf,
    },
    LsTree {
        #[clap(long)]
        name_only: bool,
        tree_hash: String,
    },
    WriteTree,
    CommitTree {
        #[clap(short = 'm')]
        message: String,

        #[clap(short = 'p')]
        parrent_hash: Option<String>,

        tree_hash: String,
    },
    Commit {
        #[clap(short = 'm')]
        message: String,
    },
}

fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");
    let args = Args::parse();
    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory");
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => {
            cat_file::invoke(pretty_print, &object_hash)?;
        }
        Command::HashObject { write, file } => {
            let _hash = hash_object::invoke(write, &file)?;
        }
        Command::LsTree {
            name_only,
            tree_hash,
        } => {
            ls_tree::invoke(name_only, tree_hash)?;
        }
        Command::WriteTree => write_tree::invoke()?,
        Command::CommitTree {
            message,
            parrent_hash,
            tree_hash,
        } => {
            commit_tree::invoke(message, parrent_hash, tree_hash)?;
        }
        Command::Commit { message } => {
            let head_ref = std::fs::read_to_string(".git/HEAD").context("git/HEAD")?;
            let Some(head_ref) = head_ref.strip_prefix("ref: ") else {
                anyhow::bail!("refusing to commit onto detached HEAD");
            };
            let head_ref = head_ref.trim();
            let parent_hash = std::fs::read_to_string(format!(".git/{head_ref}"))
                .with_context(|| format!("read HEAD reference target '{head_ref}'"))?;
            let parent_hash = parent_hash.trim();
            let Some(tree_hash) = write_tree_for(Path::new(".")).context("write tree")? else {
                eprintln!("not committing empty tree");
                return Ok(());
            };
            let commit_hash = write_commit(&message, Some(&parent_hash), &hex::encode(tree_hash))
                .context("create commit")?;
            let commit_hash = hex::encode(commit_hash);
            std::fs::write(format!(".git/{head_ref}"), &commit_hash)
                .with_context(|| format!("update HEAD reference target {head_ref}"))?;
            println!("HEAD is now at {}", commit_hash);
        }
    }
    Ok(())
}

// struct LimitReader<R> {
//     reader: R,
//     limit: usize,
// }

// impl<R> Read for LimitReader<R>
// where
//     R: Read
// {
//     fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
//         if buf.len() > self.limit {
//             buf = &mut buf[..self.limit + 1];
//         }
//         let n = self.reader.read(buf)?;
//         if n > self.limit {
//             return Err(io::Error::new(io::ErrorKind::Other, "too amny bytes"));
//         }
//         self.limit -= n;
//         Ok(n)
//     }

// }
