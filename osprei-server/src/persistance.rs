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
    async fn set_execution_status(&self, id: i64, execution_status: i64);
    async fn get_execution(&self, id: i64) -> osprei::ExecutionDetails;
    async fn last_executions(&self, job_id: i64, limit: usize) -> Vec<osprei::ExecutionSummary>;
}

#[async_trait::async_trait]
pub trait ScheduleStore {
    async fn create_daily(&self, job_id: i64, request: osprei::ScheduleRequest) -> i64;
    async fn get_schedules(&self, job_id: i64) -> Vec<osprei::Schedule>;
    async fn get_all_schedules(&self) -> Vec<osprei::Schedule>;
}

#[async_trait::async_trait]
pub trait Persistance {
    async fn init(&self);
    async fn create_execution(&self, job_name: String) -> i64;
    async fn set_execution_status(&self, execution_id: i64, execution_status: i64);
    async fn get_execution(&self, execution_id: i64) -> osprei::ExecutionDetails;
    async fn last_executions(&self, job_name: String, limit: i64) -> Vec<osprei::ExecutionSummary>;
}

#[cfg(test)]
mod tests {
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

        store.set_execution_status(execution_id, 0).await;
        let execution = store.get_execution(execution_id).await;
        assert_eq!(execution.status, Some(0));
    }
}

pub mod memory;
pub mod sqlite;
