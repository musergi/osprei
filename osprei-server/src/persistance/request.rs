#[derive(Debug)]
pub struct Job {
    pub name: String,
    pub source: String,
    pub path: String,
}

#[derive(Debug)]
pub struct Execution {
    pub id: i64,
    pub status: osprei::ExecutionStatus,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug)]
pub struct Schedule {
    pub job_id: i64,
    pub hour: u8,
    pub minute: u8,
}
