use std::{
    io::{self, stdout, Error},
    sync::mpsc,
    time::Duration,
};

use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

use crate::device_adapter::adapter::get_devices;

mod device_adapter;
mod utils;

fn main() -> Result<(), Error> {
    let devices = get_devices();

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    terminal.draw(|rect| {
        let size = rect.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(2),
                Constraint::Length(3),
            ])
            .split(size);
    });

    // devices.iter().for_each(|d| {
    //     d.unlock_device();
    //     d.open_app(&String::from("it.clikapp.toduba"))
    // });
    Ok(())
}
