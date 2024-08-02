

/**
 * Main cli entry function.
 */
pub async fn run_cli() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let cmd = clap::Command::new("pkarr-cli")
        .version(VERSION)
        .subcommand(
            clap::Command::new("publish")
                .about("Publish pkarr dns records.")
                .arg(
                    clap::Arg::new("tabfile_path")
                        .required(false)
                        .help("File path to the dns records file.")
                        .default_value("./records.conf"),
                )
                .arg(
                    clap::Arg::new("once")
                        .long("once")
                        .required(false)
                        .num_args(0)
                        .help("File path to the dns records csv file."),
                ),
        )
        .subcommand(
            clap::Command::new("resolve")
                .about("Resolve pkarr dns records.")
                .arg(clap::Arg::new("pubkey").required(false).help("Pkarr public key uri.")),
        );
    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("resolve", matches)) => {
            println!("resolve called");
            // cli_resolve(matches, folder_buf, verbose);
        }
        Some(("publish", matches)) => {
            println!("publish called");
            // cli_publish(matches, folder_buf, verbose);
        }
        _ => {
            unimplemented!("command not implemented")
        }
    };
}
