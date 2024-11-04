mod state;

use std::io;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind},
    layout::{
        Constraint::{Length, Min, Ratio},
        Layout,
    },
    widgets::{Block, Paragraph},
    DefaultTerminal,
};

use crate::state::State;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal);
    ratatui::restore();
    app_result
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    let state = State::default();

    loop {
        let tab = state.active_tab();

        terminal.draw(|frame| {
            let [meta, resources] = Layout::vertical([Length(3), Min(0)]).areas(frame.area());
            let [namespace, resource, filter] =
                Layout::horizontal([Ratio(1, 3), Ratio(1, 3), Ratio(1, 3)]).areas(meta);

            let namespace_p = Paragraph::new(tab.namespace.clone().unwrap_or("".into()))
                .block(Block::bordered().title("Namespace"));
            let resource_p =
                Paragraph::new(tab.resource.clone()).block(Block::bordered().title("Resource"));
            let filter_p =
                Paragraph::new(tab.filter.clone()).block(Block::bordered().title("Filter"));

            frame.render_widget(namespace_p, namespace);
            frame.render_widget(resource_p, resource);
            frame.render_widget(filter_p, filter);
        })?;

        if let event::Event::Key(KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Char('q'),
            ..
        }) = event::read()?
        {
            return Ok(());
        }
    }
}
