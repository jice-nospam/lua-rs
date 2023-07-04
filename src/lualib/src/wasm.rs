//! this modules provides stdout/stderr for the wasm target. see LuaState::default()

#[cfg(target_arch = "wasm32")]

use std::io::Write;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub(crate) struct ConsoleWriter {
    buffer: String,
    is_buffered: bool,
}

impl ConsoleWriter {
    fn new(is_buffered: bool) -> Self {
        Self {
            buffer: String::new(),
            is_buffered,
        }
    }    
}

impl Write for ConsoleWriter{
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> { 
        self.buffer.push_str(&String::from_utf8_lossy(buf));

        if !self.is_buffered {
            log(&self.buffer);
            self.buffer.clear();

            return Ok(buf.len());
        }

        if let Some(i) = self.buffer.rfind('\n') {
            let buffered = {
                let (first, last) = self.buffer.split_at(i);
                log(first);

                String::from(&last[1..])
            };

            self.buffer.clear();
            self.buffer.push_str(&buffered);
        }

        Ok(buf.len())        
    }
    fn flush(&mut self) -> Result<(), std::io::Error> { 
        log(&self.buffer);
        self.buffer.clear();

        Ok(())        
     }
}

/// provides a Writer that outputs to the javascript console for wasm target
pub fn js_console() -> Box<dyn Write> {
    Box::new(ConsoleWriter::new(true))
}