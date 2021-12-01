mod clargs;
mod logger;

use pilaris::lexer::Token;

fn main() {
    logger::PilarisLogger::init(log::Level::Debug);
    log::info!("Log enabled");

    let arguments = clargs::Arguments::from_args();

    let mut lexer = pilaris::lexer::Lexer::new(arguments.source).unwrap();

    // while let Ok(tok) = lexer.get_token() {
    loop {
        match lexer.get_token() {
            Ok(tok) => {
                println!(
                    "{:?} \"{}\", starts at col: {}",
                    tok,
                    lexer.token_str(),
                    lexer.token_start_column()
                );
                if tok == Token::EOF {
                    break;
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
}
