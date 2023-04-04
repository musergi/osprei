CREATE TABLE execution (
  id INTEGER NOT NULL,
  job_name VARCHAR NOT NULL,
  start_time TIMESTAMP NOT NULL,
  status INTEGER,
  PRIMARY KEY (id)
)
