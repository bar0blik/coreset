use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub mod events;
pub mod session;
pub mod systems;

pub use events::*;
pub use session::{ControllerState, CoresetSession, ExecutionMode, MemoryBank};

/// The main Coreset plugin.
///
/// # Embedding
///
/// Add `CoresetPlugin` to any `App` that already has (or will have) an egui
/// context.  If no [`EguiPlugin`] has been added yet the plugin adds it
/// automatically.
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use coreset_bevy::CoresetPlugin;
///
/// App::new()
///     .add_plugins((DefaultPlugins, CoresetPlugin))
///     .run();
/// ```
pub struct CoresetPlugin;

impl Plugin for CoresetPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }

        app
            // --- NonSend session (contains Rc) ---
            .insert_non_send_resource(CoresetSession::default())
            // --- Events ---
            .add_event::<events::CompileEvent>()
            .add_event::<events::StepEvent>()
            .add_event::<events::RunEvent>()
            .add_event::<events::PauseEvent>()
            .add_event::<events::ResetEvent>()
            .add_event::<events::AddMemoryEvent>()
            .add_event::<events::RemoveMemoryEvent>()
            .add_event::<events::AddControllerEvent>()
            .add_event::<events::RemoveControllerEvent>()
            .add_event::<events::BindMemoryEvent>()
            .add_event::<events::UnbindMemoryEvent>()
            // File I/O events
            .add_event::<events::NewFileEvent>()
            .add_event::<events::OpenSourceEvent>()
            .add_event::<events::SaveSourceEvent>()
            .add_event::<events::SaveSourceAsEvent>()
            .add_event::<events::SaveBinaryEvent>()
            .add_event::<events::OpenBinaryEvent>()
            // --- Systems (file → session → compile → execution → ui) ---
            .add_systems(Update, systems::file::file_system)
            .add_systems(
                Update,
                (
                    systems::session::manage_memory_system,
                    systems::session::manage_controller_system,
                )
                    .after(systems::file::file_system),
            )
            .add_systems(
                Update,
                systems::compile::compile_system.after(systems::session::manage_controller_system),
            )
            .add_systems(
                Update,
                systems::execution::execution_system.after(systems::compile::compile_system),
            )
            .add_systems(
                Update,
                systems::ui::ui_system.after(systems::execution::execution_system),
            );
    }
}
