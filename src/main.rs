use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dirs::config_dir;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Terminal,
};
use std::{
    error::Error,
    fs::{create_dir_all, File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    fn get_color(&self) -> Color {
        match self {
            Priority::Low => Color::Rgb(9, 245, 33),
            Priority::Medium => Color::Rgb(245, 151, 9),
            Priority::High => Color::Rgb(245, 9, 9),
        }
    }

    fn next(&self) -> Self {
        match self {
            Priority::Low => Priority::Medium,
            Priority::Medium => Priority::High,
            Priority::High => Priority::Low,
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct Task {
    description: String,
    priority: Priority,
}

struct App {
    input: String,
    tasks: Vec<Task>,
    current_priority: Priority,
    selected_index: Option<usize>,
    scroll_state: ScrollbarState,
    scroll_offset: usize,

    filepath: PathBuf,
}

impl App {
    fn new() -> App {
        let mut app = App {
            input: String::new(),
            tasks: Vec::new(),
            current_priority: Priority::Medium,
            selected_index: None,
            scroll_state: ScrollbarState::default(),
            scroll_offset: 0,
            filepath: (config_dir().expect("config directory not found"))
                .join("cratouille")
                .join("tasks.json"),
        };

        app.read_file();
        app
    }

    fn add_task(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }

        self.tasks.push(Task {
            description: self.input.clone(),
            priority: self.current_priority.clone(),
        });
        self.input.clear();
        self.scroll_state = self.scroll_state.content_length(self.tasks.len());
        self.save_file();
    }

    fn cycle_priority(&mut self) {
        self.current_priority = self.current_priority.next();
    }

    fn move_selection(&mut self, down: bool, max_visible: usize) {
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

    fn delete_selected_task(&mut self) {
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

    fn get_file(&self, truncate: bool) -> Result<File, io::Error> {
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

    fn save_file(&mut self) {
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

    fn read_file(&mut self) {
        self.tasks = self.get_file(false).map_or_else(
            |_| vec![],
            |file| serde_json::from_reader(file).unwrap_or_else(|_| vec![]),
        );
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }
    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let main_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .title("╔══ Cratouille By ZxFae33 ══╗")
                .title_alignment(Alignment::Center)
                .style(Style::default().fg(Color::Rgb(41, 186, 56)));

            let area = main_block.inner(f.area());
            f.render_widget(main_block, f.area());

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(5),
                        Constraint::Length(3),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(area);

            let header_text = vec![
                Span::styled(
                    "Task",
                    Style::default()
                        .fg(Color::Rgb(103, 241, 34))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    "Manager",
                    Style::default()
                        .fg(Color::Rgb(241, 103, 34))
                        .add_modifier(Modifier::BOLD),
                ),
            ];

            let header = Paragraph::new(Line::from(header_text))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(header, chunks[0]);

            let max_visible = (chunks[1].height as usize).saturating_sub(2);

            let visible_tasks = app
                .tasks
                .iter()
                .enumerate()
                .skip(app.scroll_offset)
                .take(max_visible);

            let items: Vec<ListItem> = visible_tasks
                .map(|(i, task)| {
                    let bullet = if Some(i) == app.selected_index {
                        "►"
                    } else {
                        "•"
                    };
                    let content = format!(
                        "{} {} [{}]",
                        bullet,
                        task.description,
                        task.priority.as_str()
                    );
                    ListItem::new(content).style(if Some(i) == app.selected_index {
                        Style::default()
                            .fg(task.priority.get_color())
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(task.priority.get_color())
                    })
                })
                .collect();

            let tasks_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .title(format!(
                    " Tasks ({}/{}) ",
                    if app.tasks.is_empty() {
                        0
                    } else {
                        app.scroll_offset + 1
                    },
                    app.tasks.len()
                ))
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::Rgb(150, 150, 150)));

            let tasks = List::new(items).block(tasks_block);
            f.render_widget(tasks, chunks[1]);

            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            f.render_stateful_widget(
                scrollbar,
                chunks[1],
                &mut app.scroll_state.position(app.scroll_offset),
            );

            let title = format!(" Priority: {} ", app.current_priority.as_str());
            let input = Paragraph::new(app.input.as_str())
                .style(Style::default().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(title)
                        .title_alignment(Alignment::Center)
                        .border_style(Style::default().fg(app.current_priority.get_color())),
                );
            f.render_widget(input, chunks[2]);

            let help =
                " 📝 [Enter] Add | 🎯 [F1] Priority | ⬆️⬇️ Select | ❌ [Del] Delete | 🚪 [ESC] Quit ";
            let content3 = Paragraph::new(help)
                .style(Style::default().fg(Color::Rgb(180, 180, 180)))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(content3, chunks[3]);
        })?;

        if let Event::Key(key) = event::read()? {
            let max_visible = (terminal.size()?.height as usize).saturating_sub(12);
            match key.code {
                KeyCode::Esc => return Ok(()),
                KeyCode::F(1) => app.cycle_priority(),
                KeyCode::Delete => app.delete_selected_task(),
                KeyCode::Char(c) => {
                    app.input.push(c);
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Enter => {
                    app.add_task();
                }
                KeyCode::Up => app.move_selection(false, max_visible),
                KeyCode::Down => app.move_selection(true, max_visible),
                _ => {}
            }
        }
    }
}
