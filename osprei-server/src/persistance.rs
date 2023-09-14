use std::{error::Error, fmt::Display};

use warp::Filter;

type StoreResult<T> = Result<T, StorageError>;

#[async_trait::async_trait]
pub trait Storage: Send + Sync + 'static {
    async fn list_jobs_new(&self) -> StoreResult<Vec<osprei::JobOverview>>;

    async fn store_job(&self, name: String, source: String, path: String) -> StoreResult<i64>;

    async fn fetch_job(&self, id: i64) -> StoreResult<osprei::JobPointer>;

    async fn create_execution(&self, job_id: i64) -> StoreResult<i64>;

    async fn set_execution_status(
        &self,
        id: i64,
        execution_status: osprei::ExecutionStatus,
    ) -> StoreResult<()>;

    async fn set_execution_result(
        &self,
        id: i64,
        stdout: String,
        stderr: String,
        execution_status: osprei::ExecutionStatus,
    ) -> StoreResult<()>;

    async fn get_execution(&self, id: i64) -> StoreResult<osprei::ExecutionDetails>;

    async fn create_daily(&self, job_id: i64, request: osprei::ScheduleRequest)
        -> StoreResult<i64>;

    async fn get_all_schedules(&self) -> StoreResult<Vec<osprei::Schedule>>;

    async fn get_stdout(&self, execution_id: i64) -> StoreResult<String>;

    async fn get_stderr(&self, execution_id: i64) -> StoreResult<String>;
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

    pub async fn when_execution_log_stored_it_can_be_retrieved(storage: impl super::Storage) {
        let name = String::from("test_job_name");
        let source = String::from("https://github.com/musergi/osprei.git");
        let path = String::from(".ci/test.json");
        let id = storage.store_job(name, source, path).await.unwrap();
        let stdout = "test stdout".to_string();
        let stderr = "test stderr".to_string();
        let execution_id = storage.create_execution(id).await.unwrap();
        storage
            .set_execution_result(
                execution_id,
                stdout.clone(),
                stderr.clone(),
                osprei::ExecutionStatus::Success,
            )
            .await
            .unwrap();
        assert_eq!(storage.get_stdout(execution_id).await.unwrap(), stdout);
        assert_eq!(storage.get_stderr(execution_id).await.unwrap(), stderr);
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
                use $crate::add_persistance_test;
                use $crate::persistance::test;

                add_persistance_test!(get_back_job_when_using_retruned_id, $init);
                add_persistance_test!(inserted_executions_dont_have_status, $init);
                add_persistance_test!(status_properly_saved, $init);

                add_persistance_test!(when_job_added_listing_increases, $init);
                add_persistance_test!(when_job_not_executed_last_execution_empty, $init);
                add_persistance_test!(when_job_executed_last_execution_present, $init);
                add_persistance_test!(when_job_executed_and_status_set_status_present, $init);
                add_persistance_test!(when_job_executed_and_status_not_set_status_empty, $init);
                add_persistance_test!(when_multiple_jobs_and_only_one_executed_all_lister, $init);
                add_persistance_test!(when_execution_log_stored_it_can_be_retrieved, $init);
            }
        };
    }
}

mod memory;
mod sqlite;
