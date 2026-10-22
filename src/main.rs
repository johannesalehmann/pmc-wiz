fn main() {
    let Ok((files, consts)) = parse_args() else {
        return;
    };
    let inputs =
        tiny_pmc::parsing::Inputs::from_cli_args(&files).expect("failed to parse input files");
    let consts = tiny_pmc::parsing::parse_const_assignments(&consts[0])
        .expect("failed to parse constant assignments");
}

fn parse_args() -> Result<(Vec<String>, Vec<String>), ()> {
    let mut next_is_const = false;
    let mut files = Vec::new();
    let mut consts = Vec::new();
    for arg in std::env::args().skip(1) {
        if arg == "-h" || arg == "--help" {
            print_help();
            return Err(());
        }
        if arg == "-c" {
            next_is_const = true;
        }
        if arg.starts_with("-") {
            println!("unexpected argument `{arg}`");
            return Err(());
        }
        if next_is_const {
            if consts.len() > 0 {
                println!(
                    "constants specified twice (combine all constants into a single argument instead)"
                );
                return Err(());
            }
            consts.push(arg);
        } else {
            files.push(arg);
        }
    }
    if files.is_empty() {
        print_help();
        return Err(());
    }
    Ok((files, consts))
}

fn print_help() {
    let help = r#"                      .
            /^\     .
       /\   "V"                 ***********
      /__\   I      O  o        * PMC-WIZ *
     //..\\  I     .            ***********
     \].`[/  I
     /l\/j\  (]    .  O         | Call with:
    /. ~~ ,\/I          .       |     pmc-wiz model_file.prism props_file.props -c "K=5,n=12"
    \\L__j^\/I       o          |
     \/--v}  I     o   .        | Model and property files can be in any order. UMB files also supported.
     |    |  I   _________
     |    |  I c(`       ')o
     |    l  I   \.     ,/
   _/j  L l\_!  _//^---^\\_     This ASCII art is by Rowan Crawford
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

            "#;
    println!("{help}")
}
