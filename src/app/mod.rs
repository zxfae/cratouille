use crate::task::*;
use crate::priority::*;
use ratatui::widgets::ScrollbarState;
use std::path::PathBuf;
use std::fs::File;
use dirs::config_dir;
use std::io;
use std::fs::create_dir_all;
use std::fs::OpenOptions;
use std::io::Write;

pub struct App {
    pub input: String,
    pub tasks: Vec<Task>,
    pub current_priority: Priority,
    pub selected_index: Option<usize>,
    pub scroll_state: ScrollbarState,
    pub scroll_offset: usize,
    pub filepath: PathBuf,
}

impl App {
    pub fn new() -> App {
        let mut app = App {
            input: String::new(),
            tasks: Vec::new(),
            current_priority: Priority::Medium,
            selected_index: None,
            scroll_state: ScrollbarState::default(),
            scroll_offset: 0,
            filepath: (config_dir().expect("config directory not found"))
            // Cratouille folder created at .config folder
            .join("cratouille")
            .join("tasks.json"),
        };

        app.read_file();
        app
    }

    pub fn add_task(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }

        self.tasks.push(Task {
            description: self.input.clone(),
            priority: self.current_priority.clone(),
            loading: 0,
        });
        self.input.clear();
        self.scroll_state = self.scroll_state.content_length(self.tasks.len());
        self.save_file();
    }

    pub fn cycle_priority(&mut self) {
        self.current_priority = self.current_priority.next();
    }

    pub fn move_selection(&mut self, down: bool, max_visible: usize) {
        let len = self.tasks.len();
        if len == 0 {
            self.selected_index = None;
            return;
        }

        self.selected_index = match self.selected_index {
            None => Some(0),
            Some(i) => {
                let new_index = if down {
                    (i + 1).min(len - 1)
                } else {
                    i.saturating_sub(1)
                };

                if new_index >= self.scroll_offset + max_visible {
                    self.scroll_offset = new_index.saturating_sub(max_visible - 1);
                } else if new_index < self.scroll_offset {
                    self.scroll_offset = new_index;
                }

                Some(new_index)
            }
        };
    }

    pub fn delete_selected_task(&mut self) {
        if let Some(index) = self.selected_index {
            if index < self.tasks.len() {
                self.tasks.remove(index);
                if self.tasks.is_empty() {
                    self.selected_index = None;
                } else {
                    self.selected_index = Some(index.min(self.tasks.len() - 1));
                }
                self.scroll_state = self.scroll_state.content_length(self.tasks.len());
            }
        }
        self.save_file();
    }

    // Read, Write, Create file && Truncate
    pub fn get_file(&self, truncate: bool) -> Result<File, io::Error> {
        if let Some(parent_dir) = self.filepath.parent() {
            create_dir_all(parent_dir)?;
        }

        OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(truncate)
        .open(&self.filepath)
    }

    pub fn save_file(&mut self) {
        // Serialized into a string
        let serialized = match serde_json::to_string_pretty(&self.tasks) {
            Ok(res) => res,
            Err(_) => String::new(),
        };

        match self.get_file(true) {
            Ok(mut file) => {
                let _ = file.write(serialized.as_bytes());
            }
            Err(e) => eprintln!("Error opening file: {}", e),
        }
    }

    pub fn read_file(&mut self) {
        self.tasks = self.get_file(false).map_or_else(
            |_| vec![],
            |file| serde_json::from_reader(file).unwrap_or_else(|_| vec![]),
        );
    }
}
