use crate::entity::todo;
use crate::service::error::TodoError;
use chrono::NaiveDate;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, sea_query::Expr,
};
use serde_json::Value as JsonValue;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, TodoError>;

const STATUS_DONE: &str = "done";

/// Scope to fetch/move todos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListScope {
    Day(NaiveDate),
    Backlog,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ProjectFilter {
    #[default]
    Any,
    Equals(String),
    IsNull,
}

/// Pagination and filtering options for listing commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListOptions {
    pub scope: ListScope,
    pub include_done: bool,
    pub project: ProjectFilter,
    pub epic_id: Option<Uuid>,
}

impl ListOptions {
    pub fn today(date: NaiveDate) -> Self {
        Self {
            scope: ListScope::Day(date),
            include_done: false,
            project: ProjectFilter::Any,
            epic_id: None,
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
        project: Option<String>,
        epic_id: Option<Uuid>,
    ) -> Result<todo::Model> {
        let resolved_project = self
            .resolve_project_with_epic(project, epic_id)
            .await?;

        let order_index = self.next_top_order_index(scheduled_for).await?;

        let model = todo::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(title.into()),
            status: Set("pending".to_string()),
            scheduled_for: Set(scheduled_for),
            order_index: Set(order_index),
            notes: Set(notes),
            project: Set(resolved_project),
            epic_id: Set(epic_id),
            metadata: Set(JsonValue::Null),
            ..Default::default()
        };

        Ok(model.insert(&self.db).await?)
    }

    async fn resolve_project_with_epic(
        &self,
        project: Option<String>,
        epic_id: Option<Uuid>,
    ) -> Result<Option<String>> {
        let Some(epic_id) = epic_id else {
            return Ok(project);
        };

        let epic_project = todo::Entity::find_by_id(epic_id)
            .select_only()
            .column(todo::Column::Project)
            .into_tuple::<(Option<String>,)>()
            .one(&self.db)
            .await?
            .ok_or(TodoError::EpicNotFound(epic_id))?
            .0;

        match (&project, &epic_project) {
            (Some(p), Some(ep)) if p != ep => {
                Err(TodoError::ProjectMismatch(p.clone(), ep.clone()))
            }
            (Some(_), _) => Ok(project),
            (None, _) => Ok(epic_project),
        }
    }

    /// List todos using the provided filters.
    pub async fn list(&self, opts: ListOptions) -> Result<Vec<todo::Model>> {
        let mut query = todo::Entity::find().filter(scope_condition(opts.scope));

        if !opts.include_done {
            query = query.filter(todo::Column::Status.ne(STATUS_DONE));
        }

        query = match opts.project {
            ProjectFilter::Any => query,
            ProjectFilter::Equals(ref p) => query.filter(todo::Column::Project.eq(p.clone())),
            ProjectFilter::IsNull => query.filter(todo::Column::Project.is_null()),
        };

        if let Some(eid) = opts.epic_id {
            query = query.filter(todo::Column::EpicId.eq(eid));
        }

        let done_first = Expr::cust("CASE WHEN status = 'done' THEN 1 ELSE 0 END");

        Ok(query
            .order_by(done_first, Order::Asc)
            .order_by_asc(todo::Column::OrderIndex)
            .all(&self.db)
            .await?)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let children_count = todo::Entity::find()
            .filter(todo::Column::EpicId.eq(id))
            .count(&self.db)
            .await?;

        if children_count > 0 {
            return Err(TodoError::HasChildren(children_count));
        }

        let res = todo::Entity::delete_by_id(id).exec(&self.db).await?;

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

        Ok(active.update(&self.db).await?)
    }


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

