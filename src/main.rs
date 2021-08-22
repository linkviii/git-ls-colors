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
    let rpath = pathdiff::diff_paths(p.canonicalize().unwrap(), app.git_root.as_path()).unwrap();

    app.index.get_path(rpath.as_path(), 0).is_some()
}

fn strip_dot(p: &Path) -> PathBuf {
    p.into_iter().filter(|&x| x != ".").collect()
}

#[derive(Debug, Clone, Copy)]
enum Trackedness {
    All,
    Some,
    None,
}

// I think returning static ref makes sense?
fn dir_track_indecator(track: Trackedness) -> &'static str {
    match track {
        Trackedness::All => "*",
        Trackedness::Some => "+",
        Trackedness::None => "^",
    }
}

// Recursivly check all children of p
fn has_tracked(p: &Path, app: &App) -> Trackedness {
    if p.is_file() {
        match is_tracked(p, &app) {
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

        match has_tracked(epath, &app) {
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
        if p.is_dir() {
            let track = has_tracked(p, &app);
            let indicator = dir_track_indecator(track);
            match track {
                Trackedness::None => {}
                _ => println!("{} {}", color_p, indicator),
            };
        } else if is_tracked(p, &app) {
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
        printdir(&dot, &app);
    } else {
        let mut line_space = true;
        for x in &args[1..] {
            let p = Path::new(&x);
            let style = app
                .lscolors
                .style_for_path(p)
                .map(Style::to_ansi_term_style)
                .unwrap_or_default();

            let color_p = format!("{}", style.paint(strip_dot(p).to_str().unwrap()));

            if p.is_dir() {
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
            } else {
                if is_tracked(p, &app) {
                    println!("{}", color_p);
                    line_space = false;
                }
            }
        }
    }
}
