use std::{error::Error, fmt::Display};

use warp::Filter;

type StoreResult<T> = Result<T, StorageError>;

#[async_trait::async_trait]
pub trait Storage: Send + Sync + 'static {
    async fn list_jobs_new(&self) -> StoreResult<Vec<osprei::JobOverview>>;

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

    /// Gets the last execution if it exists for the specified job ID.
    ///
    /// # Arguments
    ///
    /// * `job_id` - The ID of the job for which to retrieve the last execution.
    ///
    /// # Returns
    ///
    /// Returns a `StoreResult` containing the latest execution if present. If
    /// the requested job does not exist, a `StorageError::UserError` is
    /// returned. If the job has been executed, its last execution is
    /// returned; otherwise, `None` is returned.
    async fn get_last_execution(&self, job_id: i64) -> StoreResult<Option<osprei::LastExecution>>;
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

    pub async fn last_execution_details(storage: impl super::Storage) {
        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        let job_id = storage.store_job(name, source, path).await.unwrap();
        let last_execution = storage.get_last_execution(job_id).await.unwrap();
        assert!(last_execution.is_none());
        let execution_id = storage.create_execution(job_id).await.unwrap();
        storage
            .set_execution_status(execution_id, osprei::ExecutionStatus::Failed)
            .await
            .unwrap();
        let last_execution = storage.get_last_execution(job_id).await.unwrap().unwrap();
        assert_eq!(last_execution.id, execution_id);
        assert_eq!(last_execution.status, Some(osprei::ExecutionStatus::Failed));
    }

    pub async fn when_job_not_executed_last_execution_empty(storage: impl super::Storage) {
        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        storage.store_job(name, source, path).await.unwrap();
        let job_listing = storage.list_jobs_new().await.unwrap();
        assert!(job_listing
            .get(0)
            .expect("no job found")
            .last_execution
            .is_none())
    }

    pub async fn when_job_added_listing_increases(storage: impl super::Storage) {
        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        storage.store_job(name, source, path).await.unwrap();
        let job_listing = storage.list_jobs_new().await.unwrap();
        assert_eq!(job_listing.len(), 1)
    }

    pub async fn when_job_executed_last_execution_present(storage: impl super::Storage) {
        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        let id = storage.store_job(name, source, path).await.unwrap();
        storage.create_execution(id).await.unwrap();
        let job_listing = storage.list_jobs_new().await.unwrap();
        assert!(job_listing
            .get(0)
            .expect("no job found")
            .last_execution
            .is_some())
    }

    pub async fn when_job_executed_and_status_set_status_present(storage: impl super::Storage) {
        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        let id = storage.store_job(name, source, path).await.unwrap();
        let execution_id = storage.create_execution(id).await.unwrap();
        storage
            .set_execution_status(execution_id, osprei::ExecutionStatus::Success)
            .await
            .unwrap();
        let job_listing = storage.list_jobs_new().await.unwrap();
        assert_eq!(
            job_listing
                .get(0)
                .expect("no job found")
                .last_execution
                .as_ref()
                .expect("execution not found")
                .status
                .expect("status not set"),
            osprei::ExecutionStatus::Success
        )
    }

    pub async fn when_job_executed_and_status_not_set_status_empty(storage: impl super::Storage) {
        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        let id = storage.store_job(name, source, path).await.unwrap();
        storage.create_execution(id).await.unwrap();
        let job_listing = storage.list_jobs_new().await.unwrap();
        assert!(job_listing
            .get(0)
            .expect("no job found")
            .last_execution
            .as_ref()
            .expect("execution not found")
            .status
            .is_none())
    }

    pub async fn when_multiple_jobs_and_only_one_executed_all_lister(storage: impl super::Storage) {
        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        let id = storage
            .store_job(name.clone(), source.clone(), path.clone())
            .await
            .unwrap();
        storage.create_execution(id).await.unwrap();
        storage.store_job(name, source, path).await.unwrap();
        let job_listing = storage.list_jobs_new().await.unwrap();
        assert_eq!(job_listing.len(), 2)
    }

    #[macro_export]
    macro_rules! add_persistance_test {
        ($test_name: ident, $init: expr) => {
            #[tokio::test]
            async fn $test_name() {
                let store = $init().await;
                test::$test_name(store).await;
            }
        };
    }

    #[macro_export]
    macro_rules! test_store {
        ($init: expr) => {
            mod storage {
                use super::*;
                use crate::add_persistance_test;
                use crate::persistance::test;

                add_persistance_test!(get_back_job_when_using_retruned_id, $init);
                add_persistance_test!(created_execution_added_to_job, $init);
                add_persistance_test!(inserted_executions_dont_have_status, $init);
                add_persistance_test!(status_properly_saved, $init);
                add_persistance_test!(last_execution_details, $init);

                add_persistance_test!(when_job_added_listing_increases, $init);
                add_persistance_test!(when_job_not_executed_last_execution_empty, $init);
                add_persistance_test!(when_job_executed_last_execution_present, $init);
                add_persistance_test!(when_job_executed_and_status_set_status_present, $init);
                add_persistance_test!(when_job_executed_and_status_not_set_status_empty, $init);
                add_persistance_test!(when_multiple_jobs_and_only_one_executed_all_lister, $init);
            }
        };
    }
}

mod memory;
mod sqlite;
