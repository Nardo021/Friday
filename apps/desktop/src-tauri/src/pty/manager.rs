use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

struct PtyEntry {
    master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
    session_id: String,
    child_pid: Option<u32>,
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
    ) -> AppResult<(String, Option<u32>, mpsc::UnboundedReceiver<Vec<u8>>)> {
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

        let child_pid = child.process_id();
        let master = pair.master;
        let writer = master
            .take_writer()
            .map_err(|e| AppError::Process(format!("pty writer failed: {e}")))?;

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
            writer: Arc::new(Mutex::new(writer)),
            child: Arc::new(Mutex::new(child)),
            session_id: session_id.to_string(),
            child_pid,
        };

        self.ptys
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?
            .insert(pty_id.clone(), entry);

        Ok((pty_id, child_pid, rx))
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
            .writer
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        writer.write_all(data).map_err(AppError::Io)?;
        writer.flush().map_err(AppError::Io)?;
        Ok(())
    }

    pub fn interrupt(&self, pty_id: &str) -> AppResult<()> {
        // Ctrl+C
        self.write(pty_id, &[0x03])
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

    pub fn child_pid(&self, pty_id: &str) -> Option<u32> {
        self.ptys
            .lock()
            .ok()
            .and_then(|ptys| ptys.get(pty_id).and_then(|e| e.child_pid))
    }

    pub fn child_pid_for_session(&self, session_id: &str) -> Option<u32> {
        self.ptys.lock().ok().and_then(|ptys| {
            ptys.values()
                .find(|e| e.session_id == session_id)
                .and_then(|e| e.child_pid)
        })
    }

    /// Soft stop: Ctrl+C, wait up to `grace`, then kill if still alive.
    pub fn close(&self, pty_id: &str, force: bool) -> AppResult<()> {
        let entry = {
            let mut ptys = self
                .ptys
                .lock()
                .map_err(|e| AppError::Other(e.to_string()))?;
            ptys.remove(pty_id)
        };

        let Some(entry) = entry else {
            return Ok(());
        };

        if force {
            if let Ok(mut child) = entry.child.lock() {
                let _ = child.kill();
                let _ = child.wait();
            }
            return Ok(());
        }

        // Interrupt first (SIGINT via Ctrl+C).
        if let Ok(mut writer) = entry.writer.lock() {
            let _ = writer.write_all(&[0x03]);
            let _ = writer.flush();
        }

        let grace = Duration::from_secs(3);
        let deadline = Instant::now() + grace;
        loop {
            let exited = entry
                .child
                .lock()
                .ok()
                .and_then(|mut child| child.try_wait().ok().flatten());
            if exited.is_some() {
                break;
            }
            if Instant::now() >= deadline {
                if let Ok(mut child) = entry.child.lock() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        Ok(())
    }

    pub fn pty_id_for_session(&self, session_id: &str) -> Option<String> {
        self.ptys.lock().ok().and_then(|ptys| {
            ptys.iter()
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

    /// Poll until the PTY child exits (or the entry is removed). Returns exit success.
    pub fn wait_for_exit(&self, pty_id: &str) -> bool {
        loop {
            let status = {
                let ptys = match self.ptys.lock() {
                    Ok(p) => p,
                    Err(_) => return false,
                };
                let Some(entry) = ptys.get(pty_id) else {
                    return false;
                };
                let result = match entry.child.lock() {
                    Ok(mut child) => child.try_wait().ok().flatten(),
                    Err(_) => return false,
                };
                result
            };

            if let Some(status) = status {
                let success = status.success();
                let _ = self.close(pty_id, true);
                return success;
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn owns_pid(&self, pid: u32) -> bool {
        self.ptys
            .lock()
            .ok()
            .map(|ptys| ptys.values().any(|e| e.child_pid == Some(pid)))
            .unwrap_or(false)
    }
}
