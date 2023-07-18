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

type StoreResult<T> = Result<T, StorageError>;

#[async_trait::async_trait]
pub trait Storage {
    /// Lists all valid job identifiers.
    ///
    /// # Returns
    ///
    /// Returns a `StoreResult` containing a vector of valid job identifiers.
    /// If the operation is successful, the vector of job identifiers is
    /// returned. Otherwise, a store error is returned.
    async fn list_jobs(&self) -> StoreResult<Vec<i64>>;

    /// Stores a job into the database and returns the newly created job ID.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the job being stored.
    /// * `source` - The Git repository URL to the source code to test.
    /// * `path` - The relative path from the root of the repository to the job
    ///   description.
    ///
    /// # Returns
    ///
    /// Returns the ID of the newly created job wrapped in a `StoreResult`.
    /// If the job is successfully stored, its ID is returned.
    /// Otherwise, a store error is returned.
    async fn store_job(&self, name: String, source: String, path: String) -> StoreResult<i64>;

    /// Fetches the job pointer from the database.
    ///
    /// It is important to note that the job description is not stored in the
    /// database. Instead, it may be cached and later returned by the
    /// corresponding method.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the job for which the job pointer is fetched.
    ///
    /// # Returns
    ///
    /// Returns the job pointer wrapped in a `StoreResult`.
    /// If the requested ID is valid, the job pointer is returned.
    /// Otherwise, a user error is returned.
    async fn fetch_job(&self, id: i64) -> StoreResult<osprei::JobPointer>;

    /// Fetches the job description for a given job ID, if available.
    ///
    /// On job registration, no attempt is made to fetch the job description.
    /// This approach aims to enhance the agility of the registration process.
    /// The primary goal of this store is to provide job definitions for each
    /// execution, enabling better traceability of each execution.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the job for which the job description is fetched.
    ///
    /// # Returns
    ///
    /// Returns the job description wrapped in a `StoreResult`.
    /// If the job description is available, it is returned.
    /// Otherwise, a store error is returned.
    async fn fetch_job_description(&self, id: i64) -> StoreResult<osprei::Job>;

    /// Creates an execution for a given job.
    ///
    /// # Arguments
    ///
    /// * `job_id` - The ID of the job for which the execution is created.
    ///
    /// # Returns
    ///
    /// Returns the ID of the newly created execution wrapped in a
    /// `StoreResult`. If the requested job ID is valid, the execution is
    /// created and its ID is returned. Otherwise, a user error is returned.
    async fn create_execution(&self, job_id: i64) -> StoreResult<i64>;

    /// Sets the status of an execution to the specified status.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the execution whose status is being set.
    /// * `execution_status` - The status to set for the execution.
    ///
    /// # Returns
    ///
    /// Returns a `StoreResult` indicating the success of the operation.
    /// If the execution with the specified ID exists, its status is updated to
    /// the specified status. Otherwise, a user error is returned.
    async fn set_execution_status(
        &self,
        id: i64,
        execution_status: osprei::ExecutionStatus,
    ) -> StoreResult<()>;

    /// Fetches the execution with the specified ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the execution to fetch.
    ///
    /// # Returns
    ///
    /// Returns an `osprei::ExecutionDetails` wrapped in a `StoreResult`.
    /// If the requested ID is valid, the execution details are returned.
    /// Otherwise, a user error is returned.
    async fn get_execution(&self, id: i64) -> StoreResult<osprei::ExecutionDetails>;

    /// Fetches the last `limit` executions of the job with the specified ID.
    ///
    /// # Arguments
    ///
    /// * `job_id` - The ID of the job for which to fetch the last executions.
    /// * `limit` - The maximum number of executions to fetch.
    ///
    /// # Returns
    ///
    /// Returns a `StoreResult` containing a vector of `ExecutionSummary` for
    /// the last executions. If the job ID is invalid or the limit is zero,
    /// negative, or too large, a user error is returned. Otherwise, the
    /// vector of execution summaries is returned.
    async fn last_executions(
        &self,
        job_id: i64,
        limit: usize,
    ) -> StoreResult<Vec<osprei::ExecutionSummary>>;

    /// Creates a daily schedule for the requested job ID at the specified hour
    /// and minute.
    ///
    /// # Arguments
    ///
    /// * `job_id` - The ID of the job for which to create a daily schedule.
    /// * `request` - The schedule request containing the desired hour and
    ///   minute.
    ///
    /// # Returns
    ///
    /// Returns a `StoreResult` indicating the success of the operation.
    /// If the job ID is invalid, a `StorageError::UserError` is returned.
    /// On success, the ID of the created schedule is returned.
    async fn create_daily(&self, job_id: i64, request: osprei::ScheduleRequest)
        -> StoreResult<i64>;

    /// Gets all the schedules for a particular job ID.
    ///
    /// # Arguments
    ///
    /// * `job_id` - The ID of the job for which to get the schedules.
    ///
    /// # Returns
    ///
    /// Returns a `StoreResult` containing a vector of schedules.
    /// If the job ID is invalid, a `StorageError::UserError` is returned.
    /// Otherwise, the vector of schedules is returned.
    async fn get_schedules(&self, job_id: i64) -> StoreResult<Vec<osprei::Schedule>>;

    /// Gets all the schedules for all jobs.
    ///
    /// # Returns
    ///
    /// Returns a `StoreResult` containing a vector of schedules.
    /// If the operation encounters an error, a store error is returned.
    /// Otherwise, the vector of schedules is returned.
    async fn get_all_schedules(&self) -> StoreResult<Vec<osprei::Schedule>>;
}

pub enum StorageError {
    UserError(String),
    InternalError(String),
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
