CREATE TABLE jobs (
    id INTEGER PRIMARY KEY,
    source TEXT NOT NULL
);

CREATE TABLE executions (
    id INTEGER PRIMARY KEY,
    job INTEGER,
    status INTEGER,
    start_time INTEGER,
    end_time INTEGER,
    FOREIGN KEY(job) REFERENCES jobs(id)
);

CREATE TABLE stages (
    id INTEGER PRIMARY KEY,
    dependency INTEGER,
    job INTEGER,
    definition TEXT NOT NULL,
    FOREIGN KEY(job) REFERENCES jobs(id),
    FOREIGN KEY(dependency) REFERENCES stages(id)
);

CREATE TABLE templates (
    name TEXT NOT NULL,
    definition TEXT NOT NULL
);

INSERT INTO templates (
    name,
    definition
) VALUES (
    'sqlx',
    '{
        "name": "sqlx",
        "image": "ghcr.io/musergi/sqlx:latest",
        "command": [
            "sqlx",
            "database",
            "setup"
        ],
        "environment": [
            {
                "name": "DATABASE_URL",
                "value": "sqlite:testing.db"
            }
        ]
    }'
);

INSERT INTO templates (
    name,
    definition
) VALUES (
    'build',
    '{
        "name": "build",
        "image": "rust:latest",
        "command": [
            "cargo",
            "build"
        ],
        "environment": [
            {
                "name": "DATABASE_URL",
                "value": "sqlite:testing.db"
            }
        ]
    }'
);
