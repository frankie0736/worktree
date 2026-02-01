//! UI rendering for TUI.

use ratatui::{prelude::*, widgets::Paragraph};

use crate::models::TaskStatus;

use super::app::{App, TaskDisplay};

/// Main draw function
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Layout: header, task list, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Min(1),    // Task list
            Constraint::Length(2), // Footer
        ])
        .split(area);

    draw_header(frame, chunks[0], app);
    draw_tasks(frame, chunks[1], app);
    draw_footer(frame, chunks[2], app);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let running = app
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Running)
        .count();
    let done = app
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Done)
        .count();

    let text = Line::from(vec![
        Span::styled(" wt status", Style::default().fg(Color::Cyan).bold()),
        Span::raw("                                    "),
        Span::styled(format!("{} running", running), Style::default().fg(Color::Green)),
        Span::raw(" · "),
        Span::styled(format!("{} done", done), Style::default().fg(Color::Blue)),
    ]);

    frame.render_widget(Paragraph::new(text), area);

    // Draw separator line
    if area.height > 1 {
        let sep_area = Rect::new(area.x, area.y + 1, area.width, 1);
        let sep = "─".repeat(area.width as usize);
        frame.render_widget(
            Paragraph::new(sep).style(Style::default().fg(Color::DarkGray)),
            sep_area,
        );
    }
}

fn draw_tasks(frame: &mut Frame, area: Rect, app: &App) {
    if app.tasks.is_empty() {
        let text = Paragraph::new(" No running or done tasks.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, area);
        return;
    }

    let mut lines = Vec::new();

    for (i, task) in app.tasks.iter().enumerate() {
        let is_selected = i == app.selected;
        let line = format_task_line(task, is_selected, area.width as usize);
        lines.push(line);
    }

    let text = Text::from(lines);
    frame.render_widget(Paragraph::new(text), area);
}

fn format_task_line(task: &TaskDisplay, selected: bool, _width: usize) -> Line<'static> {
    let mut spans = Vec::new();

    // Selection indicator
    if selected {
        spans.push(Span::styled(" ▸ ", Style::default().fg(Color::Yellow)));
    } else {
        spans.push(Span::raw("   "));
    }

    // Status icon with color
    let (icon, icon_color) = get_status_icon(task);
    spans.push(Span::styled(icon, Style::default().fg(icon_color)));
    spans.push(Span::raw(" "));

    // Task name (fixed width)
    let name = format!("{:<14}", truncate(&task.name, 14));
    let name_style = if selected {
        Style::default().fg(Color::White).bold()
    } else {
        Style::default().fg(Color::White)
    };
    spans.push(Span::styled(name, name_style));

    // Duration or "done"
    let duration = task
        .duration
        .as_ref()
        .map(|d| format!("{:>6}", d))
        .unwrap_or_else(|| "  done".to_string());
    spans.push(Span::styled(
        format!("   {}", duration),
        Style::default().fg(Color::DarkGray),
    ));

    // Context bar
    spans.push(Span::raw("   "));
    spans.extend(render_context_bar(task.context_percent));
    spans.push(Span::styled(
        format!(" {:>3}%", task.context_percent),
        Style::default().fg(Color::DarkGray),
    ));

    // Changes
    let changes = format!("   {:>+5} {:>-5}", task.additions, format!("-{}", task.deletions));
    let changes_color = if task.additions > 0 || task.deletions > 0 {
        Color::Gray
    } else {
        Color::DarkGray
    };
    spans.push(Span::styled(changes, Style::default().fg(changes_color)));

    Line::from(spans)
}

fn get_status_icon(task: &TaskDisplay) -> (&'static str, Color) {
    if task.status == TaskStatus::Done {
        return ("◉", Color::Blue);
    }

    // Running task
    if !task.tmux_alive {
        return ("⚠", Color::Yellow);
    }

    if task.active {
        ("●", Color::Green)
    } else {
        ("○", Color::DarkGray)
    }
}

fn render_context_bar(percent: u8) -> Vec<Span<'static>> {
    let filled = (percent as usize * 10) / 100;
    let empty = 10 - filled;

    let bar_color = if percent >= 95 {
        Color::Red
    } else if percent >= 80 {
        Color::Yellow
    } else {
        Color::Cyan
    };

    vec![
        Span::styled("▰".repeat(filled), Style::default().fg(bar_color)),
        Span::styled("▱".repeat(empty), Style::default().fg(Color::DarkGray)),
    ]
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    // Separator
    let sep = "─".repeat(area.width as usize);
    frame.render_widget(
        Paragraph::new(sep).style(Style::default().fg(Color::DarkGray)),
        Rect::new(area.x, area.y, area.width, 1),
    );

    // Keybindings - context sensitive
    if area.height > 1 {
        let help_area = Rect::new(area.x, area.y + 1, area.width, 1);

        let mut spans = vec![
            Span::raw(" "),
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(" navigate  "),
            Span::styled("⏎", Style::default().fg(Color::Yellow)),
            Span::raw(" cd  "),
        ];

        // Context-sensitive actions based on selected task
        if let Some(task) = app.selected_task() {
            if task.status == TaskStatus::Done {
                // Done: r (review), m (merged)
                spans.push(Span::styled("r", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(" review  "));
                spans.push(Span::styled("m", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(" merged  "));
            } else if task.status == TaskStatus::Running && !task.tmux_alive {
                // Running but tmux exited: d (done)
                spans.push(Span::styled("d", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(" done  "));
            }
        }

        spans.push(Span::styled("q", Style::default().fg(Color::Yellow)));
        spans.push(Span::raw(" quit"));

        frame.render_widget(Paragraph::new(Line::from(spans)), help_area);
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
