use clap::{command, Arg, Command};
use px::config::Config;

fn build_cli(config: &Config) -> Command {
    let mut cmd = command!()
        .subcommand_required(true)
        .arg_required_else_help(true);

    for script in config.scripts.values() {
        cmd = cmd.subcommand(
            Command::new(&script.name)
                .about(script.description.clone().unwrap_or_default())
                .arg(Arg::new("raw").hide(true).raw(true)),
        );
    }

    cmd
}

fn main() {
    let config = Config::new();
    let matches = build_cli(&config).get_matches();

    match matches.subcommand() {
        Some((cmd_name, cmd_args)) => {
            let script = config.scripts.get(cmd_name).unwrap();

            let raw_args = if let Some(raw) = cmd_args.get_many::<String>("raw") {
                raw.map(|s| s.as_str()).collect()
            } else {
                vec![]
            };

            script.execute(&raw_args);
        }
        _ => unreachable!("Exhausted list of subcommands"),
    }
}
