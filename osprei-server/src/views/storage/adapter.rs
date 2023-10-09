use sqlx::sqlite::SqliteRow;
use sqlx::Row;

pub struct JobOverview(osprei::JobOverview);

impl From<SqliteRow> for JobOverview {
    fn from(value: SqliteRow) -> Self {
        let id: i64 = value.get(0);
        let name: String = value.get(1);
        let adapter = LastExecutionAdapter {
            execution_id: value.get(2),
            start_time: value.get(3),
            status: value.get(4),
        };
        Self(osprei::JobOverview {
            id,
            name,
            last_execution: adapter.into(),
        })
    }
}

impl From<JobOverview> for osprei::JobOverview {
    fn from(value: JobOverview) -> Self {
        value.0
    }
}

struct LastExecutionAdapter {
    status: Option<i64>,
    execution_id: Option<i64>,
    start_time: Option<String>,
}

impl From<LastExecutionAdapter> for Option<osprei::LastExecution> {
    fn from(value: LastExecutionAdapter) -> Self {
        let LastExecutionAdapter {
            status,
            execution_id,
            start_time,
        } = value;
        let status = status.map(osprei::ExecutionStatus::from);
        match (execution_id, start_time) {
            (Some(id), Some(start_time)) => Some(osprei::LastExecution {
                id,
                start_time,
                status,
            }),
            _ => None,
        }
    }
}
