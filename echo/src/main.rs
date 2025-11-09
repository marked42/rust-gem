use clap::{Arg, ArgAction, Command};

fn main() {
    let matches = Command::new("echo")
        .version("0.1.0")
        .author("kos")
        .about("Rust echo")
        .arg(
            Arg::new("omit_newline")
                .short('n')
                .long("omit-newline")
                .action(ArgAction::SetTrue)
                .help("Do not print newline"),
        )
        .arg(
            Arg::new("text")
                .value_name("TEXT")
                .num_args(1..)
                .help("Input text")
                .required(true),
        )
        .get_matches();

    let text = matches
        .get_many::<String>("text")
        .unwrap()
        .map(|s| s.as_str())
        .collect::<Vec<_>>();

    let omit_newline = matches.get_flag("omit_newline");
    let ending = if omit_newline { "" } else { "\n" };

    print!("{}{}", text.join(" "), ending);
}
