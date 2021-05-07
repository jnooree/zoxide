use super::Run;
use crate::app::Remove;
use crate::config;
use crate::db::{DatabaseFile, Query};
use crate::error::WriteErrorHandler;
use crate::fzf::Fzf;
use crate::util;

use anyhow::{bail, Result};

use std::io::Write;

impl Run for Remove {
    fn run(&self) -> Result<()> {
        let data_dir = config::zo_data_dir()?;
        let mut db = DatabaseFile::new(data_dir);
        let mut db = db.open()?;

        let selection;
        match &self.interactive {
            Some(keywords) => {
                let query = Query::new(keywords);
                let now = util::current_time()?;
                let resolve_symlinks = config::zo_resolve_symlinks();

                let mut fzf = Fzf::new(true)?;
                for dir in db.iter_matches(&query, now, resolve_symlinks) {
                    writeln!(fzf.stdin(), "{}", dir.display_score(now)).pipe_exit("fzf")?;
                }

                selection = fzf.wait_select()?;
                let paths = selection.lines().filter_map(|line| line.get(5..));
                let mut not_found = Vec::new();
                for path in paths {
                    if !db.remove(&path) {
                        not_found.push(path);
                    }
                }

                if !not_found.is_empty() {
                    let mut err = "path not found in database:".to_string();
                    for path in not_found {
                        err.push_str("\n  ");
                        err.push_str(path.as_ref());
                    }
                    bail!(err);
                }
            }
            None => {
                // unwrap is safe here because path is required_unless_present = "interactive"
                let path = self.path.as_ref().unwrap();
                if !db.remove(path) {
                    let path_abs = util::resolve_path(&path)?;
                    let path_abs = util::path_to_str(&path_abs)?;
                    if path_abs != path && !db.remove(path) {
                        bail!("path not found in database:\n  {}", &path)
                    }
                }
            }
        }

        Ok(())
    }
}