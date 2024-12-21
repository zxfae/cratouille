use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
    },
    Terminal,
};
use std::{
    error::Error,
    io::{self},
};

mod priority;
mod task;
mod app;
use crate::app::*;


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
