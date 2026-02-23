use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::events::{
    AddControllerEvent, AddMemoryEvent, BindMemoryEvent, NewFileEvent, OpenBinaryEvent,
    OpenSourceEvent, RemoveControllerEvent, RemoveMemoryEvent, SaveBinaryEvent, SaveSourceAsEvent,
    SaveSourceEvent, UnbindMemoryEvent,
};

/// Bundles all session mutation event writers (memory & controller management).
#[derive(SystemParam)]
pub struct SessionEventWriters<'w> {
    pub add_mem: EventWriter<'w, AddMemoryEvent>,
    pub remove_mem: EventWriter<'w, RemoveMemoryEvent>,
    pub add_ctrl: EventWriter<'w, AddControllerEvent>,
    pub remove_ctrl: EventWriter<'w, RemoveControllerEvent>,
    pub bind: EventWriter<'w, BindMemoryEvent>,
    pub unbind: EventWriter<'w, UnbindMemoryEvent>,
}

/// Bundles all file I/O event writers.
#[derive(SystemParam)]
pub struct FileEventWriters<'w> {
    pub new_file: EventWriter<'w, NewFileEvent>,
    pub open_src: EventWriter<'w, OpenSourceEvent>,
    pub save_src: EventWriter<'w, SaveSourceEvent>,
    pub save_src_as: EventWriter<'w, SaveSourceAsEvent>,
    pub save_bin: EventWriter<'w, SaveBinaryEvent>,
    pub open_bin: EventWriter<'w, OpenBinaryEvent>,
}
