use anyhow::{Context, Result};
use file_owner::PathExt;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let path = env::current_dir()?;
    let mut stdout = io::stdout().lock();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let cur_path = entry.path();
        let last_path_char = relative_path(&cur_path)
            .and_then(|rel_path| rel_path.chars().nth(0))
            .and_then(|c| if c == '.' { Some(true) } else { Some(false) })
            .unwrap_or(false);

        if last_path_char {
            println!("PASSING {}", cur_path.display());
            continue;
        }

        let mut lsrs = Lsrs::new(&cur_path);

        lsrs.traverse_dir(&cur_path).context("got some error")?;
        let mut dir_size = lsrs.calculate_size();
        dir_size += "\n";
        stdout.write_all(dir_size.as_bytes())?;
    }

    Ok(())
}

struct Lsrs<'a> {
    path: &'a PathBuf,
    size: u64,
}

impl<'a> Lsrs<'a> {
    fn new(p: &PathBuf) -> Lsrs {
        let lsrs = Lsrs { path: p, size: 0 };
        lsrs
    }
    fn permisison_graph(&mut self) -> String {
        let mode = self.path.metadata().unwrap().permissions().mode();
        let user_perms = format!(
            "{}{}{}",
            if (mode & 0o400) > 0 { 'r' } else { '-' },
            if (mode & 0o200) > 0 { 'w' } else { '-' },
            if (mode & 0o100) > 0 { 'x' } else { '-' }
        );

        let group_perms = format!(
            "{}{}{}",
            if (mode & 0o40) > 0 { 'r' } else { '-' },
            if (mode & 0o20) > 0 { 'w' } else { '-' },
            if (mode & 0o10) > 0 { 'x' } else { '-' }
        );

        let other_perms = format!(
            "{}{}{}",
            if (mode & 0o4) > 0 { 'r' } else { '-' },
            if (mode & 0o2) > 0 { 'w' } else { '-' },
            if (mode & 0o1) > 0 { 'x' } else { '-' }
        );

        return format!("{}{}{}", user_perms, group_perms, other_perms);
    }

    fn owners(&mut self) -> Result<(String, String)> {
        let owner = self
            .path
            .owner()
            .unwrap()
            .name()
            .unwrap()
            .unwrap_or("".to_string());

        let group = self
            .path
            .group()
            .unwrap()
            .name()
            .unwrap()
            .unwrap_or("".to_string());

        Ok((owner, group))
    }
    fn traverse_dir(&mut self, path: &PathBuf) -> anyhow::Result<()> {
        if path.is_file() {
            let meta = path.metadata().context("couldn't get metadata")?;
            self.size += meta.size();

            return Ok(());
        }

        'looper: for entry in fs::read_dir(path).context("couldn't read the dir")? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.traverse_dir(&path)?;
                continue 'looper;
            }
            let meta = path.metadata().context("couldn't get metadata")?;
            self.size += meta.size();
        }

        Ok(())
    }

    fn calculate_size(&self) -> String {
        //NOTE three option available for now: first KB, second MB, third is GB
        let KB = 1024.0;
        let MB = KB * 1024.0;
        let GB = MB * 1024.0;

        if self.size < KB as u64 {
            format!("{}B", self.size as f64 / KB)
        } else if self.size < MB as u64 {
            format!("{:.2}KB", self.size as f64 / KB)
        } else if self.size < GB as u64 {
            format!("{:.2}MB", self.size as f64 / MB)
        } else {
            format!("{:.2}GB", self.size as f64 / GB)
        }
    }
}

fn relative_path(path: &PathBuf) -> Option<&str> {
    //TODO: add windows support too;
    path.to_str().unwrap_or("/").split("/").last()
}
