use warp::Filter;

#[async_trait::async_trait]
pub trait JobStore {
    async fn list_jobs(&self) -> Vec<i64>;
    async fn store_job(&self, name: String, source: String, path: String) -> i64;
    async fn fetch_job(&self, id: i64) -> osprei::JobPointer;
    async fn fetch_job_description(&self, id: i64) -> osprei::Job;
}

#[async_trait::async_trait]
pub trait ExecutionStore {
    async fn create_execution(&self, job_id: i64) -> i64;
    async fn set_execution_status(&self, id: i64, execution_status: osprei::ExecutionStatus);
    async fn get_execution(&self, id: i64) -> osprei::ExecutionDetails;
    async fn last_executions(&self, job_id: i64, limit: usize) -> Vec<osprei::ExecutionSummary>;
}

#[async_trait::async_trait]
pub trait ScheduleStore {
    async fn create_daily(&self, job_id: i64, request: osprei::ScheduleRequest) -> i64;
    async fn get_schedules(&self, job_id: i64) -> Vec<osprei::Schedule>;
    async fn get_all_schedules(&self) -> Vec<osprei::Schedule>;
}

pub trait Store: JobStore + ExecutionStore + ScheduleStore + Send + Sync + 'static {}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum PersistanceConfig {
    Memory,
    Sqlite { path: String },
}

pub async fn build(config: PersistanceConfig) -> Persistances {
    match config {
        PersistanceConfig::Memory => Persistances::Memory(memory::MemoryStore::default()),
        PersistanceConfig::Sqlite { path } => {
            Persistances::Sqlite(sqlite::DatabasePersistance::new(&path).await)
        }
    }
}

#[derive(Clone)]
pub enum Persistances {
    Memory(memory::MemoryStore),
    Sqlite(sqlite::DatabasePersistance),
}

impl Persistances {
    pub fn boxed(&self) -> Box<dyn Store> {
        match self {
            Persistances::Memory(p) => Box::new(p.clone()),
            Persistances::Sqlite(p) => Box::new(p.clone()),
        }
    }
}

pub fn with(
    persistances: Persistances,
) -> impl Filter<Extract = (Box<dyn Store>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || persistances.clone().boxed())
}

#[cfg(test)]
mod tests {
    use osprei::ScheduleRequest;

    pub async fn test_job_store<T: super::JobStore>(store: T) {
        let job_ids = store.list_jobs().await;
        let initial_job_count = job_ids.len();

        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        let stored_id = store.store_job(name.clone(), source, path).await;

        let job_ids = store.list_jobs().await;
        assert_eq!(job_ids.len(), initial_job_count + 1);

        let fetched = store.fetch_job(stored_id).await;
        assert_eq!(&fetched.id, &stored_id);
        assert_eq!(&fetched.name, &name);
    }

    pub async fn test_execution_store<T: super::ExecutionStore + super::JobStore>(store: T) {
        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        let job_id = store.store_job(name.clone(), source, path).await;

        let execution_id = store.create_execution(job_id).await;
        let recent_executions = store.last_executions(job_id, 1).await;
        let most_recent_execution = recent_executions.get(0).unwrap();
        assert_eq!(most_recent_execution.id, execution_id);

        let execution = store.get_execution(execution_id).await;
        assert_eq!(execution.job_name, name);
        assert!(execution.status.is_none());

        store
            .set_execution_status(execution_id, osprei::ExecutionStatus::Success)
            .await;
        let execution = store.get_execution(execution_id).await;
        assert_eq!(execution.status, Some(osprei::ExecutionStatus::Success));
    }

    pub async fn test_schedule_store<T: super::ScheduleStore + super::JobStore>(store: T) {
        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        let job_id = store.store_job(name.clone(), source, path).await;

        let schedule_id = store
            .create_daily(
                job_id,
                ScheduleRequest {
                    hour: 12,
                    minute: 0,
                },
            )
            .await;
        let schedules = store.get_schedules(job_id).await;
        assert_eq!(schedules.len(), 1);
        let schedule = schedules.get(0).unwrap();
        assert_eq!(schedule.schedule_id, schedule_id);
        assert_eq!(schedule.job_id, job_id);
        assert_eq!(schedule.hour, 12);
        assert_eq!(schedule.minute, 0);

        assert_eq!(store.get_all_schedules().await.len(), 1);
    }
}

mod memory;
mod sqlite;
