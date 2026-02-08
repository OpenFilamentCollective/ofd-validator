use std::io::IsTerminal;

/// Print a colored progress step to stderr.
pub fn log_step(label: &str, count: Option<usize>) {
    let is_tty = std::io::stderr().is_terminal();
    if is_tty {
        match count {
            Some(n) => eprintln!("  \x1b[36m->\x1b[0m {} \x1b[2m({} items)\x1b[0m", label, n),
            None => eprintln!("  \x1b[36m->\x1b[0m {}", label),
        }
    } else {
        match count {
            Some(n) => eprintln!("  -> {} ({} items)", label, n),
            None => eprintln!("  -> {}", label),
        }
    }
}
