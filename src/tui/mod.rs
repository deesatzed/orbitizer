use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::stdout;
use std::path::PathBuf;

mod state;
mod ui;

pub fn run(root_str: &str, dry_run: bool) -> Result<()> {
    let root = PathBuf::from(root_str);

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut st = state::State::new(root, dry_run)?;

    loop {
        terminal.draw(|f| ui::draw(f, &mut st))?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(k) = event::read()? {
                if k.kind != KeyEventKind::Press {
                    continue;
                }
                if st.search_mode {
                    match k.code {
                        KeyCode::Esc => st.cancel_search(),
                        KeyCode::Enter => st.apply_search(),
                        KeyCode::Backspace => st.backspace_search(),
                        KeyCode::Char(c) => st.push_search(c),
                        _ => {}
                    }
                    continue;
                }

                match k.code {
                    KeyCode::Char('q') => {
                        let _ = st.save_session();
                        break;
                    }
                    KeyCode::Tab => st.next_panel(),
                    KeyCode::BackTab => st.prev_panel(),
                    KeyCode::Up => st.up(),
                    KeyCode::Down => st.down(),
                    KeyCode::Char(' ') => st.toggle_checkbox(),
                    KeyCode::Enter => st.primary_action()?,
                    KeyCode::Char('/') => st.start_search(),
                    KeyCode::Char('f') => st.toggle_pin_selected()?,
                    KeyCode::Char('s') => st.snapshot()?,
                    KeyCode::Char('e') => st.export()?,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
