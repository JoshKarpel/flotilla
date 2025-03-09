mod discovery;
mod state;
mod table;

use clap::Parser;
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
use unicode_width::UnicodeWidthStr;

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
    let mut ui = UIState::default();

    let client = Client::try_default().await?;
    let discovery = discovery::Discovery::discover(client.clone()).await?;

    let mut table = Table::default();

    loop {
        let tab = ui.active_tab();

        let res = discovery.get(&tab.resource);

        if let Some(r) = res {
            let resource_table: ResourceTable = client
                .request(r.table_request(tab.namespace.as_deref()))
                .await?;

            // https://ratatui.rs/examples/widgets/table/
            let header_strings: Vec<String> = resource_table
                .column_definitions
                .iter()
                .map(|cd| cd.name.clone())
                .collect::<Vec<String>>();
            let row_strings: Vec<Vec<String>> = resource_table
                .rows
                .iter()
                .map(|row| {
                    row.cells
                        .iter()
                        .map(|cell| cell.to_string())
                        .collect::<Vec<String>>()
                })
                .collect();

            let header_row = header_strings
                .iter()
                .map(|s| Cell::from(s.clone()))
                .collect::<Row>();
            let rows = row_strings
                .iter()
                .map(|r| r.iter().map(|s| Cell::from(s.clone())).collect::<Row>());

            // TODO: must account for actual lengths
            let lengths = row_strings
                .iter()
                .map(|r| r.iter().map(|s| s.width()).collect::<Vec<usize>>())
                .fold(
                    header_strings
                        .iter()
                        .map(|s| s.width())
                        .collect::<Vec<usize>>(),
                    |acc, x| {
                        acc.iter()
                            .zip(x)
                            .map(|(a, b)| *a.max(&b))
                            .collect::<Vec<usize>>()
                    },
                )
                .into_iter()
                .map(|l| Length(l as u16));

            table = Table::new(rows, lengths)
                .header(header_row)
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
