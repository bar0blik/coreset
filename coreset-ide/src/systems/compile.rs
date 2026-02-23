use std::panic::{AssertUnwindSafe, catch_unwind};

use bevy::prelude::*;

use crate::{events::CompileEvent, session::CoresetSession};

/// Recompile the active controller's source whenever a [`CompileEvent`] is received.
pub fn compile_system(
    mut session: NonSendMut<CoresetSession>,
    mut events: EventReader<CompileEvent>,
) {
    for _ in events.read() {
        let Some(idx) = session.active_controller else {
            events.clear();
            return;
        };

        let source = session.controllers[idx].source.clone();

        let result = catch_unwind(AssertUnwindSafe(|| coreset_compiler::compile(&source)));

        match result {
            Ok(bytes) => {
                session.controllers[idx].bytecode = bytes.clone();
                session.controllers[idx].compile_error = None;
                session.controllers[idx].controller.set_program(bytes);
                session.controllers[idx].controller.reset();
            }
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Unknown compile error".to_string()
                };
                session.controllers[idx].compile_error = Some(msg);
            }
        }
    }
}
