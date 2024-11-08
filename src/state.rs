use std::{collections::HashMap, io};

use crossterm::{
    event,
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
};
use kube::api::DynamicObject;

use crate::discovery::{DiscoveredAPIResource, Discovery};

#[derive(Debug)]
pub(crate) struct App {
    kube: KubeState,
    ui: UIState,
}

#[derive(Debug)]
pub(crate) struct KubeState {
    discovery: Discovery,
    resources: HashMap<DiscoveredAPIResource, Vec<DynamicObject>>,
}

#[derive(Debug)]
pub(crate) struct UIState {
    pub(crate) tabs: Vec<Tab>,
    pub(crate) active_tab_idx: usize,
    pub(crate) editing: Option<Editing>,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            tabs: vec![Tab::default()],
            active_tab_idx: 0,
            editing: None,
        }
    }
}

#[derive(Debug)]
pub(crate) enum Editing {
    Namespace,
    Resource,
    Filter,
}

#[derive(Debug)]
pub(crate) enum Action {
    Continue,
    Quit,
}

impl UIState {
    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab_idx]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab_idx]
    }

    pub fn handle_events(&mut self) -> io::Result<Action> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key {
                    KeyEvent {
                        modifiers: KeyModifiers::CONTROL,
                        code: KeyCode::Char('t'),
                        ..
                    } => self.new_tab(),
                    KeyEvent {
                        code: KeyCode::Tab, ..
                    } => {
                        self.active_tab_idx = self
                            .active_tab_idx
                            .saturating_add(1)
                            .min(self.tabs.len() - 1)
                    }
                    KeyEvent {
                        code: KeyCode::BackTab,
                        ..
                    } => self.active_tab_idx = self.active_tab_idx.saturating_sub(1),
                    KeyEvent {
                        code: KeyCode::Char('f'),
                        ..
                    } if self.editing.is_none() => self.editing = Some(Editing::Filter),
                    KeyEvent {
                        code: KeyCode::Char('r'),
                        ..
                    } if self.editing.is_none() => self.editing = Some(Editing::Resource),
                    KeyEvent {
                        code: KeyCode::Char('n'),
                        ..
                    } if self.editing.is_none() => self.editing = Some(Editing::Namespace),
                    KeyEvent {
                        code: KeyCode::Char(c),
                        ..
                    } if self.editing.is_some() => match self.editing.as_ref().unwrap() {
                        Editing::Filter => {
                            self.active_tab_mut().filter.push(c);
                        }
                        Editing::Namespace => {
                            if let Some(ref mut n) = self.active_tab_mut().namespace {
                                n.push(c);
                            }
                        }
                        Editing::Resource => {
                            self.active_tab_mut().resource.push(c);
                        }
                    },
                    KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    } if self.editing.is_some() => match self.editing.as_ref().unwrap() {
                        Editing::Filter => {
                            self.active_tab_mut().filter.pop();
                        }
                        Editing::Namespace => {
                            if let Some(ref mut n) = self.active_tab_mut().namespace {
                                n.pop();
                            }
                        }
                        Editing::Resource => {
                            self.active_tab_mut().resource.pop();
                        }
                    },
                    KeyEvent {
                        code: KeyCode::Enter | KeyCode::Esc,
                        ..
                    } if self.editing.is_some() => {
                        self.editing = None;
                    }
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    } => return Ok(Action::Quit),
                    _ => {}
                }
            }
        }

        Ok(Action::Continue)
    }

    fn new_tab(&mut self) {
        self.tabs.push(Tab::default());
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
            resource: "pods".to_string(),
            filter: String::default(),
        }
    }
}
