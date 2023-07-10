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
            let executions = self.executions(job_id).await;
            let status = match executions.get(0) {
                Some(osprei::ExecutionSummary { id, .. }) => {
                    let osprei::ExecutionDetails { status, .. } = self.execution(*id).await;
                    match status {
                        Some(0) => "Success",
                        None => "Running",
                        _ => "Failure",
                    }
                }
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

    async fn executions(&self, id: i64) -> Vec<osprei::ExecutionSummary> {
        let url = format!("{}/job/{}/executions", self.url, id);
        let response = reqwest::get(&url).await.unwrap().text().await.unwrap();
        serde_json::from_str(&response).unwrap()
    }

    async fn execution(&self, id: i64) -> osprei::ExecutionDetails {
        let url = format!("{}/execution/{}", self.url, id);
        let response = reqwest::get(&url).await.unwrap().text().await.unwrap();
        serde_json::from_str(&response).unwrap()
    }
}

pub struct JobLine {
    pub id: i64,
    pub name: String,
    pub status: String,
}
