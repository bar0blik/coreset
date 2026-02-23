use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Compilation
// ---------------------------------------------------------------------------

/// Trigger a (re)compile of the source currently in the session.
#[derive(Event)]
pub struct CompileEvent;

// ---------------------------------------------------------------------------
// Execution control
// ---------------------------------------------------------------------------

/// Advance every non-halted controller by one instruction.
#[derive(Event)]
pub struct StepEvent;

/// Start continuous execution.
#[derive(Event)]
pub struct RunEvent;

/// Pause continuous execution (keeps IP / register intact).
#[derive(Event)]
pub struct PauseEvent;

/// Reset every controller (IP → 0, register → 0) without unloading the program.
#[derive(Event)]
pub struct ResetEvent;

// ---------------------------------------------------------------------------
// Memory management
// ---------------------------------------------------------------------------

/// Add a new named memory bank of `size` u64 cells.
#[derive(Event)]
pub struct AddMemoryEvent {
    pub name: String,
    pub size: usize,
}

/// Remove the memory bank at `index`.
#[derive(Event)]
pub struct RemoveMemoryEvent {
    pub index: usize,
}

// ---------------------------------------------------------------------------
// Controller management
// ---------------------------------------------------------------------------

/// Add a new named controller.
#[derive(Event)]
pub struct AddControllerEvent {
    pub name: String,
}

/// Remove the controller at `index`.
#[derive(Event)]
pub struct RemoveControllerEvent {
    pub index: usize,
}

// ---------------------------------------------------------------------------
// Memory binding
// ---------------------------------------------------------------------------

/// Attach memory bank `memory` to controller `controller` as its next slot.
#[derive(Event)]
pub struct BindMemoryEvent {
    pub controller: usize,
    pub memory: usize,
}

/// Detach memory bank `memory` from controller `controller`.
#[derive(Event)]
pub struct UnbindMemoryEvent {
    pub controller: usize,
    pub memory: usize,
}

// ---------------------------------------------------------------------------
// File I/O
// ---------------------------------------------------------------------------

/// Create a new empty source (clears editor + bytecode).
#[derive(Event)]
pub struct NewFileEvent;

/// Open a native dialog to pick a .cst file and load it into the editor.
#[derive(Event)]
pub struct OpenSourceEvent;

/// Save the current source to its path (or prompt for one if unsaved).
#[derive(Event)]
pub struct SaveSourceEvent;

/// Save the current source to a new path chosen via dialog.
#[derive(Event)]
pub struct SaveSourceAsEvent;

/// Save compiled bytecode as a .bin file chosen via dialog.
#[derive(Event)]
pub struct SaveBinaryEvent;

/// Open a .bin file via dialog, decompile it, and load into the editor.
#[derive(Event)]
pub struct OpenBinaryEvent;
