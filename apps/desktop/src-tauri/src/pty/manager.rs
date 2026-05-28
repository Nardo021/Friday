use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

struct PtyEntry {
    master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
    session_id: String,
}

pub struct PtyManager {
    ptys: Arc<Mutex<HashMap<String, PtyEntry>>>,
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            ptys: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_pty(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
        cwd: &Path,
        executable: &str,
        args: &[String],
    ) -> AppResult<(String, mpsc::UnboundedReceiver<Vec<u8>>)> {
        let pty_system = NativePtySystem::default();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::Process(format!("openpty failed: {e}")))?;

        let mut cmd = CommandBuilder::new(executable);
        cmd.cwd(cwd);
        for arg in args {
            cmd.arg(arg);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| AppError::Process(format!("pty spawn failed: {e}")))?;

        let master = pair.master;
        let pty_id = Uuid::new_v4().to_string();
        let (tx, rx) = mpsc::unbounded_channel();

        let reader_master = master.try_clone_reader().map_err(|e| {
            AppError::Process(format!("pty reader clone failed: {e}"))
        })?;

        std::thread::spawn(move || {
            let mut reader = reader_master;
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let entry = PtyEntry {
            master: Arc::new(Mutex::new(master)),
            child: Arc::new(Mutex::new(child)),
            session_id: session_id.to_string(),
        };

        self.ptys
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?
            .insert(pty_id.clone(), entry);

        Ok((pty_id, rx))
    }

    pub fn write(&self, pty_id: &str, data: &[u8]) -> AppResult<()> {
        let ptys = self
            .ptys
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        let entry = ptys
            .get(pty_id)
            .ok_or_else(|| AppError::Other(format!("pty not found: {pty_id}")))?;
        let mut writer = entry
            .master
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?
            .take_writer()
            .map_err(|e| AppError::Process(format!("pty writer failed: {e}")))?;
        writer
            .write_all(data)
            .map_err(|e| AppError::Io(e))?;
        Ok(())
    }

    pub fn resize(&self, pty_id: &str, cols: u16, rows: u16) -> AppResult<()> {
        let ptys = self
            .ptys
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        let entry = ptys
            .get(pty_id)
            .ok_or_else(|| AppError::Other(format!("pty not found: {pty_id}")))?;
        entry
            .master
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::Process(format!("pty resize failed: {e}")))?;
        Ok(())
    }

    pub fn close(&self, pty_id: &str, force: bool) -> AppResult<()> {
        let mut ptys = self
            .ptys
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        if let Some(entry) = ptys.remove(pty_id) {
            if let Ok(mut child) = entry.child.lock() {
                if force {
                    let _ = child.kill();
                } else {
                    let _ = child.wait();
                }
            }
        }
        Ok(())
    }

    pub fn pty_id_for_session(&self, session_id: &str) -> Option<String> {
        self.ptys
            .lock()
            .ok()
            .and_then(|ptys| {
                ptys
                    .iter()
                    .find(|(_, e)| e.session_id == session_id)
                    .map(|(id, _)| id.clone())
            })
    }

    pub fn close_by_session(&self, session_id: &str, force: bool) -> AppResult<()> {
        let ids: Vec<String> = self
            .ptys
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?
            .iter()
            .filter(|(_, e)| e.session_id == session_id)
            .map(|(id, _)| id.clone())
            .collect();
        for id in ids {
            self.close(&id, force)?;
        }
        Ok(())
    }
}
