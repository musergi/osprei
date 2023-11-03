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

CREATE TABLE IF NOT EXISTS stages (
    id INTEGER PRIMARY KEY,
    dependency INTEGER,
    job INTEGER,
    definition TEXT NOT NULL,
    FOREIGN KEY(job) REFERENCES jobs(id),
    FOREIGN KEY(dependency) REFERENCES stages(id)
);

INSERT INTO jobs (source) VALUES ('https://github.com/musergi/osprei.git');
INSERT INTO stages (dependency, job, definition) VALUES (NULL, 1, '{"name":"cargo build","command":["cargo","build"],"environment":[]}');