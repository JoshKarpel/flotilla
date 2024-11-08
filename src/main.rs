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
    style::Color,
    text::Text,
    widgets::{Block, Cell, Paragraph, Row, Table, Tabs},
    DefaultTerminal,
};

use crate::state::{Action, State};

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

    loop {
        let tab = state.active_tab();

        let r = discovery.get(&tab.resource).expect("Unknown resource");
        let api: Api<DynamicObject> = Api::all_with(client.clone(), &ApiResource::from(r.as_ref()));

        // https://ratatui.rs/examples/widgets/table/
        let table = Table::new(
            api.list(&ListParams::default()).await?.iter().map(|obj| {
                [obj.metadata.name.clone().expect("Object missing name")]
                    .into_iter()
                    .map(|c| Cell::from(Text::from(c)))
                    .collect::<Row>()
            }),
            [Length(50)],
        )
        .block(Block::bordered());

        terminal.draw(|frame| {
            let [tabs_area, meta, _resources_layout] =
                Layout::vertical([Length(1), Length(3), Min(0)]).areas(frame.area());
            let [namespace, resource, filter] =
                Layout::horizontal([Ratio(1, 3), Ratio(1, 3), Ratio(1, 3)]).areas(meta);

            let namespace_p = Paragraph::new(tab.namespace.clone().unwrap_or("".into()))
                .block(Block::bordered().title("Namespace"));
            let resource_p =
                Paragraph::new(tab.resource.clone()).block(Block::bordered().title("Resource"));
            let filter_p =
                Paragraph::new(tab.filter.clone()).block(Block::bordered().title("Filter"));

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
            frame.render_widget(table, _resources_layout);
        })?;

        if let Ok(Action::Quit) = state.handle_events() {
            return Ok(());
        }
    }
}
