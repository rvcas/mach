use chrono::NaiveDate;
use uuid::Uuid;

use crate::service::config::WeekStart;

#[derive(Clone)]
pub enum UiMode {
    Board,
    Backlog,
    Settings(SettingsState),
    AddTodo(AddTodoState),
    Detail(DetailState),
}

#[derive(Clone)]
pub struct SettingsState {
    pub week_start: WeekStart,
}

#[derive(Clone)]
pub struct AddTodoState {
    pub input: String,
    pub target: AddTarget,
}

#[derive(Clone)]
pub enum AddTarget {
    Day(NaiveDate),
    BacklogColumn(usize),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DetailField {
    Title,
    Project,
    Epic,
    Date,
    Status,
    Notes,
}

impl DetailField {
    pub fn next(self) -> Self {
        match self {
            Self::Title => Self::Project,
            Self::Project => Self::Epic,
            Self::Epic => Self::Date,
            Self::Date => Self::Status,
            Self::Status => Self::Notes,
            Self::Notes => Self::Notes,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Title => Self::Title,
            Self::Project => Self::Title,
            Self::Epic => Self::Project,
            Self::Date => Self::Epic,
            Self::Status => Self::Date,
            Self::Notes => Self::Status,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Project => "Project",
            Self::Epic => "Epic",
            Self::Date => "Date",
            Self::Status => "Status",
            Self::Notes => "Notes",
        }
    }

    pub fn is_editable(self) -> bool {
        !matches!(self, Self::Status | Self::Epic)
    }
}

#[derive(Clone)]
pub struct DetailState {
    pub todo_id: Uuid,
    pub title: String,
    pub project: Option<String>,
    pub epic_title: Option<String>,
    pub date: Option<NaiveDate>,
    pub status: String,
    pub notes: String,
    pub field: DetailField,
    pub editing: Option<String>,
    pub from_backlog: bool,
    pub error: Option<String>,
}

impl DetailState {
    pub fn field_value(&self, field: DetailField) -> String {
        match field {
            DetailField::Title => self.title.clone(),
            DetailField::Project => self.project.clone().unwrap_or_default(),
            DetailField::Epic => self.epic_title.clone().unwrap_or_default(),
            DetailField::Date => self
                .date
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "none".to_string()),
            DetailField::Status => self.status.clone(),
            DetailField::Notes => self.notes.clone(),
        }
    }
}
