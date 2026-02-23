use std::{fs, path::PathBuf};

use bevy::prelude::*;
use rfd::FileDialog;

use crate::{
    events::{
        CompileEvent, NewFileEvent, OpenBinaryEvent, OpenSourceEvent, SaveBinaryEvent,
        SaveSourceAsEvent, SaveSourceEvent,
    },
    session::CoresetSession,
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

fn load_source(
    session: &mut CoresetSession,
    path: PathBuf,
    compile_ev: &mut EventWriter<CompileEvent>,
) {
    match fs::read_to_string(&path) {
        Ok(text) => {
            session.source = text;
            session.current_path = Some(path);
            session.compile_error = None;
            compile_ev.send(CompileEvent);
        }
        Err(e) => {
            session.compile_error = Some(format!("Could not open file: {e}"));
        }
    }
}

fn save_source_to(session: &mut CoresetSession, path: PathBuf) {
    match fs::write(&path, &session.source) {
        Ok(_) => session.current_path = Some(path),
        Err(e) => session.compile_error = Some(format!("Could not save file: {e}")),
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
    // New -------------------------------------------------------------------
    for _ in new_ev.read() {
        session.source.clear();
        session.bytecode.clear();
        session.compile_error = None;
        session.current_path = None;
    }

    // Open source (.cst) ----------------------------------------------------
    for _ in open_src_ev.read() {
        if let Some(path) = cst_dialog().pick_file() {
            load_source(&mut session, path, &mut compile_ev);
        }
    }

    // Save source -----------------------------------------------------------
    for _ in save_src_ev.read() {
        if let Some(path) = session.current_path.clone() {
            save_source_to(&mut session, path);
        } else {
            // No path yet — behave like Save As.
            if let Some(path) = cst_dialog().set_file_name("program.cst").save_file() {
                save_source_to(&mut session, path);
            }
        }
    }

    // Save source as --------------------------------------------------------
    for _ in save_src_as_ev.read() {
        let default_name = session
            .current_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("program.cst")
            .to_string();
        if let Some(path) = cst_dialog().set_file_name(&default_name).save_file() {
            save_source_to(&mut session, path);
        }
    }

    // Save binary (.bin) ----------------------------------------------------
    for _ in save_bin_ev.read() {
        if session.bytecode.is_empty() {
            session.compile_error = Some("Nothing compiled yet.".to_string());
            continue;
        }
        let default_name = session
            .current_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .map(|s| format!("{s}.bin"))
            .unwrap_or_else(|| "program.bin".to_string());

        if let Some(path) = bin_dialog().set_file_name(&default_name).save_file() {
            if let Err(e) = fs::write(&path, &session.bytecode) {
                session.compile_error = Some(format!("Could not save binary: {e}"));
            }
        }
    }

    // Open binary (.bin) → decompile ----------------------------------------
    for _ in open_bin_ev.read() {
        if let Some(path) = bin_dialog().pick_file() {
            match fs::read(&path) {
                Ok(bytes) => {
                    session.source = coreset_compiler::decompile(&bytes);
                    session.bytecode = bytes;
                    session.compile_error = None;
                    // A decompiled binary is not tied to a .cst path.
                    session.current_path = None;
                    // Push the loaded program to every controller.
                    let program = session.bytecode.clone();
                    for cs in &mut session.controllers {
                        cs.controller.set_program(program.clone());
                        cs.controller.reset();
                    }
                }
                Err(e) => {
                    session.compile_error = Some(format!("Could not open binary: {e}"));
                }
            }
        }
    }
}
