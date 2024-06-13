use std::{env, io::Cursor};

use crate::objects::{Kind, Object};
use anyhow::Context;
use std::fmt::Write;

pub(crate) fn write_commit(
    message: &str,
    parent_hash: Option<&str>,
    tree_hash: &str,
) -> anyhow::Result<[u8; 20]> {
    let mut commit = String::new();
    writeln!(commit, "tree {tree_hash}")?;
    if let Some(parent_hash) = parent_hash {
        writeln!(commit, "parent {parent_hash}")?;
    };
    let (name, email) =
        if let (Some(name), Some(email)) = (env::var_os("NAME"), env::var_os("EMAIL")) {
            let name = name
                .into_string()
                .map_err(|_| anyhow::anyhow!("$NAME is invalid utf-8"))?;
            let email = email
                .into_string()
                .map_err(|_| anyhow::anyhow!("$EMAIL is nvalid utf-8"))?;
            (name, email)
        } else {
            (
                String::from("milovanovmaksim"),
                String::from("milovanov160386@gmail.com"),
            )
        };
    let time = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .context("current system time is before UNIX epoch")?
        .as_secs();
    writeln!(commit, "author {name} <{email}> {time} +0000")?;
    writeln!(commit, "commiter {name} <{email}> {time} +0000")?;
    writeln!(commit, "")?;
    writeln!(commit, "{message}")?;

    let hash = Object {
        kind: Kind::Commit,
        expected_size: commit.len() as u64,
        reader: Cursor::new(commit),
    }
    .write_to_objects()
    .context("write commit object")?;
    Ok(hash)
}

pub(crate) fn invoke(
    message: String,
    parent_hash: Option<String>,
    tree_hash: String,
) -> anyhow::Result<()> {
    let hash = write_commit(&message, parent_hash.as_deref(), &tree_hash)?;
    println!("{}", hex::encode(hash));

    Ok(())
}
