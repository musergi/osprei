CREATE TABLE IF NOT EXISTS jobs (
    id INTEGER PRIMARY KEY,
    source TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS executions (
    id INTEGER PRIMARY KEY,
    job INTEGER,
    status INTEGER,
    start_time INTEGER,
    end_time INTEGER,
    FOREIGN KEY(job) REFERENCES jobs(id)
);
