mod discovery;
mod state;

use std::collections::HashMap;

use clap::Parser;
use crossterm::event::KeyEvent;
use itertools::Itertools;
use kube::{
    api::{ApiResource, DynamicObject, ListParams},
    discovery::verbs,
    Api, Client, Discovery,
};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{
        Constraint::{Length, Min, Ratio},
        Layout,
    },
    text::Text,
    widgets::{Block, Cell, Paragraph, Row, Table},
    DefaultTerminal,
};

use crate::state::State;

#[derive(Parser, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    discovery: bool,
}

pub type DynResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> DynResult<()> {
    let cli = Cli::parse();

    if cli.discovery {
        let resources = discovery::discover_api_resources().await?;

        for (k, v) in resources.iter().sorted_by_key(|&(k, _)| k.clone()) {
            println!("{k:?} -> {v:?}");
        }

        return Ok(());
    }

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal).await;
    ratatui::restore();
    app_result
}

async fn run(mut terminal: DefaultTerminal) -> DynResult<()> {
    let state = State::default();

    let client = Client::try_default().await?;
    let discovery = Discovery::new(client.clone()).run().await?;

    let _resources: HashMap<String, ApiResource> = discovery
        .groups()
        .flat_map(|group| {
            group
                .recommended_resources()
                .iter()
                .filter(|(_, caps)| caps.supports_operation(verbs::LIST))
                .flat_map(|(res, _)| {
                    [
                        (res.kind.to_lowercase(), res.clone()),
                        (res.plural.to_lowercase(), res.clone()),
                    ]
                })
                .collect_vec()
        })
        .collect();

    loop {
        let tab = state.active_tab();

        let r = _resources.get(&tab.resource).expect("Unknown resource");
        let api: Api<DynamicObject> = Api::all_with(client.clone(), r);

        // https://ratatui.rs/examples/widgets/table/
        let table = Table::new(
            api.list(&ListParams::default()).await?.iter().map(|obj| {
                [obj.metadata.name.clone().expect("Object missing name")]
                    .into_iter()
                    .map(|c| Cell::from(Text::from(c)))
                    .collect::<Row>()
            }),
            [Length(10)],
        )
        .block(Block::bordered());

        terminal.draw(|frame| {
            let [meta, _resources_layout] =
                Layout::vertical([Length(3), Min(0)]).areas(frame.area());
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
            frame.render_widget(table, _resources_layout);
        })?;

        if let Event::Key(KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Char('q'),
            ..
        }) = event::read()?
        {
            return Ok(());
        }
    }
}
