use std::{collections::BTreeMap, sync::Arc};

use chrono::{DateTime, NaiveDateTime, Utc};
use osprei::{ExecutionDetails, ExecutionSummary, Job, JobPointer, Schedule, ScheduleRequest};
use tokio::sync::Mutex;

use super::{ExecutionStore, JobStore, ScheduleStore, Store};

#[derive(Default, Clone)]
pub struct MemoryStore {
    data: Arc<Mutex<MemoryStorage>>,
}

#[derive(Default)]
struct MemoryStorage {
    job_counter: i64,
    jobs: BTreeMap<i64, JobPointer>,
    execution_counter: i64,
    executions: BTreeMap<i64, Execution>,
    schedule_couter: i64,
    schedules: BTreeMap<i64, Schedule>,
}

struct Execution {
    id: i64,
    job_id: i64,
    start_time: u64,
    status: Option<i64>,
}

#[async_trait::async_trait]
impl JobStore for MemoryStore {
    async fn list_jobs(&self) -> Vec<i64> {
        self.data.lock().await.jobs.keys().cloned().collect()
    }

    async fn store_job(&self, name: String, source: String, path: String) -> i64 {
        let mut data = self.data.lock().await;
        let id = data.job_counter;
        data.job_counter += 1;
        let new_job = JobPointer {
            id,
            name,
            source,
            path,
        };
        data.jobs.insert(id, new_job);
        id
    }

    async fn fetch_job(&self, id: i64) -> JobPointer {
        self.data.lock().await.jobs.get(&id).unwrap().clone()
    }

    async fn fetch_job_description(&self, _id: i64) -> Job {
        todo!()
    }
}

#[async_trait::async_trait]
impl ExecutionStore for MemoryStore {
    async fn create_execution(&self, job_id: i64) -> i64 {
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Clock before epoch")
            .as_secs();
        let mut data = self.data.lock().await;
        let id = data.execution_counter;
        data.execution_counter += 1;
        let new_execution = Execution {
            id,
            job_id,
            start_time,
            status: None,
        };
        data.executions.insert(id, new_execution);
        id
    }

    async fn set_execution_status(&self, id: i64, execution_status: i64) {
        let mut data = self.data.lock().await;
        let execution = data.executions.get_mut(&id).unwrap();
        execution.status = Some(execution_status);
    }

    async fn get_execution(&self, id: i64) -> ExecutionDetails {
        let data = self.data.lock().await;
        let Execution {
            id,
            job_id,
            start_time,
            status,
        } = data.executions.get(&id).unwrap();
        let JobPointer { name, .. } = data.jobs.get(job_id).unwrap();
        let job_name = name.clone();
        let start_time = timestamp_str(*start_time);
        let status = *status;
        ExecutionDetails {
            execution_id: *id,
            job_name,
            start_time,
            status,
        }
    }

    async fn last_executions(&self, job_id: i64, limit: usize) -> Vec<ExecutionSummary> {
        let data = self.data.lock().await;
        let mut job_executions: Vec<_> = data
            .executions
            .values()
            .filter(|execution| execution.job_id == job_id)
            .collect();
        job_executions.sort_by(|&a, &b| b.start_time.cmp(&a.start_time));
        job_executions
            .into_iter()
            .take(limit)
            .map(|execution| ExecutionSummary {
                id: execution.id,
                start_time: timestamp_str(execution.start_time),
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl ScheduleStore for MemoryStore {
    async fn create_daily(&self, job_id: i64, request: ScheduleRequest) -> i64 {
        let mut data = self.data.lock().await;
        let id = data.schedule_couter;
        data.schedule_couter += 1;
        let ScheduleRequest { hour, minute } = request;
        data.schedules.insert(
            id,
            Schedule {
                schedule_id: id,
                job_id,
                hour,
                minute,
            },
        );
        id
    }

    async fn get_schedules(&self, job_id: i64) -> Vec<osprei::Schedule> {
        let data = self.data.lock().await;
        data.schedules
            .values()
            .filter(|s| s.job_id == job_id)
            .cloned()
            .collect()
    }

    async fn get_all_schedules(&self) -> Vec<osprei::Schedule> {
        let data = self.data.lock().await;
        data.schedules.values().cloned().collect()
    }
}

fn timestamp_str(secs_since_epoch: u64) -> String {
    let datetime = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp_opt(secs_since_epoch as i64, 0).unwrap(),
        Utc,
    );
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

impl Store for MemoryStore {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::persistance::tests::{test_execution_store, test_job_store, test_schedule_store};

    #[tokio::test]
    async fn test_memory_job_store() {
        let store = MemoryStore::default();
        test_job_store(store).await;
    }

    #[tokio::test]
    async fn test_memory_execution_store() {
        let store = MemoryStore::default();
        test_execution_store(store).await;
    }

    #[tokio::test]
    async fn test_memory_schedule_store() {
        let store = MemoryStore::default();
        test_schedule_store(store).await;
    }
}
