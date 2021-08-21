use std::path::Path;

fn find_git_root() -> Box<Path> {
    let  dot = Path::new(".").canonicalize().unwrap();

    let mut here = dot.as_path();

    while !here.join(".git").exists() {
        here = here.parent().unwrap();
    }

    None
}


fn main() {
    println!("Hello, world!");

    // ---------
    use lscolors::{LsColors, Style};
    let lscolors = LsColors::from_env().unwrap_or_default();
    let path = "./.git";
    let style = lscolors.style_for_path(path);
    // If you want to use `ansi_term`:
    let ansi_style = style.map(Style::to_ansi_term_style).unwrap_or_default();
    println!("{}", ansi_style.paint(path));
    // ---------
    println!("--------------");
    use git2::Repository;

    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };


    let index = repo.index().unwrap();

    let iitem = index.get_path(Path::new("Cargo.toml"), 0);
    match iitem {
        Some(_) => println!("In index"),
        None => println!("not index")
    }



    let wts = repo.worktrees().unwrap();
    for wt in wts.iter(){

        println!("{:?}", wt);
    }

    // repo.worktree(name: &str, path: &Path, opts: Option<&WorktreeAddOptions<'a>>)
    // let wt = git2::Worktree::open_from_repository(&repo).unwrap();

    // println!("{}", wt.name().unwrap());
    // println!("{}", wt);
    // ---------
    println!("--------------");

    println!("world world world");
}
