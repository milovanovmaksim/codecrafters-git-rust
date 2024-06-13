use anyhow::Context;
#[allow(unused_imports)]
use std::fs;

use crate::objects::{Kind, Object};

pub(crate) fn invoke(pretty_print: bool, object_hash: &str) -> anyhow::Result<()> {
    anyhow::ensure!(pretty_print, "mode must be given -p, and we ");
    // TODO: support shortest-unique object hashes
    let mut object = Object::read(object_hash).context("parses out blob object file")?;
    match object.kind {
        Kind::Blob => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let n = std::io::copy(&mut object.reader, &mut stdout)
                .context("write .git/object file to stdout")?;
            anyhow::ensure!(
                n == object.expected_size,
                ".git/objects file was not the expected size (expected: {}, actual {n})",
                object.expected_size
            );
            Ok(())
        }
        _ => {
            anyhow::bail!("do not yet know how to print '{}'", object.kind)
        }
    }
}
