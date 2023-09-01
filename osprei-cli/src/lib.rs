use osprei::LastExecutionResponse;

pub struct Client {
    url: String,
}

impl Client {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn jobs_list(&self) -> Vec<JobLine> {
        let mut lines = Vec::new();
        for job_id in self.list_jobs().await {
            let osprei::JobPointer { name, .. } = self.job(job_id).await;
            let execution = self.last_execution(job_id).await;
            let status = match execution {
                Some(osprei::LastExecution { status, .. }) => match status {
                    Some(osprei::ExecutionStatus::Success) => "Success",
                    Some(osprei::ExecutionStatus::Failed) => "Failed",
                    Some(osprei::ExecutionStatus::InvalidConfig) => "Invalid config",
                    None => "Running",
                },
                None => "Not executed",
            }
            .to_string();
            lines.push(JobLine {
                id: job_id,
                name,
                status,
            });
        }
        lines
    }

    async fn list_jobs(&self) -> Vec<i64> {
        let url = format!("{}/job", self.url);
        let response = reqwest::get(&url).await.unwrap().text().await.unwrap();
        serde_json::from_str(&response).unwrap()
    }

    async fn job(&self, id: i64) -> osprei::JobPointer {
        let url = format!("{}/job/{}", self.url, id);
        let response = reqwest::get(&url).await.unwrap().text().await.unwrap();
        serde_json::from_str(&response).unwrap()
    }

    async fn last_execution(&self, id: i64) -> Option<osprei::LastExecution> {
        let url = format!("{}/job/{}/last", self.url, id);
        let response = reqwest::get(&url).await.unwrap().text().await.unwrap();
        let last_execution: LastExecutionResponse = serde_json::from_str(&response).unwrap();
        last_execution.last
    }
}

pub struct JobLine {
    pub id: i64,
    pub name: String,
    pub status: String,
}

pub struct Handler {
    client: Client,
}

impl Handler {
    pub fn new(client: Client) -> Self {
        Handler { client }
    }

    pub async fn handle_list(&self) {
        let jobs = self.client.jobs_list().await;
        if jobs.is_empty() {
            println!("No jobs yet, try adding a job.");
        } else {
            for JobLine { id, name, status } in jobs {
                println!("- id: {}", id);
                println!("  name: {}", name);
                println!("  status: {}", status);
            }
        }
    }
}
