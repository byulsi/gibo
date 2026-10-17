mod cli;
mod graph;
mod hooks;
mod knowledge;
mod packs;
mod workspace;

fn main() {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some("hook") {
        run_hook(&arguments[1..]);
        return;
    }
    if let Err(error) = cli::run(arguments) {
        eprintln!("gibo: {error}");
        std::process::exit(1);
    }
}

fn run_hook(arguments: &[String]) {
    use std::io::Read;

    let Some(event) = arguments.first() else {
        eprintln!("gibo: usage: gibo hook <pre-tool-use|post-tool-use>");
        std::process::exit(1);
    };
    if arguments.len() != 1 {
        eprintln!("gibo: usage: gibo hook <pre-tool-use|post-tool-use>");
        std::process::exit(1);
    }

    let mut input = String::new();
    if let Err(error) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("gibo: could not read hook input: {error}");
        std::process::exit(1);
    }
    match hooks::evaluate(event, &input) {
        Ok(hooks::Outcome::Allow) => {}
        Ok(hooks::Outcome::Block(reason)) => {
            eprintln!("{reason}");
            std::process::exit(2);
        }
        Ok(hooks::Outcome::Feedback(output)) => println!("{output}"),
        Err(error) => {
            eprintln!("gibo: {error}");
            std::process::exit(1);
        }
    }
}
