mod discovery;
mod state;
mod table;

use std::{fs::File, io::Write};

use clap::Parser;
use http::Request;
use kube::Client;
use ratatui::{
    layout::{
        Constraint::{Length, Min, Ratio},
        Layout,
    },
    style::{Color, Styled},
    widgets::{Block, Cell, Paragraph, Row, Table, Tabs},
    DefaultTerminal,
};

use crate::{
    state::{Action, Editing, UIState},
    table::ResourceTable,
};

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

        // for (name, resource) in discovery.name_to_resource {
        //     println!("{} -> {:?}", name, resource);
        // }

        let request = Request::builder()
            .uri("/api/v1/namespace/default/pods")
            .header("Accept", "application/json;as=Table;g=meta.k8s.io;v=v1")
            .body(vec![])?;
        let response: ResourceTable = client.request(request).await?;
        //
        dbg!(response);

        return Ok(());
    }

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal).await;
    ratatui::restore();
    app_result
}

async fn run(mut terminal: DefaultTerminal) -> DynResult<()> {
    let mut log = File::create("flotilla.log")?;
    let mut ui = UIState::default();

    let client = Client::try_default().await?;
    let discovery = discovery::Discovery::discover(client.clone()).await?;

    let mut table = Table::default();

    loop {
        let tab = ui.active_tab();

        let res = discovery.get(&tab.resource);

        if let Some(r) = res {
            let req = Request::builder()
                .uri(r.url_path(tab.namespace.as_deref()))
                .body(vec![])
                .unwrap();
            log.write_all(req.uri().to_string().as_bytes())?;
            log.flush()?;
            let resource_table: ResourceTable = client.request(req).await?;

            // https://ratatui.rs/examples/widgets/table/
            let header = resource_table
                .column_definitions
                .iter()
                .map(|cd| Cell::from(cd.name.clone()))
                .collect::<Row>();
            let rows = resource_table.rows.iter().map(|row| {
                row.cells
                    .iter()
                    .map(|cell| Cell::from(cell.clone()))
                    .collect::<Row>()
            });
            table = Table::new(
                rows,
                [Length(10), Length(10), Length(10), Length(10), Length(10)],
            )
            .header(header)
            .block(Block::bordered());
        }

        terminal.draw(|frame| {
            let [tabs_area, meta, _resources_layout] =
                Layout::vertical([Length(1), Length(3), Min(0)]).areas(frame.area());
            let [namespace_selector, resource_selector, name_filter] =
                Layout::horizontal([Ratio(1, 3), Ratio(1, 3), Ratio(1, 3)]).areas(meta);

            let namespace_p = Paragraph::new(tab.namespace.clone().unwrap_or("".into())).block(
                Block::bordered().title("Namespace").set_style(
                    if let Some(Editing::Namespace) = ui.editing {
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
                    if let Some(Editing::Resource) = ui.editing {
                        Color::LightCyan
                    } else {
                        Color::White
                    },
                ));
            let filter_p = Paragraph::new(tab.filter.clone()).block(
                Block::bordered().title("Filter").set_style(
                    if let Some(Editing::Filter) = ui.editing {
                        Color::LightCyan
                    } else {
                        Color::White
                    },
                ),
            );

            let highlight_style = (Color::default(), Color::Cyan);
            let tabs = Tabs::new(
                ui.tabs
                    .iter()
                    .enumerate()
                    .map(|(idx, t)| format!("{idx} {}", t.resource)),
            )
            .highlight_style(highlight_style)
            .select(ui.active_tab_idx)
            .padding("", "")
            .divider(" ");

            frame.render_widget(tabs, tabs_area);
            frame.render_widget(namespace_p, namespace_selector);
            frame.render_widget(resource_p, resource_selector);
            frame.render_widget(filter_p, name_filter);
            frame.render_widget(&table, _resources_layout);
        })?;

        if let Ok(Action::Quit) = ui.handle_events() {
            return Ok(());
        }
    }
}
