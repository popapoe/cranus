mod anal;
mod check;
mod graph;
mod interpret;
mod location;
mod parse;
mod scan;
mod token;
mod tree;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    WrongArgumentCount,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::WrongArgumentCount => {
                write!(f, "wrong argument count")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

fn run(
    reader: impl std::io::Read,
) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
    let characters = utf8_decode::UnsafeDecoder::new(reader.bytes()).map(|character| {
        character
            .map_err(|error| std::boxed::Box::new(error) as std::boxed::Box<dyn std::error::Error>)
    });
    let scanner = crate::scan::scan(characters)?;
    let tree = crate::parse::parse(scanner)?;
    let graph = crate::anal::anal(tree)?;
    crate::check::check(&graph)?;
    let mut line = std::string::String::new();
    std::io::stdin().read_line(&mut line)?;
    let input = line.trim().parse().map_err(std::boxed::Box::new)?;
    let value = crate::interpret::interpret(graph, input)?;
    println!("{:?}", value);
    Ok(())
}

fn main() -> std::process::ExitCode {
    fn inner(
        args: &[std::string::String],
    ) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        if args.len() == 2 {
            let file = std::fs::File::open(&args[1]).map_err(std::boxed::Box::new)?;
            let reader = std::io::BufReader::new(file);
            run(reader)?;
            Ok(())
        } else {
            Err(std::boxed::Box::new(Error::WrongArgumentCount))
        }
    }
    let args: std::vec::Vec<std::string::String> = std::env::args().collect();
    match inner(&args) {
        Ok(_) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}: {}", args[0], error);
            std::process::ExitCode::FAILURE
        }
    }
}
