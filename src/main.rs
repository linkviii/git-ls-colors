use std::path::Path;
use std::path::PathBuf;

fn find_git_root() -> PathBuf {
    let dot = Path::new(".").canonicalize().unwrap();

    let mut here = dot.as_path();

    while !here.join(".git").exists() {
        match here.parent() {
            Some(p) => here = p,
            None => {
                println!("Not in a git repo");
                std::process::exit(1);
            }
        }
    }

    here.to_path_buf()
}

fn is_tracked(p: &Path, index: &git2::Index) -> bool {
    let groot = index.path().unwrap().parent().unwrap().parent().unwrap();

    let rpath = pathdiff::diff_paths(p.canonicalize().unwrap(), groot).unwrap();

    index.get_path(rpath.as_path(), 0).is_some()
}

fn strip_dot(p: &Path) -> PathBuf {
    p.into_iter().filter(|&x| x != ".").collect()
}

fn main() {
    // println!("Hello, world!");

    // ---------

    // ---------
    // println!("--------------");
    use git2::Repository;

    let git_root = find_git_root();
    // println!("Git root: {:?}", git_root);
    //
    let repo = match Repository::open(git_root) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let index = repo.index().unwrap();

    let wts = repo.worktrees().unwrap();
    for wt in wts.iter() {
        println!("{:?}", wt);
    }

    // println!("--------------");
    foobar(&index);
    // println!("world world world");
}

#[derive(Debug)]
enum Trackedness {
    All,
    Some,
    None,
}

// Recursivly check all children of p
fn has_tracked(p: &Path, index: &git2::Index) -> Trackedness {
    if p.is_file() {
        match is_tracked(p, &index) {
            true => return Trackedness::All,
            false => return Trackedness::None,
        }
    }

    let mut any_tracked = false;
    let mut any_untracked = false;

    let mut dir = p.read_dir().unwrap().peekable();

    if dir.peek().is_none() {
        // Empty dir cannot be tracked
        return Trackedness::None;
    }

    for entry in dir {
        let epathbuf = entry.unwrap().path();
        let epath = epathbuf.as_path();

        match has_tracked(epath, &index) {
            Trackedness::All => any_tracked = true,
            Trackedness::Some => {
                any_tracked = true;
                any_untracked = true;
            }
            Trackedness::None => any_untracked = true,
        };

        if any_tracked && any_untracked {
            break;
        }
    }

    if any_tracked {
        if any_untracked {
            return Trackedness::Some;
        } else {
            return Trackedness::All;
        }
    } else {
        return Trackedness::None;
    }
}

fn foobar(index: &git2::Index) {
    use lscolors::{LsColors, Style};
    let lscolors = LsColors::from_env().unwrap_or_default();

    let dot = Path::new(".");
    let ddir = dot.read_dir().unwrap();
    for x in ddir {
        let xx = x.unwrap().path();
        let p = xx.as_path();

        let style = lscolors
            .style_for_path(p)
            .map(Style::to_ansi_term_style)
            .unwrap_or_default();

        // let color_p = format!("{}", style.paint(p.to_str().unwrap()));
        let color_p = format!("{}", style.paint(strip_dot(p).to_str().unwrap()));
        if p.is_dir() {
            println!("d {}", color_p);
            let track = has_tracked(p, &index);
            println!("{:?}", track);
        } else if is_tracked(p, &index) {
            println!("t {}", color_p);
        } else {
            println!("u {}", color_p);
        }
    }
}
