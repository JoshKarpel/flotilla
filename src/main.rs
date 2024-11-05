mod state;

use std::collections::HashMap;

use itertools::Itertools;
use kube::{
    api::{ApiResource, DynamicObject},
    discovery::verbs,
    Api, Client, Discovery, ResourceExt,
};
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

type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> DynResult<()> {
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
    for group in discovery.groups() {
        for (ar, caps) in group.recommended_resources() {
            if !caps.supports_operation(verbs::LIST) {
                continue;
            }
            let api: Api<DynamicObject> = Api::all_with(client.clone(), &ar);
            // can now api.list() to emulate kubectl get all --all
            for obj in api.list(&Default::default()).await? {
                println!("{} {}: {}", ar.api_version, ar.kind, obj.name_any());
            }
        }
    }

    let _resources: HashMap<String, ApiResource> = discovery
        .groups()
        .flat_map(|group| {
            group
                .recommended_resources()
                .iter()
                .filter(|(_, caps)| caps.supports_operation(verbs::LIST))
                .map(|(res, _)| (res.kind.clone(), res.clone()))
                .collect_vec()
        })
        .collect();

    loop {
        let tab = state.active_tab();

        let r = _resources.get(&tab.resource).expect("Unknown resource");
        let api: Api<DynamicObject> = Api::all_with(client.clone(), r);
        for obj in api.list(&Default::default()).await? {
            println!("{} {}: {}", r.api_version, r.kind, obj.name_any());
        }

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
