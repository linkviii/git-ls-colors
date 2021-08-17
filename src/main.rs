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
    use git2::Repository;

    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    // ---------

    println!("world world world");
}
