use std::{collections::BTreeMap, sync::Arc};

use chrono::{DateTime, NaiveDateTime, Utc};
use osprei::{
    ExecutionDetails, ExecutionSummary, JobPointer, LastExecution, Schedule, ScheduleRequest,
};
use tokio::sync::Mutex;

use super::{Storage, StorageError, StoreResult};

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

#[derive(Debug, Clone)]
struct Execution {
    id: i64,
    job_id: i64,
    start_time: u64,
    status: Option<osprei::ExecutionStatus>,
}

fn timestamp_str(secs_since_epoch: u64) -> String {
    let datetime = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp_opt(secs_since_epoch as i64, 0).expect("Negative timestamp"),
        Utc,
    );
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[async_trait::async_trait]
impl Storage for MemoryStore {
    async fn list_jobs(&self) -> StoreResult<Vec<i64>> {
        let ids = self.data.lock().await.jobs.keys().cloned().collect();
        Ok(ids)
    }

    async fn list_jobs_new(&self) -> StoreResult<Vec<osprei::JobOverview>> {
        let data = self.data.lock().await;
        let mut res = Vec::new();
        for (_, JobPointer { id, name, .. }) in data.jobs.iter() {
            let last_execution = data
                .executions
                .values()
                .filter(|execution| execution.job_id == *id)
                .max_by_key(|execution| execution.start_time)
                .cloned()
                .map(
                    |Execution {
                         id,
                         start_time,
                         status,
                         ..
                     }| LastExecution {
                        id,
                        start_time: timestamp_str(start_time),
                        status,
                    },
                );
            res.push(osprei::JobOverview {
                id: id.clone(),
                name: name.clone(),
                last_execution,
            });
        }
        Ok(res)
    }

    async fn store_job(&self, name: String, source: String, path: String) -> StoreResult<i64> {
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
        Ok(id)
    }

    async fn fetch_job(&self, id: i64) -> StoreResult<osprei::JobPointer> {
        let job = self
            .data
            .lock()
            .await
            .jobs
            .get(&id)
            .ok_or_else(|| StorageError::UserError(String::from("Invalid job id")))?
            .clone();
        Ok(job)
    }

    async fn fetch_job_description(&self, _id: i64) -> StoreResult<osprei::Job> {
        todo!()
    }

    async fn create_execution(&self, job_id: i64) -> StoreResult<i64> {
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
        Ok(id)
    }

    async fn set_execution_status(
        &self,
        id: i64,
        execution_status: osprei::ExecutionStatus,
    ) -> StoreResult<()> {
        let mut data = self.data.lock().await;
        let execution = data
            .executions
            .get_mut(&id)
            .ok_or_else(|| StorageError::UserError(String::from("Invalid excution id")))?;
        execution.status = Some(execution_status);
        Ok(())
    }

    async fn get_execution(&self, id: i64) -> StoreResult<osprei::ExecutionDetails> {
        let data = self.data.lock().await;
        let Execution {
            id,
            job_id,
            start_time,
            status,
        } = data
            .executions
            .get(&id)
            .ok_or_else(|| StorageError::UserError(String::from("Invalid job id")))?;
        let JobPointer { name, .. } = data
            .jobs
            .get(job_id)
            .ok_or_else(|| StorageError::UserError(String::from("Invalid execution id")))?;
        let job_name = name.clone();
        let start_time = timestamp_str(*start_time);
        let status = *status;
        let execution = ExecutionDetails {
            execution_id: *id,
            job_name,
            start_time,
            status,
        };
        Ok(execution)
    }

    async fn last_executions(
        &self,
        job_id: i64,
        limit: usize,
    ) -> StoreResult<Vec<osprei::ExecutionSummary>> {
        let data = self.data.lock().await;
        let mut job_executions: Vec<_> = data
            .executions
            .values()
            .filter(|execution| execution.job_id == job_id)
            .collect();
        job_executions.sort_by(|&a, &b| b.start_time.cmp(&a.start_time));
        let summaries = job_executions
            .into_iter()
            .take(limit)
            .map(|execution| ExecutionSummary {
                id: execution.id,
                start_time: timestamp_str(execution.start_time),
            })
            .collect();
        Ok(summaries)
    }

    async fn create_daily(
        &self,
        job_id: i64,
        request: osprei::ScheduleRequest,
    ) -> StoreResult<i64> {
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
        Ok(id)
    }

    async fn get_schedules(&self, job_id: i64) -> StoreResult<Vec<osprei::Schedule>> {
        let data = self.data.lock().await;
        let schedules = data
            .schedules
            .values()
            .filter(|s| s.job_id == job_id)
            .cloned()
            .collect();
        Ok(schedules)
    }

    async fn get_all_schedules(&self) -> StoreResult<Vec<osprei::Schedule>> {
        let data = self.data.lock().await;
        let schedules = data.schedules.values().cloned().collect();
        Ok(schedules)
    }

    async fn get_last_execution(&self, job_id: i64) -> StoreResult<Option<osprei::LastExecution>> {
        let data = self.data.lock().await;
        let job = data
            .executions
            .iter()
            .filter(|(_, execution)| execution.job_id == job_id)
            .max_by_key(|(_, exec)| exec.start_time)
            .map(
                |(
                    id,
                    Execution {
                        start_time, status, ..
                    },
                )| LastExecution {
                    id: *id,
                    start_time: timestamp_str(*start_time),
                    status: *status,
                },
            );
        Ok(job)
    }
}

#[cfg(test)]
mod test {
    use super::MemoryStore;
    use crate::test_store;

    async fn create() -> MemoryStore {
        MemoryStore::default()
    }

    test_store!(create);
}
