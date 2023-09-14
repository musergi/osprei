pub struct Client {
    url: String,
}

impl Client {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn jobs_list(&self) -> Vec<JobLine> {
        let mut lines = Vec::new();
        for osprei::JobOverview {
            id,
            name,
            last_execution,
        } in self.list_jobs().await
        {
            let (execution_id, status) = match last_execution {
                Some(osprei::LastExecution { id, status, .. }) => match status {
                    Some(osprei::ExecutionStatus::Success) => (Some(id), "Success"),
                    Some(osprei::ExecutionStatus::Failed) => (Some(id), "Failed"),
                    Some(osprei::ExecutionStatus::InvalidConfig) => (Some(id), "Invalid config"),
                    None => (Some(id), "Running"),
                },
                None => (None, "Not executed"),
            };
            let status = status.to_string();
            lines.push(JobLine {
                id,
                name,
                execution_id,
                status,
            });
        }
        lines
    }

    async fn list_jobs(&self) -> Vec<osprei::JobOverview> {
        let url = format!("{}/job", self.url);
        let response = reqwest::get(&url).await.unwrap().text().await.unwrap();
        serde_json::from_str(&response).unwrap()
    }
}

pub struct JobLine {
    pub id: i64,
    pub name: String,
    pub execution_id: Option<i64>,
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
            for JobLine {
                id,
                name,
                execution_id,
                status,
            } in jobs
            {
                println!("- id: {}", id);
                println!("  name: {}", name);
                if let Some(execution_id) = execution_id {
                    println!("  execution_id: {}", execution_id);
                }
                println!("  status: {}", status);
            }
        }
    }
}
