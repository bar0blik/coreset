use std::panic::{catch_unwind, AssertUnwindSafe};

use bevy::prelude::*;

use crate::{events::CompileEvent, session::CoresetSession};

/// Recompile the session source whenever a [`CompileEvent`] is received.
///
/// Any compile error (including panics from unknown mnemonics) is stored in
/// `session.compile_error` rather than crashing the app.
pub fn compile_system(
    mut session: NonSendMut<CoresetSession>,
    mut events: EventReader<CompileEvent>,
) {
    for _ in events.read() {
        let source = session.source.clone();

        let result = catch_unwind(AssertUnwindSafe(|| coreset_compiler::compile(&source)));

        match result {
            Ok(bytes) => {
                session.bytecode = bytes.clone();
                session.compile_error = None;
                // Push the new program to every controller and reset them.
                for cs in &mut session.controllers {
                    cs.controller.set_program(bytes.clone());
                    cs.controller.reset();
                }
            }
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Unknown compile error".to_string()
                };
                session.compile_error = Some(msg);
            }
        }
    }
}
