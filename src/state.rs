#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct State {
    tabs: Vec<Tab>,
    active_tab_idx: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            tabs: vec![Tab::default()],
            active_tab_idx: 0,
        }
    }
}

impl State {
    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab_idx]
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct Tab {
    pub(crate) namespace: Option<String>,
    pub(crate) resource: String,
    pub(crate) filter: String,
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            namespace: Some("default".to_string()),
            resource: "pod".to_string(),
            filter: String::default(),
        }
    }
}
