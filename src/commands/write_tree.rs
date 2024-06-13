use std::{
    cmp::Ordering,
    fs,
    io::Cursor,
    os::unix::{fs::PermissionsExt, prelude::OsStrExt},
    path::Path,
};

use anyhow::Context;

use crate::objects::{Kind, Object};

pub(crate) fn write_tree_for(path: &Path) -> anyhow::Result<Option<[u8; 20]>> {
    let mut dir =
        fs::read_dir(path).with_context(|| format!("open directory {}", path.display()))?;

    let mut entries = Vec::new();
    while let Some(entry) = dir.next() {
        let entry = entry.with_context(|| format!("bad directory entry in {}", path.display()))?;
        let name = entry.file_name();
        let meta = entry.metadata().context("metadata for directory entry")?;
        entries.push((entry, name, meta));
    }
    entries.sort_unstable_by(|a, b| {
        // git has very specific rules for how to compare names
        // https://github.com/git/git/blob/e09f1254c54329773904fe25d7c545a1fb4fa920/tree.c#L99
        let afn = &a.1;
        let afn = afn.as_bytes();
        let bfn = &b.1;
        let bfn = bfn.as_bytes();
        let common_len = std::cmp::min(afn.len(), bfn.len());
        match afn[..common_len].cmp(&bfn[..common_len]) {
            Ordering::Equal => {}
            o => return o,
        }
        if afn.len() == bfn.len() {
            return Ordering::Equal;
        }
        let c1 = if let Some(c) = afn.get(common_len).copied() {
            Some(c)
        } else if a.2.is_dir() {
            Some(b'/')
        } else {
            None
        };
        let c2 = if let Some(c) = bfn.get(common_len).copied() {
            Some(c)
        } else if b.2.is_dir() {
            Some(b'/')
        } else {
            None
        };

        c1.cmp(&c2)
    });

    let mut tree_object = Vec::new();
    for (entry, file_name, meta) in entries {
        if file_name == ".git" || file_name == "target" {
            continue;
        }
        let mode = if meta.is_dir() {
            "40000"
        } else if meta.is_symlink() {
            "120000"
        } else if (meta.permissions().mode() & 0o111) != 0 {
            // has at least one executable bit set
            "100755"
        } else {
            "100644"
        };
        let path = entry.path();
        let hash = if meta.is_dir() {
            let Some(hash) = write_tree_for(&path)? else {
                // empty directory, so don't include in parent
                continue;
            };
            hash
        } else {
            let hash = Object::blob_from_file(&path)
                .context("open blob input file")?
                .write_to_objects()?;
            hash
        };
        tree_object.extend(mode.as_bytes());
        tree_object.push(b' ');
        tree_object.extend(file_name.as_bytes());
        tree_object.push(0);
        tree_object.extend(hash);
    }

    if tree_object.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            Object {
                kind: Kind::Tree,
                expected_size: tree_object.len() as u64,
                reader: Cursor::new(tree_object),
            }
            .write_to_objects()
            .context("write tree object")?,
        ))
    }
}

pub(crate) fn invoke() -> anyhow::Result<()> {
    let Some(hash) = write_tree_for(Path::new(".")).context("construct root tree object")? else {
        anyhow::bail!("asked to make object for empty tree");
    };
    println!("{}", hex::encode(hash));
    Ok(())
}
