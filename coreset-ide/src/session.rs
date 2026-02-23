use std::{cell::RefCell, path::PathBuf, rc::Rc};

use coreset_vm::{Controller, Memory, SharedMemory};

// ---------------------------------------------------------------------------
// Memory bank
// ---------------------------------------------------------------------------

pub struct MemoryBank {
    pub name: String,
    pub memory: SharedMemory,
}

impl MemoryBank {
    pub fn new(name: impl Into<String>, size: usize) -> Self {
        Self {
            name: name.into(),
            memory: Rc::new(RefCell::new(Memory::new(size))),
        }
    }
}

// ---------------------------------------------------------------------------
// Controller state
// ---------------------------------------------------------------------------

pub struct ControllerState {
    pub name: String,
    pub controller: Controller,
    /// Indices into `CoresetSession::memories` that are bound to this controller
    pub bound_memories: Vec<usize>,
}

impl ControllerState {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            controller: Controller::new(),
            bound_memories: Vec::new(),
        }
    }

    /// Rebuild the controller's memory list from the session's memory banks.
    pub fn sync_memories(&mut self, banks: &[MemoryBank]) {
        self.controller.memories.clear();
        for &idx in &self.bound_memories {
            if let Some(bank) = banks.get(idx) {
                self.controller.memories.push(Rc::clone(&bank.memory));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Execution mode
// ---------------------------------------------------------------------------

#[derive(Default, Clone, Copy, PartialEq)]
pub enum ExecutionMode {
    #[default]
    Stopped,
    Running,
}

// ---------------------------------------------------------------------------
// Session  (NonSend resource — contains Rc)
// ---------------------------------------------------------------------------

pub struct CoresetSession {
    /// Assembly source currently in the editor.
    pub source: String,
    /// Last successfully compiled bytecode.
    pub bytecode: Vec<u8>,
    /// Most recent compile error, if any.
    pub compile_error: Option<String>,
    /// All memory banks known to this session.
    pub memories: Vec<MemoryBank>,
    /// All controller states.
    pub controllers: Vec<ControllerState>,
    /// Whether controllers are running continuously.
    pub mode: ExecutionMode,
    /// Path of the currently open .cst file (None if unsaved).
    pub current_path: Option<PathBuf>,
    /// Target instructions per second when running.
    pub run_speed: f64,
    /// Accumulated time (seconds) since last instruction tick.
    pub run_accumulator: f64,
}

impl Default for CoresetSession {
    fn default() -> Self {
        Self {
            source: String::new(),
            bytecode: Vec::new(),
            compile_error: None,
            memories: Vec::new(),
            controllers: Vec::new(),
            mode: ExecutionMode::Stopped,
            current_path: None,
            run_speed: 10.0,
            run_accumulator: 0.0,
        }
    }
}
