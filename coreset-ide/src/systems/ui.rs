use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::{
    events::{
        AddControllerEvent, AddMemoryEvent, BindMemoryEvent, CompileEvent, NewFileEvent,
        OpenBinaryEvent, OpenSourceEvent, PauseEvent, RemoveControllerEvent, RemoveMemoryEvent,
        ResetEvent, RunEvent, SaveBinaryEvent, SaveSourceAsEvent, SaveSourceEvent, StepEvent,
        UnbindMemoryEvent,
    },
    session::{CoresetSession, ExecutionMode},
    systems::params::{FileEventWriters, SessionEventWriters},
};

pub fn ui_system(
    mut contexts: EguiContexts,
    mut session: NonSendMut<CoresetSession>,
    mut compile_ev: EventWriter<CompileEvent>,
    mut step_ev: EventWriter<StepEvent>,
    mut run_ev: EventWriter<RunEvent>,
    mut pause_ev: EventWriter<PauseEvent>,
    mut reset_ev: EventWriter<ResetEvent>,
    mut session_ev: SessionEventWriters,
    mut file_ev: FileEventWriters,
) {
    let ctx = contexts.ctx_mut();

    // -----------------------------------------------------------------------
    // Top: toolbar
    // -----------------------------------------------------------------------
    egui::TopBottomPanel::top("coreset_toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // File menu
            ui.menu_button("📁 File", |ui| {
                if ui.button("🆕 New").clicked() {
                    file_ev.new_file.send(NewFileEvent);
                    ui.close_menu();
                }
                if ui.button("📂 Open .cst").clicked() {
                    file_ev.open_src.send(OpenSourceEvent);
                    ui.close_menu();
                }
                ui.separator();
                let save_label = if let Some(p) = &session.current_path {
                    format!(
                        "💾 Save  ({})",
                        p.file_name().and_then(|n| n.to_str()).unwrap_or("?")
                    )
                } else {
                    "💾 Save".to_string()
                };
                if ui.button(save_label).clicked() {
                    file_ev.save_src.send(SaveSourceEvent);
                    ui.close_menu();
                }
                if ui.button("💾 Save As .cst…").clicked() {
                    file_ev.save_src_as.send(SaveSourceAsEvent);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("📦 Save As .bin…").clicked() {
                    file_ev.save_bin.send(SaveBinaryEvent);
                    ui.close_menu();
                }
                if ui.button("🔍 Open .bin (decompile)").clicked() {
                    file_ev.open_bin.send(OpenBinaryEvent);
                    ui.close_menu();
                }
            });
            ui.separator();
            if ui.button("⚙ Compile").clicked() {
                compile_ev.send(CompileEvent);
            }
            ui.separator();
            if ui.button("▶ Step").clicked() {
                step_ev.send(StepEvent);
            }
            match session.mode {
                ExecutionMode::Stopped => {
                    if ui.button("▶▶ Run").clicked() {
                        run_ev.send(RunEvent);
                    }
                }
                ExecutionMode::Running => {
                    if ui.button("⏸ Pause").clicked() {
                        pause_ev.send(PauseEvent);
                    }
                }
            }
            if ui.button("↩ Reset").clicked() {
                reset_ev.send(ResetEvent);
            }
            ui.separator();
            ui.label("Speed:");
            ui.add(
                egui::Slider::new(&mut session.run_speed, 0.1..=1_000_000.0)
                    .logarithmic(true)
                    .text("ins/s"),
            );
        });

        if let Some(err) = &session.compile_error.clone() {
            ui.colored_label(egui::Color32::RED, format!("⚠  {err}"));
        }
    });

    // -----------------------------------------------------------------------
    // Left: source editor
    // -----------------------------------------------------------------------
    egui::SidePanel::left("coreset_source")
        .min_width(300.0)
        .show(ctx, |ui| {
            // File path bar
            let path_label = match &session.current_path {
                Some(p) => p.to_string_lossy().into_owned(),
                None => "Unsaved".to_string(),
            };
            ui.horizontal(|ui| {
                ui.weak(path_label);
            });
            ui.separator();
            let avail = ui.available_size();
            let response = ui.add(
                egui::TextEdit::multiline(&mut session.source)
                    .code_editor()
                    .desired_width(avail.x)
                    .desired_rows(40),
            );
            // Dynamic compilation: recompile on every keystroke.
            if response.changed() {
                compile_ev.send(CompileEvent);
            }
        });

    // -----------------------------------------------------------------------
    // Right: controller + memory management
    // -----------------------------------------------------------------------
    egui::SidePanel::right("coreset_sidebar")
        .min_width(260.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // --- Memory banks -------------------------------------------
                ui.heading("Memory Banks");
                ui.horizontal(|ui| {
                    let len = session.memories.len();
                    if ui.small_button("+ 64").clicked() {
                        session_ev.add_mem.send(AddMemoryEvent {
                            name: format!("mem{len}"),
                            size: 64,
                        });
                    }
                    if ui.small_button("+ 256").clicked() {
                        session_ev.add_mem.send(AddMemoryEvent {
                            name: format!("mem{len}"),
                            size: 256,
                        });
                    }
                    if ui.small_button("+ 1024").clicked() {
                        session_ev.add_mem.send(AddMemoryEvent {
                            name: format!("mem{len}"),
                            size: 1024,
                        });
                    }
                });

                let mut mem_to_remove: Option<usize> = None;
                for (i, bank) in session.memories.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "[{i}] {} — {} cells",
                            bank.name,
                            bank.memory.borrow().len()
                        ));
                        if ui.small_button("✕").clicked() {
                            mem_to_remove = Some(i);
                        }
                    });
                }
                if let Some(i) = mem_to_remove {
                    session_ev.remove_mem.send(RemoveMemoryEvent { index: i });
                }

                ui.separator();

                // --- Controllers --------------------------------------------
                ui.heading("Controllers");
                if ui.small_button("+ Add Controller").clicked() {
                    session_ev.add_ctrl.send(AddControllerEvent {
                        name: format!("ctrl{}", session.controllers.len()),
                    });
                }

                let n_memories = session.memories.len();
                let running = session.mode == ExecutionMode::Running;
                let mut ctrl_to_remove: Option<usize> = None;
                let mut to_bind: Option<BindMemoryEvent> = None;
                let mut to_unbind: Option<UnbindMemoryEvent> = None;

                for (ci, cs) in session.controllers.iter().enumerate() {
                    let status = if cs.controller.halted {
                        "halted"
                    } else if running {
                        "running"
                    } else {
                        "stopped"
                    };

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(&cs.name);
                            ui.weak(format!("[{status}]"));
                            if ui.small_button("✕").clicked() {
                                ctrl_to_remove = Some(ci);
                            }
                        });

                        egui::Grid::new(format!("ctrl_info_{ci}"))
                            .num_columns(2)
                            .show(ui, |ui| {
                                ui.label("IP");
                                ui.monospace(cs.controller.ip.to_string());
                                ui.end_row();
                                ui.label("REG");
                                ui.monospace(cs.controller.register.to_string());
                                ui.end_row();
                            });

                        ui.label("Bound memories:");
                        for &mi in &cs.bound_memories {
                            if let Some(bank) = session.memories.get(mi) {
                                ui.horizontal(|ui| {
                                    ui.label(format!("[{mi}] {}", bank.name));
                                    if ui.small_button("unbind").clicked() {
                                        to_unbind = Some(UnbindMemoryEvent {
                                            controller: ci,
                                            memory: mi,
                                        });
                                    }
                                });
                            }
                        }
                        for mi in 0..n_memories {
                            if !cs.bound_memories.contains(&mi) {
                                if let Some(bank) = session.memories.get(mi) {
                                    if ui
                                        .small_button(format!("bind [{mi}] {}", bank.name))
                                        .clicked()
                                    {
                                        to_bind = Some(BindMemoryEvent {
                                            controller: ci,
                                            memory: mi,
                                        });
                                    }
                                }
                            }
                        }
                    });
                }

                if let Some(i) = ctrl_to_remove {
                    session_ev
                        .remove_ctrl
                        .send(RemoveControllerEvent { index: i });
                }
                if let Some(ev) = to_bind {
                    session_ev.bind.send(ev);
                }
                if let Some(ev) = to_unbind {
                    session_ev.unbind.send(ev);
                }
            });
        });

    // -----------------------------------------------------------------------
    // Bottom: memory contents
    // -----------------------------------------------------------------------
    egui::TopBottomPanel::bottom("coreset_memory")
        .resizable(true)
        .min_height(140.0)
        .show(ctx, |ui| {
            ui.heading("Memory Contents");
            egui::ScrollArea::both().show(ui, |ui| {
                for (i, bank) in session.memories.iter().enumerate() {
                    let mem = bank.memory.borrow();
                    let data = mem.data();
                    let header = format!("[{i}] {}  ({} cells)", bank.name, data.len());
                    egui::CollapsingHeader::new(header)
                        .default_open(true)
                        .show(ui, |ui| {
                            egui::Grid::new(format!("mem_grid_{i}"))
                                .striped(true)
                                .min_col_width(54.0)
                                .show(ui, |ui| {
                                    // Header row
                                    ui.strong("addr");
                                    for col in 0..8_usize {
                                        ui.strong(format!("+{col}"));
                                    }
                                    ui.end_row();
                                    // Data rows
                                    let rows = (data.len() + 7) / 8;
                                    for row in 0..rows {
                                        ui.monospace(format!("{}", row * 8));
                                        for col in 0..8 {
                                            let idx = row * 8 + col;
                                            if idx < data.len() {
                                                ui.monospace(data[idx].to_string());
                                            }
                                        }
                                        ui.end_row();
                                    }
                                });
                        });
                }
            });
        });

    // -----------------------------------------------------------------------
    // Central: decompiled bytecode view
    // -----------------------------------------------------------------------
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Decompiled Bytecode");
        egui::ScrollArea::vertical().show(ui, |ui| {
            if session.bytecode.is_empty() {
                ui.weak("No program compiled yet.");
            } else {
                let decompiled = coreset_compiler::decompile(&session.bytecode);
                // Immutable read-only viewer
                ui.add(
                    egui::TextEdit::multiline(&mut decompiled.as_str())
                        .code_editor()
                        .desired_width(f32::INFINITY),
                );
            }
        });
    });
}
