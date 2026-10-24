#[path = "../font.rs"]
mod font;

use eframe::egui::{self, Event, ImeEvent, Pos2, Vec2};
use serde_json::{Map, Value};
use std::{
    env,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

const WINDOW_SIZE: [f32; 2] = [1280.0, 900.0];
const COMPOSER_SIZE: Vec2 = Vec2::new(560.0, 88.0);
const TRACE_PATH_ENV: &str = "KCU_TRACE_PATH";

fn main() {
    if let Err(error) = run() {
        eprintln!("egui IME reference failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    require_ci_runner()?;
    let trace_path = trace_path()?;
    let trace = Arc::new(Mutex::new(
        TraceWriter::open(trace_path).map_err(|error| error.to_string())?,
    ));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(WINDOW_SIZE),
        ..Default::default()
    };
    let trace_for_app = Arc::clone(&trace);
    eframe::run_native(
        "harness egui-reference",
        options,
        Box::new(move |creation_context| {
            let _ = font::ManualFontInstaller::install(&creation_context.egui_ctx);
            creation_context
                .egui_ctx
                .set_visuals(egui::Visuals::light());
            Ok(Box::new(ReferenceApp::new(trace_for_app)))
        }),
    )
    .map_err(|error| error.to_string())
}

fn require_ci_runner() -> Result<(), String> {
    if env::var("GITHUB_ACTIONS").ok().as_deref() != Some("true")
        || env::var("RUNNER_OS").ok().as_deref() != Some("macOS")
    {
        return Err("must run only on a GitHub Actions macOS runner".into());
    }
    Ok(())
}

fn trace_path() -> Result<PathBuf, String> {
    let path = env::var_os(TRACE_PATH_ENV)
        .map(PathBuf::from)
        .ok_or_else(|| format!("{TRACE_PATH_ENV} is required"))?;
    if !path.is_absolute() {
        return Err(format!("{TRACE_PATH_ENV} must be an absolute path"));
    }
    Ok(path)
}

struct ReferenceApp {
    text: String,
    trace: Arc<Mutex<TraceWriter>>,
}

impl ReferenceApp {
    fn new(trace: Arc<Mutex<TraceWriter>>) -> Self {
        Self {
            text: String::new(),
            trace,
        }
    }

    fn record(&self, value: Value) {
        let result = self
            .trace
            .lock()
            .map_err(|_| "trace writer lock poisoned".to_string())
            .and_then(|mut trace| trace.write(value));
        if let Err(error) = result {
            eprintln!("KCU_REFERENCE_TRACE_ERROR: {error}");
            std::process::exit(1);
        }
    }

    fn record_input(&self, event: &Event) {
        let mut record = Map::new();
        record.insert("kind".into(), Value::String("egui_input".into()));
        record.insert("event".into(), Value::String(format!("{event:?}")));
        self.record(Value::Object(record));
        match event {
            Event::Ime(ImeEvent::Preedit { .. }) => {}
            Event::Ime(ImeEvent::Commit(text)) => self.record_command(text, "ImeCommit"),
            Event::Text(text) => self.record_command(text, "Text"),
            _ => {}
        }
    }

    fn record_command(&self, text: &str, source: &str) {
        let mut record = Map::new();
        record.insert("kind".into(), Value::String("command".into()));
        record.insert(
            "command".into(),
            Value::String(format!("DraftChanged({text:?})")),
        );
        record.insert("source".into(), Value::String(source.into()));
        self.record(Value::Object(record));
    }
}

impl eframe::App for ReferenceApp {
    fn raw_input_hook(&mut self, _context: &egui::Context, raw_input: &mut egui::RawInput) {
        for event in &raw_input.events {
            self.record_input(event);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let available = ui.available_rect_before_wrap();
        let composer_position = Pos2::new(
            available.center().x - COMPOSER_SIZE.x / 2.0,
            available.max.y - 80.0 - COMPOSER_SIZE.y,
        );
        let response = ui.put(
            egui::Rect::from_min_size(composer_position, COMPOSER_SIZE),
            egui::TextEdit::multiline(&mut self.text).hint_text(
                egui::RichText::new("Ask anything")
                    .color(egui::Color32::BLACK)
                    .size(18.0),
            ),
        );
        let ime = ui.ctx().output(|output| output.ime);
        let mut platform_output = Map::new();
        platform_output.insert("kind".into(), Value::String("platform_output".into()));
        platform_output.insert("ime".into(), Value::String(format!("{ime:?}")));
        self.record(Value::Object(platform_output));
        let mut text_edit = Map::new();
        text_edit.insert("kind".into(), Value::String("text_edit".into()));
        text_edit.insert("buffer".into(), Value::String(self.text.as_str().into()));
        text_edit.insert("focused".into(), Value::Bool(response.has_focus()));
        self.record(Value::Object(text_edit));
        ui.ctx().request_repaint_after(Duration::from_millis(50));
    }
}

struct TraceWriter {
    file: File,
}

impl TraceWriter {
    fn open(path: PathBuf) -> io::Result<Self> {
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        Ok(Self { file })
    }

    fn write(&mut self, value: Value) -> Result<(), String> {
        serde_json::to_writer(&mut self.file, &value).map_err(|error| error.to_string())?;
        self.file
            .write_all(b"\n")
            .and_then(|_| self.file.flush())
            .map_err(|error| error.to_string())
    }
}
