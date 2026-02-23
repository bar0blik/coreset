use std::rc::Rc;

use bevy::prelude::*;

use crate::{
    events::{
        AddControllerEvent, AddMemoryEvent, BindMemoryEvent, RemoveControllerEvent,
        RemoveMemoryEvent, UnbindMemoryEvent,
    },
    session::{ControllerState, CoresetSession, MemoryBank},
};

/// Handle add / remove events for memory banks.
pub fn manage_memory_system(
    mut session: NonSendMut<CoresetSession>,
    mut add_ev: EventReader<AddMemoryEvent>,
    mut remove_ev: EventReader<RemoveMemoryEvent>,
    mut bind_ev: EventReader<BindMemoryEvent>,
    mut unbind_ev: EventReader<UnbindMemoryEvent>,
) {
    for ev in add_ev.read() {
        session.memories.push(MemoryBank::new(&ev.name, ev.size));
    }

    for ev in remove_ev.read() {
        if ev.index < session.memories.len() {
            session.memories.remove(ev.index);
            // Remove any bindings that pointed to this index and shift higher ones.
            for cs in &mut session.controllers {
                cs.bound_memories.retain(|&m| m != ev.index);
                for m in &mut cs.bound_memories {
                    if *m > ev.index {
                        *m -= 1;
                    }
                }
            }
        }
    }

    for ev in bind_ev.read() {
        // Pre-compute len to end the immutable borrow before get_mut.
        let mem_len = session.memories.len();
        if let Some(cs) = session.controllers.get_mut(ev.controller) {
            if ev.memory < mem_len && !cs.bound_memories.contains(&ev.memory) {
                cs.bound_memories.push(ev.memory);
            }
        }
    }

    for ev in unbind_ev.read() {
        if let Some(cs) = session.controllers.get_mut(ev.controller) {
            cs.bound_memories.retain(|&m| m != ev.memory);
        }
    }

    // Re-sync every controller's Rc pointers after any change.
    // Collect clones first to end the immutable borrow before iterating mutably.
    let mem_ptrs: Vec<coreset_vm::SharedMemory> = session
        .memories
        .iter()
        .map(|b| Rc::clone(&b.memory))
        .collect();

    for cs in session.controllers.iter_mut() {
        cs.controller.memories.clear();
        for &idx in &cs.bound_memories {
            if let Some(m) = mem_ptrs.get(idx) {
                cs.controller.memories.push(Rc::clone(m));
            }
        }
    }
}

/// Handle add / remove events for controllers.
pub fn manage_controller_system(
    mut session: NonSendMut<CoresetSession>,
    mut add_ev: EventReader<AddControllerEvent>,
    mut remove_ev: EventReader<RemoveControllerEvent>,
) {
    for ev in add_ev.read() {
        let mut cs = ControllerState::new(&ev.name);
        // Pre-load the current bytecode if already compiled.
        if !session.bytecode.is_empty() {
            cs.controller.set_program(session.bytecode.clone());
        }
        session.controllers.push(cs);
    }

    for ev in remove_ev.read() {
        if ev.index < session.controllers.len() {
            session.controllers.remove(ev.index);
        }
    }
}
