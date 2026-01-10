use crate::entity::todo;
use chrono::NaiveDate;
use miette::{IntoDiagnostic, Result, bail};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, Order, QueryFilter,
    QueryOrder, Set, sea_query::Expr,
};
use serde_json::Value as JsonValue;
use uuid::Uuid;

const STATUS_DONE: &str = "done";

/// Scope to fetch/move todos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListScope {
    Day(NaiveDate),
    Backlog,
}

/// Pagination and filtering options for listing commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListOptions {
    pub scope: ListScope,
    pub include_done: bool,
    pub workspace_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
}

impl ListOptions {
    pub fn today(date: NaiveDate) -> Self {
        Self {
            scope: ListScope::Day(date),
            include_done: false,
            workspace_id: None,
            project_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovePlacement {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReorderDirection {
    Up,
    Down,
}

#[derive(Clone)]
pub struct TodoService {
    db: DatabaseConnection,
}

impl TodoService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Insert a todo either scheduled for a date or backlog.
    pub async fn add(
        &self,
        title: impl Into<String>,
        scheduled_for: Option<NaiveDate>,
        notes: Option<String>,
        workspace_id: Option<Uuid>,
        project_id: Option<Uuid>,
    ) -> Result<todo::Model> {
        let order_index = self.next_top_order_index(scheduled_for).await?;

        let model = todo::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(title.into()),
            status: Set("pending".to_string()),
            scheduled_for: Set(scheduled_for),
            order_index: Set(order_index),
            notes: Set(notes),
            metadata: Set(JsonValue::Null),
            workspace_id: Set(workspace_id),
            project_id: Set(project_id),
            ..Default::default()
        };

        model.insert(&self.db).await.into_diagnostic()
    }

    /// List todos using the provided filters.
    pub async fn list(&self, opts: ListOptions) -> Result<Vec<todo::Model>> {
        let mut query = todo::Entity::find().filter(scope_condition(opts.scope));

        if !opts.include_done {
            query = query.filter(todo::Column::Status.ne(STATUS_DONE));
        }

        if let Some(workspace_id) = opts.workspace_id {
            query = query.filter(todo::Column::WorkspaceId.eq(workspace_id));
        }

        if let Some(project_id) = opts.project_id {
            query = query.filter(todo::Column::ProjectId.eq(project_id));
        }

        let done_first = Expr::cust("CASE WHEN status = 'done' THEN 1 ELSE 0 END");

        query
            .order_by(done_first, Order::Asc)
            .order_by_asc(todo::Column::OrderIndex)
            .all(&self.db)
            .await
            .into_diagnostic()
    }

    /// Delete a todo by id.
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let res = todo::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .into_diagnostic()?;

        Ok(res.rows_affected > 0)
    }

    /// Mark a todo as complete, ensuring backlog items move into today's column.
    pub async fn mark_done(&self, id: Uuid, today: NaiveDate) -> Result<todo::Model> {
        let model = self.load(id).await?;

        if model.status == STATUS_DONE {
            return Ok(model);
        }

        let scheduled_for = model.scheduled_for.or(Some(today));

        let order_index = self.next_done_order_index(scheduled_for).await?;

        let mut active: todo::ActiveModel = model.into();

        active.status = Set(STATUS_DONE.to_string());
        active.scheduled_for = Set(scheduled_for);
        active.order_index = Set(order_index);

        active.update(&self.db).await.into_diagnostic()
    }

    /// Revert a completed todo back to a pending state.
    pub async fn mark_pending(&self, id: Uuid) -> Result<todo::Model> {
        let model = self.load(id).await?;

        if model.status != STATUS_DONE {
            return Ok(model);
        }

        let scope = model.scheduled_for;
        let target_index = self.next_top_order_index(scope).await?;

        let mut active: todo::ActiveModel = model.into();
        active.status = Set("pending".to_string());
        active.order_index = Set(target_index);

        active.update(&self.db).await.into_diagnostic()
    }

    /// Move overdue todos (scheduled in the past) to today.
    pub async fn rollover_to(&self, today: NaiveDate) -> Result<usize> {
        let overdue = todo::Entity::find()
            .filter(todo::Column::ScheduledFor.lt(today))
            .filter(todo::Column::ScheduledFor.is_not_null())
            .filter(todo::Column::Status.ne(STATUS_DONE))
            .order_by_asc(todo::Column::OrderIndex)
            .all(&self.db)
            .await
            .into_diagnostic()?;

        if overdue.is_empty() {
            return Ok(0);
        }

        let mut next_index = self.next_pending_bottom_index(Some(today)).await?;
        let mut moved = 0usize;

        for model in overdue {
            next_index += 1;

            let mut active: todo::ActiveModel = model.into();

            active.scheduled_for = Set(Some(today));
            active.order_index = Set(next_index);
            active.update(&self.db).await.into_diagnostic()?;

            moved += 1;
        }

        Ok(moved)
    }

    /// Move a todo to another column (day/backlog) placing it at the top or bottom.
    pub async fn move_to_scope(
        &self,
        id: Uuid,
        scope: ListScope,
        placement: MovePlacement,
    ) -> Result<todo::Model> {
        let model = self.load(id).await?;
        let target_date = scope_to_date(scope);

        let target_index = match placement {
            MovePlacement::Top => self.next_top_order_index(target_date).await?,
            MovePlacement::Bottom => {
                if model.status == STATUS_DONE {
                    self.next_done_order_index(target_date).await?
                } else {
                    self.next_pending_bottom_index(target_date).await?
                }
            }
        };

        let mut active: todo::ActiveModel = model.clone().into();

        active.scheduled_for = Set(target_date);
        active.order_index = Set(target_index);

        let updated = active.update(&self.db).await.into_diagnostic()?;

        Ok(updated)
    }

    /// Update the backlog_column field for a backlog item.
    pub async fn set_backlog_column(&self, id: Uuid, column: i64) -> Result<todo::Model> {
        let model = self.load(id).await?;

        let mut active: todo::ActiveModel = model.into();
        active.backlog_column = Set(column);

        active.update(&self.db).await.into_diagnostic()
    }

    /// Get a todo by id.
    pub async fn get(&self, id: Uuid) -> Result<todo::Model> {
        self.load(id).await
    }

    /// Find a todo by title or id.
    pub async fn find_by_title_or_id(&self, title_or_id: &str) -> Result<Option<todo::Model>> {
        let matches = todo::Entity::find()
            .filter(
                Condition::any()
                    .add(todo::Column::Id.eq(title_or_id))
                    .add(todo::Column::Title.eq(title_or_id)),
            )
            .all(&self.db)
            .await
            .into_diagnostic()?;

        match matches.len() {
            0 => Ok(None),
            1 => Ok(Some(matches.into_iter().next().unwrap())),
            _ => bail!(
                "multiple todos match '{}', use the id instead (run `mach list -i` to see ids)",
                title_or_id
            ),
        }
    }

    /// Update the title of a todo.
    pub async fn update_title(&self, id: Uuid, title: String) -> Result<todo::Model> {
        let model = self.load(id).await?;
        let mut active: todo::ActiveModel = model.into();
        active.title = Set(title);
        active.update(&self.db).await.into_diagnostic()
    }

    /// Update the scheduled_for date of a todo.
    pub async fn update_scheduled_for(
        &self,
        id: Uuid,
        scheduled_for: Option<NaiveDate>,
    ) -> Result<todo::Model> {
        let model = self.load(id).await?;
        let mut active: todo::ActiveModel = model.into();
        active.scheduled_for = Set(scheduled_for);
        active.update(&self.db).await.into_diagnostic()
    }

    /// Update the notes of a todo.
    pub async fn update_notes(&self, id: Uuid, notes: Option<String>) -> Result<todo::Model> {
        let model = self.load(id).await?;
        let mut active: todo::ActiveModel = model.into();
        active.notes = Set(notes);
        active.update(&self.db).await.into_diagnostic()
    }

    /// Update the workspace and project of a todo.
    pub async fn update_workspace_project(
        &self,
        id: Uuid,
        workspace_id: Option<Uuid>,
        project_id: Option<Uuid>,
    ) -> Result<todo::Model> {
        let model = self.load(id).await?;
        let mut active: todo::ActiveModel = model.into();
        active.workspace_id = Set(workspace_id);
        active.project_id = Set(project_id);
        active.update(&self.db).await.into_diagnostic()
    }

    /// Reorder within a column/group (pending or done).
    pub async fn reorder(&self, id: Uuid, direction: ReorderDirection) -> Result<()> {
        let model = self.load(id).await?;

        let scope = match model.scheduled_for {
            Some(date) => ListScope::Day(date),
            None => ListScope::Backlog,
        };

        let status = if model.status == STATUS_DONE {
            StatusFilter::Done
        } else {
            StatusFilter::Pending
        };

        let mut tasks = self
            .column_query(scope, status)
            .all(&self.db)
            .await
            .into_diagnostic()?;

        let Some(idx) = tasks.iter().position(|t| t.id == id) else {
            bail!("todo {} no longer exists", id);
        };

        match direction {
            ReorderDirection::Up if idx > 0 => tasks.swap(idx, idx - 1),
            ReorderDirection::Down if idx + 1 < tasks.len() => tasks.swap(idx, idx + 1),
            _ => return Ok(()),
        }

        for (index, task) in tasks.into_iter().enumerate() {
            let mut active: todo::ActiveModel = task.into();

            active.order_index = Set(index as i64);

            active.update(&self.db).await.into_diagnostic()?;
        }

        Ok(())
    }

    async fn load(&self, id: Uuid) -> Result<todo::Model> {
        todo::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .into_diagnostic()?
            .ok_or_else(|| miette::miette!("todo {id} not found"))
    }

    pub async fn stats_for_workspace(&self, workspace_id: Uuid) -> Result<TodoStats> {
        let todos = todo::Entity::find()
            .filter(todo::Column::WorkspaceId.eq(workspace_id))
            .all(&self.db)
            .await
            .into_diagnostic()?;

        let total = todos.len() as u64;
        let completed = todos.iter().filter(|t| t.status == STATUS_DONE).count() as u64;

        Ok(TodoStats {
            total,
            completed,
            remaining: total - completed,
        })
    }

    pub async fn stats_for_project(&self, project_id: Uuid) -> Result<TodoStats> {
        let todos = todo::Entity::find()
            .filter(todo::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .into_diagnostic()?;

        let total = todos.len() as u64;
        let completed = todos.iter().filter(|t| t.status == STATUS_DONE).count() as u64;

        Ok(TodoStats {
            total,
            completed,
            remaining: total - completed,
        })
    }

    fn column_query(
        &self,
        scope: ListScope,
        status: StatusFilter,
    ) -> sea_orm::Select<todo::Entity> {
        let mut query = todo::Entity::find().filter(scope_condition(scope));

        query = match status {
            StatusFilter::Pending => query.filter(todo::Column::Status.ne(STATUS_DONE)),
            StatusFilter::Done => query.filter(todo::Column::Status.eq(STATUS_DONE)),
            StatusFilter::Any => query,
        };

        query.order_by_asc(todo::Column::OrderIndex)
    }

    async fn next_top_order_index(&self, scope_date: Option<NaiveDate>) -> Result<i64> {
        match self
            .find_order_index(scope_date, StatusFilter::Pending, Extremum::Min)
            .await?
        {
            Some(min) => Ok(min - 1),
            None => Ok(0),
        }
    }

    async fn next_pending_bottom_index(&self, scope_date: Option<NaiveDate>) -> Result<i64> {
        Ok(self
            .find_order_index(scope_date, StatusFilter::Pending, Extremum::Max)
            .await?
            .map(|max| max + 1)
            .unwrap_or(0))
    }

    async fn next_done_order_index(&self, scope_date: Option<NaiveDate>) -> Result<i64> {
        Ok(self
            .find_order_index(scope_date, StatusFilter::Any, Extremum::Max)
            .await?
            .map(|max| max + 1)
            .unwrap_or(0))
    }

    async fn find_order_index(
        &self,
        scope_date: Option<NaiveDate>,
        status: StatusFilter,
        extremum: Extremum,
    ) -> Result<Option<i64>> {
        let mut query = todo::Entity::find().filter(scope_condition(match scope_date {
            Some(date) => ListScope::Day(date),
            None => ListScope::Backlog,
        }));

        query = match status {
            StatusFilter::Any => query,
            StatusFilter::Pending => query.filter(todo::Column::Status.ne(STATUS_DONE)),
            StatusFilter::Done => query.filter(todo::Column::Status.eq(STATUS_DONE)),
        };

        query = match extremum {
            Extremum::Min => query.order_by_asc(todo::Column::OrderIndex),
            Extremum::Max => query.order_by_desc(todo::Column::OrderIndex),
        };

        Ok(query
            .one(&self.db)
            .await
            .into_diagnostic()?
            .map(|model| model.order_index))
    }
}

fn scope_condition(scope: ListScope) -> Condition {
    match scope {
        ListScope::Day(date) => Condition::all().add(todo::Column::ScheduledFor.eq(date)),
        ListScope::Backlog => Condition::all().add(todo::Column::ScheduledFor.is_null()),
    }
}

fn scope_to_date(scope: ListScope) -> Option<NaiveDate> {
    match scope {
        ListScope::Day(date) => Some(date),
        ListScope::Backlog => None,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TodoStats {
    pub total: u64,
    pub completed: u64,
    pub remaining: u64,
}

#[derive(Debug, Clone, Copy)]
enum StatusFilter {
    Pending,
    Done,
    Any,
}

#[derive(Debug, Clone, Copy)]
enum Extremum {
    Min,
    Max,
}
