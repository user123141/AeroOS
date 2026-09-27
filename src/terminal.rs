//! Terminal через ConPTY.

extern crate std;

use anyhow::Result;

pub struct Terminal {
    conpty: Option<conpty::PseudoConsole>,
    cols: i16,
    rows: i16,
}
impl Terminal {
    pub fn new() -> Self {
        Self {
            conpty: None,
            cols: 80,
            rows: 24,
        }
    }

    pub fn init(&mut self) -> Result<()> {
        let pc = conpty::PseudoConsole::new(self.cols, self.rows)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        self.conpty = Some(pc);
        tracing::info!("ConPTY terminal initialized ({}x{})", self.cols, self.rows);
        Ok(())
    }
    pub fn resize(&mut self, cols: i16, rows: i16) -> Result<()> {
        self.cols = cols;
        self.rows = rows;
        if let Some(pc) = &self.conpty {
            let _ = pc.resize(cols, rows);
        }
        Ok(())
    }
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        if let Some(pc) = &self.conpty {
            pc.write_input(data).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        Ok(())
    }
    pub fn read(&mut self) -> Result<Vec<u8>> {
        if let Some(pc) = &self.conpty {
            return pc.read_output(8192).map_err(|e| anyhow::anyhow!("{}", e));
        }
        Ok(Vec::new())
    }
}