        Ok(active.update(&self.db).await?)
    }

    pub async fn rollover_to(&self, today: NaiveDate) -> Result<usize> {
        let overdue = todo::Entity::find()
            .filter(todo::Column::ScheduledFor.lt(today))
            .filter(todo::Column::ScheduledFor.is_not_null())
            .filter(todo::Column::Status.ne(STATUS_DONE))
            .order_by_asc(todo::Column::OrderIndex)
            .all(&self.db)
            .await?;

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
            active.update(&self.db).await?;

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

        Ok(active.update(&self.db).await?)
    }

    pub async fn set_backlog_column(&self, id: Uuid, column: i64) -> Result<todo::Model> {
        let model = self.load(id).await?;

        let mut active: todo::ActiveModel = model.into();
        active.backlog_column = Set(column);

        Ok(active.update(&self.db).await?)
    }


    pub async fn get(&self, id: Uuid) -> Result<todo::Model> {
        self.load(id).await
    }

    pub async fn get_epic_title(&self, epic_id: Uuid) -> Result<String> {
        let epic = self.load(epic_id).await?;
        Ok(epic.title)
    }

    pub async fn get_epic_titles(&self, ids: &[Uuid]) -> Result<std::collections::HashMap<Uuid, String>> {
        use std::collections::HashMap;

        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let epics = todo::Entity::find()
            .filter(todo::Column::Id.is_in(ids.to_vec()))
            .select_only()
            .columns([todo::Column::Id, todo::Column::Title])
            .into_tuple::<(Uuid, String)>()
            .all(&self.db)
            .await?;

        Ok(epics.into_iter().collect())
    }

    pub async fn update_title(&self, id: Uuid, title: String) -> Result<todo::Model> {
        let model = self.load(id).await?;
        let mut active: todo::ActiveModel = model.into();
        active.title = Set(title);
        Ok(active.update(&self.db).await?)
    }

    pub async fn update_scheduled_for(
        &self,
        id: Uuid,
        scheduled_for: Option<NaiveDate>,
    ) -> Result<todo::Model> {
        let model = self.load(id).await?;
        let mut active: todo::ActiveModel = model.into();
        active.scheduled_for = Set(scheduled_for);
        Ok(active.update(&self.db).await?)
    }

    pub async fn update_notes(&self, id: Uuid, notes: Option<String>) -> Result<todo::Model> {
        let model = self.load(id).await?;
        let mut active: todo::ActiveModel = model.into();
        active.notes = Set(notes);
        Ok(active.update(&self.db).await?)
    }

    pub async fn update_project(&self, id: Uuid, project: Option<String>) -> Result<todo::Model> {
        let model = self.load(id).await?;

        if let Some(epic_id) = model.epic_id {
            let epic = self.load(epic_id).await?;
            if let (Some(p), Some(ep)) = (&project, &epic.project)
                && p != ep
            {
                return Err(TodoError::ProjectMismatch(p.clone(), ep.clone()));
            }
        }

        let mut active: todo::ActiveModel = model.into();
        active.project = Set(project);
        Ok(active.update(&self.db).await?)
    }

    pub async fn update_epic_id(&self, id: Uuid, epic_id: Option<Uuid>) -> Result<todo::Model> {
        if epic_id == Some(id) {
            return Err(TodoError::SelfReference);
        }

        let model = self.load(id).await?;

        let resolved_project = self
            .resolve_project_with_epic(model.project.clone(), epic_id)
            .await?;

        let mut active: todo::ActiveModel = model.into();
        active.epic_id = Set(epic_id);
        active.project = Set(resolved_project);
        Ok(active.update(&self.db).await?)
    }

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

        let mut tasks = self.column_query(scope, status).all(&self.db).await?;

        let Some(idx) = tasks.iter().position(|t| t.id == id) else {
            return Err(TodoError::NotFound(id));
        };

        match direction {
            ReorderDirection::Up if idx > 0 => tasks.swap(idx, idx - 1),
            ReorderDirection::Down if idx + 1 < tasks.len() => tasks.swap(idx, idx + 1),
            _ => return Ok(()),
        }

        for (index, task) in tasks.into_iter().enumerate() {
            let mut active: todo::ActiveModel = task.into();

            active.order_index = Set(index as i64);

            active.update(&self.db).await?;
        }

        Ok(())
    }

    async fn load(&self, id: Uuid) -> Result<todo::Model> {
        todo::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(TodoError::NotFound(id))
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

        Ok(query.one(&self.db).await?.map(|model| model.order_index))
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

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;

    async fn test_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory SQLite");

        conn.get_schema_registry("machich::entity::*")
            .sync(&conn)
            .await
            .expect("Failed to sync schema");

        conn
    }

    #[tokio::test]
    async fn add_inherits_project_from_epic() {
        let db = test_db().await;
        let service = TodoService::new(db);

        let epic = service
            .add("epic: Auth", None, None, Some("myapp".into()), None)
            .await
            .unwrap();

        let child = service
            .add("Implement login", None, None, None, Some(epic.id))
            .await
            .unwrap();

        assert_eq!(child.project, Some("myapp".into()));
        assert_eq!(child.epic_id, Some(epic.id));
    }

    #[tokio::test]
    async fn add_rejects_project_mismatch() {
        let db = test_db().await;
        let service = TodoService::new(db);

        let epic = service
            .add("epic: Auth", None, None, Some("myapp".into()), None)
            .await
            .unwrap();

        let result = service
            .add("Wrong project", None, None, Some("other".into()), Some(epic.id))
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("does not match"));
    }

    #[tokio::test]
    async fn delete_blocked_for_epic_with_children() {
        let db = test_db().await;
        let service = TodoService::new(db);

        let epic = service
            .add("epic: Feature", None, None, None, None)
            .await
            .unwrap();

        service
            .add("Sub-task", None, None, None, Some(epic.id))
            .await
            .unwrap();

        let result = service.delete(epic.id).await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("epic with"));
    }

    #[tokio::test]
    async fn delete_allowed_after_children_removed() {
        let db = test_db().await;
        let service = TodoService::new(db);

        let epic = service
            .add("epic: Feature", None, None, None, None)
            .await
            .unwrap();

        let child = service
            .add("Sub-task", None, None, None, Some(epic.id))
            .await
            .unwrap();

        service.delete(child.id).await.unwrap();

        let result = service.delete(epic.id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn update_project_validates_against_epic() {
        let db = test_db().await;
        let service = TodoService::new(db);

        let epic = service
            .add("epic: Auth", None, None, Some("myapp".into()), None)
            .await
            .unwrap();

        let child = service
            .add("Task", None, None, None, Some(epic.id))
            .await
            .unwrap();

        let result = service.update_project(child.id, Some("other".into())).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_filters_by_project() {
        let db = test_db().await;
        let service = TodoService::new(db);

        service
            .add("Task A", None, None, Some("proj-a".into()), None)
            .await
            .unwrap();
        service
            .add("Task B", None, None, Some("proj-b".into()), None)
            .await
            .unwrap();
        service.add("Task C", None, None, None, None).await.unwrap();

        let opts = ListOptions {
            scope: ListScope::Backlog,
            include_done: false,
            project: ProjectFilter::Equals("proj-a".into()),
            epic_id: None,
        };

        let results = service.list(opts).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Task A");
    }

    #[tokio::test]
    async fn list_filters_null_project() {
        let db = test_db().await;
        let service = TodoService::new(db);

        service
            .add("With project", None, None, Some("proj".into()), None)
            .await
            .unwrap();
        service
            .add("No project", None, None, None, None)
            .await
            .unwrap();

        let opts = ListOptions {
            scope: ListScope::Backlog,
            include_done: false,
            project: ProjectFilter::IsNull,
            epic_id: None,
        };

        let results = service.list(opts).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "No project");
    }

    #[tokio::test]
    async fn add_with_nonexistent_epic_id_fails() {
        let db = test_db().await;
        let service = TodoService::new(db);

        let fake_id = Uuid::new_v4();
        let result = service
            .add("Task", None, None, None, Some(fake_id))
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn update_epic_id_to_none_removes_link() {
        let db = test_db().await;
        let service = TodoService::new(db);

        let epic = service
            .add("epic: Feature", None, None, Some("proj".into()), None)
            .await
            .unwrap();

        let child = service
            .add("Task", None, None, None, Some(epic.id))
            .await
            .unwrap();

        assert_eq!(child.epic_id, Some(epic.id));
        assert_eq!(child.project, Some("proj".into()));

        let updated = service.update_epic_id(child.id, None).await.unwrap();

        assert_eq!(updated.epic_id, None);
        assert_eq!(updated.project, Some("proj".into()));
    }

    #[tokio::test]
    async fn update_epic_id_to_valid_epic() {
        let db = test_db().await;
        let service = TodoService::new(db);

        let epic1 = service
            .add("epic: A", None, None, Some("proj".into()), None)
            .await
            .unwrap();

        let epic2 = service
            .add("epic: B", None, None, Some("proj".into()), None)
            .await
            .unwrap();

        let task = service
            .add("Task", None, None, Some("proj".into()), Some(epic1.id))
            .await
            .unwrap();

        let updated = service.update_epic_id(task.id, Some(epic2.id)).await.unwrap();

        assert_eq!(updated.epic_id, Some(epic2.id));
    }

    #[tokio::test]
    async fn update_epic_id_with_nonexistent_epic_fails() {
        let db = test_db().await;
        let service = TodoService::new(db);

        let task = service
            .add("Task", None, None, None, None)
            .await
            .unwrap();

        let fake_id = Uuid::new_v4();
        let result = service.update_epic_id(task.id, Some(fake_id)).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_epic_title_returns_title() {
        let db = test_db().await;
        let service = TodoService::new(db);

        let epic = service
            .add("epic: My Feature", None, None, None, None)
            .await
            .unwrap();

        let title = service.get_epic_title(epic.id).await.unwrap();
        assert_eq!(title, "epic: My Feature");
    }

    #[tokio::test]
    async fn get_epic_titles_returns_batch() {
        let db = test_db().await;
        let service = TodoService::new(db);

        let epic1 = service
            .add("epic: Auth", None, None, None, None)
            .await
            .unwrap();
        let epic2 = service
            .add("epic: Payments", None, None, None, None)
            .await
            .unwrap();

        let titles = service.get_epic_titles(&[epic1.id, epic2.id]).await.unwrap();

        assert_eq!(titles.len(), 2);
        assert_eq!(titles.get(&epic1.id), Some(&"epic: Auth".to_string()));
        assert_eq!(titles.get(&epic2.id), Some(&"epic: Payments".to_string()));
    }

    #[tokio::test]
    async fn get_epic_titles_empty_input_returns_empty() {
        let db = test_db().await;
        let service = TodoService::new(db);

        let titles = service.get_epic_titles(&[]).await.unwrap();
        assert!(titles.is_empty());
    }

    #[tokio::test]
    async fn update_epic_id_rejects_self_reference() {
        let db = test_db().await;
        let service = TodoService::new(db);

        let task = service
            .add("Task", None, None, None, None)
            .await
            .unwrap();

        let result = service.update_epic_id(task.id, Some(task.id)).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("own epic"));
    }
}
