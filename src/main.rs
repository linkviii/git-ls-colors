/// Git LS Color - List tracked files in color
/// Directories:
/// *) All files in the directory are tracked
/// +) Some files in the directory are tracked, but not all
use lscolors::{LsColors, Style};
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

fn is_tracked(p: &Path, app: &App) -> bool {
    let tmp = if p.is_file() {
        p.canonicalize().unwrap()
    } else {
        p.parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .join(p.file_name().unwrap())
    };
    let rpath = pathdiff::diff_paths(tmp, app.git_root.as_path()).unwrap();

    app.index.get_path(rpath.as_path(), 0).is_some()
}

fn strip_dot(p: &Path) -> PathBuf {
    // Must be a smarter way to do this
    // Also this returns nothing for './'. Bit of a problem
    p.iter().filter(|&x| x != ".").collect()
}

#[derive(Debug, Clone, Copy)]
enum Trackedness {
    All,
    Some,
    None,
}

fn dir_track_indecator(track: Trackedness) -> &'static str {
    match track {
        Trackedness::All => "*",
        Trackedness::Some => "+",
        Trackedness::None => "^",
    }
}

// Recursivly check all children of p
fn has_tracked(p: &Path, app: &App) -> Trackedness {
    if p.is_file() || p.is_symlink() {
        match is_tracked(p, app) {
            true => return Trackedness::All,
            false => return Trackedness::None,
        }
    }

    // Early exit on the git folder
    let absolute = p.canonicalize().unwrap();
    if absolute == app.git_root.join(".git") {
        return Trackedness::None;
    }
    // Should sub repos have a special case?
    // Should any folder called `.git` be ignored?

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

        match has_tracked(epath, app) {
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
            Trackedness::Some
        } else {
            Trackedness::All
        }
    } else {
        Trackedness::None
    }
}

struct App {
    git_root: PathBuf,
    index: git2::Index,
    lscolors: LsColors,
}

fn printdir(dot: &Path, app: &App) {
    let ddir = dot.read_dir().unwrap();
    for x in ddir {
        let xx = x.unwrap().path();
        let p = xx.as_path();

        let style = app
            .lscolors
            .style_for_path(p)
            .map(Style::to_ansi_term_style)
            .unwrap_or_default();

        // let color_p = format!("{}", style.paint(p.to_str().unwrap()));
        let color_p = format!("{}", style.paint(strip_dot(p).to_str().unwrap()));
        if p.is_dir() && !p.is_symlink() {
            let track = has_tracked(p, app);
            let indicator = dir_track_indecator(track);
            match track {
                Trackedness::None => {}
                _ => println!("{} {}", color_p, indicator),
            };
        } else if is_tracked(p, app) {
            println!("{}", color_p);
        } else {
            // Not tracked
            // println!("{}", color_p);
        }
    }
}

fn main() {
    let git_root = find_git_root();
    // println!("Git root: {:?}", git_root);

    let repo = match git2::Repository::open(git_root.as_path()) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let index = repo.index().unwrap();

    let app: App = App {
        git_root,
        index,
        lscolors: LsColors::from_env().unwrap_or_default(),
    };

    // println!("--------------");

    let args: Vec<String> = std::env::args().collect();
    // println!("Args: {:?}", args);

    if args.len() == 1 {
        let dot = Path::new(".");
        printdir(dot, &app);
    } else {
        // Different from printdir in that this will print a level of directories as their name and their contents
        // todo combine?

        let mut line_space = true;
        for x in &args[1..] {
            let p = Path::new(&x);
            let style = app
                .lscolors
                .style_for_path(p)
                .map(Style::to_ansi_term_style)
                .unwrap_or_default();

            let color_p = format!("{}", style.paint(strip_dot(p).to_str().unwrap()));

            if p.is_dir() && !p.is_symlink() {
                let track = has_tracked(p, &app);

                let indicator = dir_track_indecator(track);
                match track {
                    Trackedness::None => {}
                    _ => {
                        if !line_space {
                            println!();
                        }
                        println!("{}: {}", color_p, indicator);
                        printdir(p, &app);
                        println!();
                        line_space = true;
                    }
                };
            } else if is_tracked(p, &app) {
                println!("{}", color_p);
                line_space = false;
            } else {
                // TODO option of show untracked file?
            }
        }
    }
}
