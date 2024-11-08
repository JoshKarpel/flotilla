mod discovery;
mod state;

use clap::Parser;
use kube::{
    api::{ApiResource, DynamicObject, ListParams},
    Api, Client,
};
use ratatui::{
    layout::{
        Constraint::{Length, Min, Ratio},
        Layout,
    },
    style::{Color, Styled},
    text::Text,
    widgets::{Block, Cell, Paragraph, Row, Table, Tabs},
    DefaultTerminal,
};

use crate::state::{Action, Editing, State};

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
        let client = Client::try_default().await?;
        let discovery = discovery::Discovery::discover(client.clone()).await?;

        for (name, resource) in discovery.name_to_resource {
            println!("{} -> {:?}", name, resource);
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
    let mut state = State::default();

    state.active_tab_mut().resource = "svc".into();

    let client = Client::try_default().await?;
    let discovery = discovery::Discovery::discover(client.clone()).await?;

    let mut table = Table::default();

    loop {
        let tab = state.active_tab();

        let res = discovery.get(&tab.resource);

        if let Some(r) = res {
            let api: Api<DynamicObject> =
                Api::all_with(client.clone(), &ApiResource::from(r.as_ref()));

            // https://ratatui.rs/examples/widgets/table/
            table = Table::new(
                api.list(&ListParams::default())
                    .await?
                    .iter()
                    .filter(|&obj| {
                        obj.metadata
                            .name
                            .clone()
                            .is_some_and(|n| n.starts_with(&tab.filter))
                    })
                    .map(|obj| {
                        [obj.metadata.name.clone().expect("Object missing name")]
                            .into_iter()
                            .map(|c| Cell::from(Text::from(c)))
                            .collect::<Row>()
                    }),
                [Length(50)],
            )
            .block(Block::bordered());
        }

        terminal.draw(|frame| {
            let [tabs_area, meta, _resources_layout] =
                Layout::vertical([Length(1), Length(3), Min(0)]).areas(frame.area());
            let [namespace, resource, filter] =
                Layout::horizontal([Ratio(1, 3), Ratio(1, 3), Ratio(1, 3)]).areas(meta);

            let namespace_p = Paragraph::new(tab.namespace.clone().unwrap_or("".into())).block(
                Block::bordered().title("Namespace").set_style(
                    if let Some(Editing::Namespace) = state.editing {
                        Color::LightCyan
                    } else {
                        Color::White
                    },
                ),
            );
            let resource_p = Paragraph::new(tab.resource.clone())
                .set_style(
                    if res.is_some() {
                        Color::White
                    } else {
                        Color::Red
                    },
                )
                .block(Block::bordered().title("Resource").border_style(
                    if let Some(Editing::Resource) = state.editing {
                        Color::LightCyan
                    } else {
                        Color::White
                    },
                ));
            let filter_p = Paragraph::new(tab.filter.clone()).block(
                Block::bordered().title("Filter").set_style(
                    if let Some(Editing::Filter) = state.editing {
                        Color::LightCyan
                    } else {
                        Color::White
                    },
                ),
            );

            let highlight_style = (Color::default(), Color::Cyan);
            let tabs = Tabs::new(
                state
                    .tabs
                    .iter()
                    .enumerate()
                    .map(|(idx, t)| format!("{idx} {}", t.resource)),
            )
            .highlight_style(highlight_style)
            .select(state.active_tab_idx)
            .padding("", "")
            .divider(" ");

            frame.render_widget(tabs, tabs_area);
            frame.render_widget(namespace_p, namespace);
            frame.render_widget(resource_p, resource);
            frame.render_widget(filter_p, filter);
            frame.render_widget(&table, _resources_layout);
        })?;

        if let Ok(Action::Quit) = state.handle_events() {
            return Ok(());
        }
    }
}
