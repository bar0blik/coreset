use std::{fs, path::PathBuf};

use bevy::prelude::*;
use rfd::FileDialog;

use crate::{
    events::{
        CompileEvent, NewFileEvent, OpenBinaryEvent, OpenSourceEvent, SaveBinaryEvent,
        SaveSourceAsEvent, SaveSourceEvent,
    },
    session::{ControllerState, CoresetSession},
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn cst_dialog() -> FileDialog {
    FileDialog::new().add_filter("Coreset source", &["cst"])
}

fn bin_dialog() -> FileDialog {
    FileDialog::new().add_filter("Coreset binary", &["bin"])
}

/// Create a new controller tab, set its source + path, select it, and fire a compile.
fn open_source_as_new_tab(
    session: &mut CoresetSession,
    source: String,
    path: Option<PathBuf>,
    compile_ev: &mut EventWriter<CompileEvent>,
) {
    let name = path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unsaved")
        .to_string();

    let mut cs = ControllerState::new(name);
    cs.source = source;
    cs.current_path = path;
    session.controllers.push(cs);
    session.active_controller = Some(session.controllers.len() - 1);
    compile_ev.send(CompileEvent);
}

fn active_save_path(session: &CoresetSession) -> Option<PathBuf> {
    session
        .active_controller
        .and_then(|i| session.controllers.get(i))
        .and_then(|cs| cs.current_path.clone())
}

fn save_active_to(session: &mut CoresetSession, path: PathBuf) {
    let Some(idx) = session.active_controller else {
        return;
    };
    let Some(cs) = session.controllers.get_mut(idx) else {
        return;
    };
    match fs::write(&path, &cs.source) {
        Ok(_) => {
            // Update tab name to match the file.
            if let Some(stem) = path.file_name().and_then(|n| n.to_str()) {
                cs.name = stem.to_string();
            }
            cs.current_path = Some(path);
        }
        Err(e) => cs.compile_error = Some(format!("Could not save file: {e}")),
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

pub fn file_system(
    mut session: NonSendMut<CoresetSession>,
    mut compile_ev: EventWriter<CompileEvent>,
    mut new_ev: EventReader<NewFileEvent>,
    mut open_src_ev: EventReader<OpenSourceEvent>,
    mut save_src_ev: EventReader<SaveSourceEvent>,
    mut save_src_as_ev: EventReader<SaveSourceAsEvent>,
    mut save_bin_ev: EventReader<SaveBinaryEvent>,
    mut open_bin_ev: EventReader<OpenBinaryEvent>,
) {
    // New file — create an empty tab ----------------------------------------
    for _ in new_ev.read() {
        open_source_as_new_tab(&mut session, String::new(), None, &mut compile_ev);
    }

    // Open source (.cst) — always a new tab ----------------------------------
    for _ in open_src_ev.read() {
        if let Some(path) = cst_dialog().pick_file() {
            match fs::read_to_string(&path) {
                Ok(text) => {
                    open_source_as_new_tab(&mut session, text, Some(path), &mut compile_ev);
                }
                Err(e) => {
                    if let Some(idx) = session.active_controller {
                        if let Some(cs) = session.controllers.get_mut(idx) {
                            cs.compile_error = Some(format!("Could not open file: {e}"));
                        }
                    }
                }
            }
        }
    }

    // Save source (active tab) ----------------------------------------------
    for _ in save_src_ev.read() {
        if let Some(path) = active_save_path(&session) {
            save_active_to(&mut session, path);
        } else {
            if let Some(path) = cst_dialog().set_file_name("program.cst").save_file() {
                save_active_to(&mut session, path);
            }
        }
    }

    // Save source as (active tab) -------------------------------------------
    for _ in save_src_as_ev.read() {
        let default_name = active_save_path(&session)
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "program.cst".to_string());
        if let Some(path) = cst_dialog().set_file_name(&default_name).save_file() {
            save_active_to(&mut session, path);
        }
    }

    // Save binary (active tab) ----------------------------------------------
    for _ in save_bin_ev.read() {
        let (bytecode, default_name) = match session
            .active_controller
            .and_then(|i| session.controllers.get(i))
        {
            Some(cs) if !cs.bytecode.is_empty() => {
                let name = cs
                    .current_path
                    .as_ref()
                    .and_then(|p| p.file_stem())
                    .and_then(|s| s.to_str())
                    .map(|s| format!("{s}.bin"))
                    .unwrap_or_else(|| "program.bin".to_string());
                (cs.bytecode.clone(), name)
            }
            _ => {
                if let Some(idx) = session.active_controller {
                    if let Some(cs) = session.controllers.get_mut(idx) {
                        cs.compile_error = Some("Nothing compiled yet.".to_string());
                    }
                }
                continue;
            }
        };
        if let Some(path) = bin_dialog().set_file_name(&default_name).save_file() {
            if let Err(e) = fs::write(&path, &bytecode) {
                if let Some(idx) = session.active_controller {
                    if let Some(cs) = session.controllers.get_mut(idx) {
                        cs.compile_error = Some(format!("Could not save binary: {e}"));
                    }
                }
            }
        }
    }

    // Open binary (.bin) → decompile into new tab ---------------------------
    for _ in open_bin_ev.read() {
        if let Some(path) = bin_dialog().pick_file() {
            match fs::read(&path) {
                Ok(bytes) => {
                    let source = coreset_compiler::decompile(&bytes);
                    // Create a new tab; binary is not tied to a .cst path.
                    let stem = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("binary")
                        .to_string();
                    let mut cs = ControllerState::new(stem);
                    cs.source = source;
                    cs.bytecode = bytes.clone();
                    cs.controller.set_program(bytes);
                    session.controllers.push(cs);
                    session.active_controller = Some(session.controllers.len() - 1);
                }
                Err(e) => {
                    if let Some(idx) = session.active_controller {
                        if let Some(cs) = session.controllers.get_mut(idx) {
                            cs.compile_error = Some(format!("Could not open binary: {e}"));
                        }
                    }
                }
            }
        }
    }
}
