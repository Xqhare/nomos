use std::io::{BufReader, stdin, stdout};

use nomos::lsp::LspServer;
use nomos::lsp::transport::{LspReader, LspWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin_raw = stdin();
    let stdout_raw = stdout();

    let reader = LspReader::new(BufReader::new(stdin_raw.lock()));
    let writer = LspWriter::new(stdout_raw.lock());

    let mut server = LspServer::new();
    server.run(reader, writer)?;

    Ok(())
}
