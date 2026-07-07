//! Transport layer for LSP frames.

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process;
use std::env;
use std::fs;

use mawu::XffValue;
use nemesis::NemesisError;
use crate::error::NomosResult;

/// LSP reader for reading frames from a `BufRead` source.
pub struct LspReader<R: BufRead> {
    reader: R,
    temp_file_path: PathBuf,
}

impl<R: BufRead> LspReader<R> {
    /// Creates a new `LspReader`.
    pub fn new(reader: R) -> Self {
        let temp_file_path = env::temp_dir().join(format!("nomos_lsp_read_{}.json", process::id()));
        Self {
            reader,
            temp_file_path,
        }
    }

    /// Reads a single LSP frame, returning the parsed `XffValue` or `None` if EOF is reached.
    pub fn read_frame(&mut self) -> NomosResult<Option<XffValue>> {
        let mut content_length = None;

        // Parse headers
        loop {
            let mut line = String::new();
            let num_bytes = self.reader.read_line(&mut line).map_err(|e| {
                NemesisError::new("LspReader::read_frame", e)
            })?;
            if num_bytes == 0 {
                return Ok(None); // EOF
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                break; // End of header block
            }

            if trimmed.to_lowercase().starts_with("content-length:") {
                let parts: Vec<&str> = trimmed.split(':').collect();
                if parts.len() == 2 {
                    let len = parts[1].trim().parse::<usize>().map_err(|e| {
                        NemesisError::new("LspReader::read_frame", std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                    })?;
                    content_length = Some(len);
                }
            }
        }

        let len = match content_length {
            Some(l) => l,
            None => {
                return Err(NemesisError::new(
                    "LspReader::read_frame",
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing Content-Length header"),
                ));
            }
        };

        // Read exact payload bytes
        let mut buf = vec![0u8; len];
        self.reader.read_exact(&mut buf).map_err(|e| {
            NemesisError::new("LspReader::read_frame", e)
        })?;

        // Write payload to temp file to parse it using mawu
        fs::write(&self.temp_file_path, &buf).map_err(|e| {
            NemesisError::new("LspReader::read_frame", e)
        })?;

        let val = mawu::read::json(&self.temp_file_path)?;

        // Cleanup temp file
        let _ = fs::remove_file(&self.temp_file_path);

        Ok(Some(val))
    }
}

impl<R: BufRead> Drop for LspReader<R> {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.temp_file_path);
    }
}

/// LSP writer for writing frames to a `Write` destination.
pub struct LspWriter<W: Write> {
    writer: W,
}

impl<W: Write> LspWriter<W> {
    /// Creates a new `LspWriter`.
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Writes a single LSP frame containing the serialized `XffValue`.
    pub fn write_frame(&mut self, val: &XffValue) -> NomosResult<()> {
        let temp_file_path = env::temp_dir().join(format!("nomos_lsp_write_{}.json", process::id()));
        mawu::write(&temp_file_path, val.clone())?;
        let payload = fs::read_to_string(&temp_file_path).map_err(|e| {
            NemesisError::new("LspWriter::write_frame", e)
        })?;
        let _ = fs::remove_file(&temp_file_path);

        write!(
            self.writer,
            "Content-Length: {}\r\n\r\n{}",
            payload.len(),
            payload
        )
        .map_err(|e| NemesisError::new("LspWriter::write_frame", e))?;
        self.writer.flush().map_err(|e| NemesisError::new("LspWriter::write_frame", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use athena::Object;

    #[test]
    fn test_reader_writer_framing() {
        let mut obj = Object::new();
        obj.insert("test".to_string(), XffValue::from("value"));
        let val = XffValue::from(obj);

        let mut write_buf = Vec::new();
        {
            let mut writer = LspWriter::new(&mut write_buf);
            writer.write_frame(&val).unwrap();
        }

        let mut reader = LspReader::new(Cursor::new(write_buf));
        let read_val = reader.read_frame().unwrap().unwrap();

        assert_eq!(
            read_val
                .as_object()
                .unwrap()
                .get("test")
                .unwrap()
                .as_string()
                .unwrap(),
            "value"
        );
    }
}
