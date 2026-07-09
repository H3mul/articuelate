mod cue;
mod engine;
mod panels;

use cue::{Cue, FollowMode, CueStatus};
use engine::AudioEngine;
use egui_dock::{DockArea, DockState, TabViewer};

// ---------------------------------------------------------------------------
// Tabs definition

#[derive(Debug, Clone, PartialEq)]
enum Tab {
    Cuelist,
    Detail,
    Media,
}

struct MyTabViewer<'a> {
    engine: &'a mut AudioEngine,
    cues: &'a mut Vec<Cue>,
    selected_cue_index: &'a mut usize,
    selected_task_index: &'a mut Option<usize>,
}

impl<'a> TabViewer for MyTabViewer<'a> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::Cuelist => "Main Cuelist".into(),
            Tab::Detail => "Detail Inspector".into(),
            Tab::Media => "Active Media".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Cuelist => {
                panels::cuelist::render_header(ui);
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        panels::cuelist::render(ui, self.cues, self.selected_cue_index);
                    });
            }
            Tab::Detail => {
                let selected_cue = self.cues.get(*self.selected_cue_index);
                let selected_task = selected_cue
                    .and_then(|cue| self.selected_task_index.map(|i| &cue.tasks[i]));
                panels::detail::render(ui, selected_cue, selected_task);
            }
            Tab::Media => {
                let playbacks = self.engine.active_playbacks();
                panels::media::render(ui, &playbacks);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Application state

struct ArticuelateApp {
    engine: AudioEngine,
    cues: Vec<Cue>,
    selected_cue_index: usize,
    selected_task_index: Option<usize>,
    search_query: String,
    tree: DockState<Tab>,
}

impl Default for ArticuelateApp {
    fn default() -> Self {
        let cues = vec![
            Cue {
                number: 1.0,
                name: "Storm Intro".into(),
                status: CueStatus::Ready,
                follow_mode: FollowMode::Manual,
                tasks: vec![
                    cue::Task {
                        target_name: "BGM".into(),
                        property: "Volume".into(),
                        target_value: -24.0,
                        duration_secs: 3.0,
                        curve: cue::FadeCurve::Linear,
                        output: cue::OutputTarget {
                            name: "Main L/R".into(),
                        },
                    },
                    cue::Task {
                        target_name: "Player".into(),
                        property: "Play".into(),
                        target_value: 0.0,
                        duration_secs: 0.0,
                        curve: cue::FadeCurve::Linear,
                        output: cue::OutputTarget {
                            name: "Main L/R".into(),
                        },
                    },
                ],
                indented: false,
                audio_file_name: Some("Wind_Loop.wav".into()),
                ..Default::default()
            },
            Cue {
                number: 2.0,
                name: "Thunder Strike".into(),
                status: CueStatus::Ready,
                follow_mode: FollowMode::Manual,
                tasks: vec![],
                indented: true,
                audio_file_name: Some("Thunder.wav".into()),
                ..Default::default()
            },
            Cue {
                number: 3.0,
                name: "Storm Outro".into(),
                status: CueStatus::Ready,
                follow_mode: FollowMode::AutoFollow,
                tasks: vec![],
                indented: true,
                ..Default::default()
            },
        ];

        let mut tree = DockState::new(vec![Tab::Cuelist]);
        let [cuelist_node, _media_node] = tree.main_surface_mut().split_right(egui_dock::NodeIndex::root(), 0.75, vec![Tab::Media]);
        let _ = tree.main_surface_mut().split_below(cuelist_node, 0.65, vec![Tab::Detail]);

        Self {
            engine: AudioEngine::new(),
            cues,
            selected_cue_index: 0,
            selected_task_index: None,
            search_query: String::new(),
            tree,
        }
    }
}

// ---------------------------------------------------------------------------
// eframe::App implementation

impl eframe::App for ArticuelateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- 1. TOOLBAR (Top) ---
        egui::TopBottomPanel::top("toolbar")
            .min_height(36.0)
            .show(ctx, |ui| {
                panels::toolbar::render(ui, &mut self.search_query, &mut self.engine);
            });

        // --- 2. STATUS BAR (Bottom) ---
        egui::TopBottomPanel::bottom("status_bar")
            .min_height(20.0)
            .show(ctx, |ui| {
                panels::status_bar::render(ui, &self.engine);
            });

        // --- 3. DOCK AREA (Takes all remaining space) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut tab_viewer = MyTabViewer {
                engine: &mut self.engine,
                cues: &mut self.cues,
                selected_cue_index: &mut self.selected_cue_index,
                selected_task_index: &mut self.selected_task_index,
            };
            DockArea::new(&mut self.tree)
                .show_inside(ui, &mut tab_viewer);
        });
    }
}

// ---------------------------------------------------------------------------
// Entry point

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Articuelate"),
        ..Default::default()
    };

    eframe::run_native(
        "Articuelate",
        options,
        Box::new(|cc| {
            catppuccin_egui::set_theme(&cc.egui_ctx, catppuccin_egui::MACCHIATO);
            Ok(Box::new(ArticuelateApp::default()))
        }),
    )
}
