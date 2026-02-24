use bevy::prelude::*;

use crate::{
    events::{PauseEvent, ResetEvent, RunEvent, StepEvent},
    session::{CoresetSession, ExecutionMode},
};

/// Handle execution control events and drive continuous execution via the
/// Bevy [`Time`] resource.
pub fn execution_system(
    time: Res<Time>,
    mut session: NonSendMut<CoresetSession>,
    mut step_ev: EventReader<StepEvent>,
    mut run_ev: EventReader<RunEvent>,
    mut pause_ev: EventReader<PauseEvent>,
    mut reset_ev: EventReader<ResetEvent>,
) {
    // --- Mode transitions ---------------------------------------------------

    if run_ev.read().count() > 0 {
        session.mode = ExecutionMode::Running;
        session.run_accumulator = 0.0;
    }

    if pause_ev.read().count() > 0 {
        session.mode = ExecutionMode::Stopped;
    }

    if reset_ev.read().count() > 0 {
        session.mode = ExecutionMode::Stopped;
        for cs in &mut session.controllers {
            cs.controller.reset();
        }
    }

    // --- Manual step --------------------------------------------------------

    for _ in step_ev.read() {
        for cs in &mut session.controllers {
            cs.controller.step();
        }
    }

    // --- Continuous run -----------------------------------------------------

    if session.mode == ExecutionMode::Running {
        session.run_accumulator += time.delta_secs_f64();
        let interval = 1.0 / session.run_speed.max(0.001);
        // Clamp accumulated debt to at most 1 second to prevent a lag spike
        // from triggering thousands of iterations in one frame.
        session.run_accumulator = session.run_accumulator.min(1.0);

        while session.run_accumulator >= interval {
            session.run_accumulator -= interval;
            let all_halted = session.controllers.iter().all(|cs| cs.controller.halted);

            if all_halted {
                session.mode = ExecutionMode::Stopped;
                break;
            }

            for cs in &mut session.controllers {
                cs.controller.step();
            }
        }
    }
}
