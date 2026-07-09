mod cue;
mod engine;
mod panels;

use cue::{Cue, CueStatus, FollowMode};
use elegance::Theme;
use engine::AudioEngine;
use egui_tiles::{Behavior, TileId, Tiles, Tree, UiResponse};

// ---------------------------------------------------------------------------
// Pane definition

#[derive(Debug, Clone, PartialEq)]
enum Pane {
    Cuelist,
    Detail,
    Media,
}

fn build_tree() -> Tree<Pane> {
    let mut tiles = Tiles::default();

    // Leaf panes
    let cuelist_pane = tiles.insert_pane(Pane::Cuelist);
    let detail_pane = tiles.insert_pane(Pane::Detail);
    let media_pane = tiles.insert_pane(Pane::Media);

    // Wrap each pane in a single-tab tab container — this is the foundation
    // for the "multiple tabs" requirement on the detail and sidebar panels.
    let cuelist = tiles.insert_tab_tile(vec![cuelist_pane]);
    let detail = tiles.insert_tab_tile(vec![detail_pane]);
    let media = tiles.insert_tab_tile(vec![media_pane]);

    // Left side: vertical split (cuelist on top, detail on the bottom).
    let left = tiles.insert_vertical_tile(vec![cuelist, detail]);

    // Root: horizontal split (left side, media on the right).
    let root = tiles.insert_horizontal_tile(vec![left, media]);

    Tree::new("articuelate_tree", root, tiles)
}

// ---------------------------------------------------------------------------
// Tile behavior — controls how each pane is rendered and styled.

struct AppBehavior<'a> {
    engine: &'a mut AudioEngine,
    cues: &'a mut Vec<Cue>,
    selected_cue_index: &'a mut usize,
    selected_task_index: &'a mut Option<usize>,
}

impl<'a> Behavior<Pane> for AppBehavior<'a> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: TileId,
        pane: &mut Pane,
    ) -> UiResponse {
        match pane {
            Pane::Cuelist => {
                panels::cuelist::render_header(ui);
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        panels::cuelist::render(ui, self.cues, self.selected_cue_index);
                    });
            }
            Pane::Detail => {
                let selected_cue = self.cues.get(*self.selected_cue_index);
                let selected_task = selected_cue.and_then(|cue| {
                    self.selected_task_index.map(|i| &cue.tasks[i])
                });
                panels::detail::render(ui, selected_cue, selected_task);
            }
            Pane::Media => {
                let playbacks = self.engine.active_playbacks();
                panels::media::render(ui, &playbacks);
            }
        }
        UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Cuelist => "Main Cuelist".into(),
            Pane::Detail => "Detail Inspector".into(),
            Pane::Media => "Active Media".into(),
        }
    }

    // Slightly more spacious / modern tab bar.
    fn tab_bar_height(&self, _style: &egui::Style) -> f32 {
        30.0
    }

    // Visible gap between tiles for a clearer separation.
    fn gap_width(&self, _style: &egui::Style) -> f32 {
        4.0
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
    tree: Tree<Pane>,
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

        Self {
            engine: AudioEngine::new(),
            cues,
            selected_cue_index: 0,
            selected_task_index: None,
            search_query: String::new(),
            tree: build_tree(),
        }
    }
}

// ---------------------------------------------------------------------------
// eframe::App implementation

impl eframe::App for ArticuelateApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // 1. Install the egui-elegance theme (dark, slate-blue) once per frame.
        //    This rewrites `egui::Style` so built-in panels and widgets inherit
        //    the modern palette, corner radius, padding and focus ring.
        Theme::slate().install(ui.ctx());

        // 2. Toolbar (top).
        egui::Panel::top("toolbar").show_inside(ui, |ui| {
            panels::toolbar::render(ui, &mut self.search_query, &mut self.engine);
        });

        // 3. Status bar (bottom).
        egui::Panel::bottom("status_bar").show_inside(ui, |ui| {
            panels::status_bar::render(ui, &self.engine);
        });

        // 4. Dock tree (cuelist + detail + media) — fills the rest.
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let mut behavior = AppBehavior {
                engine: &mut self.engine,
                cues: &mut self.cues,
                selected_cue_index: &mut self.selected_cue_index,
                selected_task_index: &mut self.selected_task_index,
            };
            self.tree.ui(&mut behavior, ui);
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
        Box::new(|_cc| Ok(Box::new(ArticuelateApp::default()))),
    )
}