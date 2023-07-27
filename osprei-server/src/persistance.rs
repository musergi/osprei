use std::{error::Error, fmt::Display};

use warp::Filter;

type StoreResult<T> = Result<T, StorageError>;

#[async_trait::async_trait]
pub trait Storage: Send + Sync + 'static {
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

#[derive(Debug)]
pub enum StorageError {
    UserError(String),
    InternalError(String),
}

impl Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StorageError::UserError(err) => write!(f, "user error: {}", err),
            StorageError::InternalError(_) => write!(f, "internal error"),
        }
    }
}

impl Error for StorageError {}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum PersistanceConfig {
    Memory,
    Sqlite { path: String },
}

pub async fn build(config: PersistanceConfig) -> StoreResult<Persistances> {
    Ok(match config {
        PersistanceConfig::Memory => Persistances::Memory(memory::MemoryStore::default()),
        PersistanceConfig::Sqlite { path } => {
            Persistances::Sqlite(sqlite::DatabasePersistance::new(&path).await?)
        }
    })
}

#[derive(Clone)]
pub enum Persistances {
    Memory(memory::MemoryStore),
    Sqlite(sqlite::DatabasePersistance),
}

impl Persistances {
    pub fn boxed(&self) -> Box<dyn Storage> {
        match self {
            Persistances::Memory(p) => Box::new(p.clone()),
            Persistances::Sqlite(p) => Box::new(p.clone()),
        }
    }
}

pub fn with(
    persistances: Persistances,
) -> impl Filter<Extract = (Box<dyn Storage>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || persistances.clone().boxed())
}

#[cfg(test)]
mod test {
    use super::StorageError;

    pub async fn listed_jobs_increase_on_job_added(storage: impl super::Storage) {
        let initial_job_ids = storage.list_jobs().await.unwrap();

        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        let _ = storage.store_job(name.clone(), source, path).await.unwrap();

        let new_job_count = storage.list_jobs().await.unwrap().len();
        assert_eq!(new_job_count, initial_job_ids.len() + 1);
    }

    pub async fn get_back_job_when_using_retruned_id(storage: impl super::Storage) {
        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        let inserted_id = storage
            .store_job(name.clone(), source.clone(), path.clone())
            .await
            .unwrap();

        let job = storage.fetch_job(inserted_id).await.unwrap();

        assert_eq!(job.name, name);
        assert_eq!(job.source, source);
        assert_eq!(job.path, path);
    }

    pub async fn using_invalid_id_returs_user_error(storage: impl super::Storage) {
        assert!(storage.list_jobs().await.unwrap().is_empty());
        let res = storage.fetch_job(0).await;
        assert!(matches!(res, Err(StorageError::UserError(_))));
    }

    pub async fn created_execution_added_to_job(storage: impl super::Storage) {
        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        let job_id = storage.store_job(name, source, path).await.unwrap();
        let execution_id = storage.create_execution(job_id).await.unwrap();
        let executions = storage.last_executions(job_id, 10).await.unwrap();
        assert!(executions
            .into_iter()
            .any(|summary| summary.id == execution_id))
    }

    pub async fn inserted_executions_dont_have_status(storage: impl super::Storage) {
        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        let job_id = storage.store_job(name, source, path).await.unwrap();
        let execution_id = storage.create_execution(job_id).await.unwrap();
        let execution = storage.get_execution(execution_id).await.unwrap();
        assert!(execution.status.is_none())
    }

    pub async fn status_properly_saved(storage: impl super::Storage) {
        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        let job_id = storage.store_job(name, source, path).await.unwrap();
        let execution_id = storage.create_execution(job_id).await.unwrap();
        storage
            .set_execution_status(execution_id, osprei::ExecutionStatus::Failed)
            .await
            .unwrap();
        let execution = storage.get_execution(execution_id).await.unwrap();
        assert!(matches!(
            execution.status,
            Some(osprei::ExecutionStatus::Failed)
        ));
        storage
            .set_execution_status(execution_id, osprei::ExecutionStatus::Success)
            .await
            .unwrap();
        let execution = storage.get_execution(execution_id).await.unwrap();
        assert!(matches!(
            execution.status,
            Some(osprei::ExecutionStatus::Success)
        ));
    }
}

mod memory;
mod sqlite;
