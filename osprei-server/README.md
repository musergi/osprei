# Osprei

## Description

Osprei attempts to solve all your CI needs in a single solution.

## Current roadmap

- [ ] Add periodic test execution
- [ ] Store log references in database
- [ ] Add a shorthand endpoint for last execution status

# Rest API

For listing all the jobs an endpoint is available, it will provide a list of string with all the job names.
```
/job
```
```json
[
  "job1_name",
  "job2_name"
]
```

For fetching the definition of any particular job the `/job/<job-name>` endpoint can be used. It will provide the job config file.
```
/job/job1_name
```
```json
{
  "name": "job1_name",
  "stages": [
    {
      "type": "Source",
      "repository_url": "https://github.com/musergi/osprei.git"
    },
    {
      "type": "Command",
      "cmd": "cargo",
      "args": [
        "test"
      ],
      "path": "."
    }
  ]
}
```

To run a job, simply send a request to `/job/<job-name>/run`, this will automatically start the execution of the job. As a response this endpoint will return the execution id asigned to the job.
```
/job/job1_name/run
```
```json
1
```

You can list the 10 most recen executions of a job by sending a request to `/job/<job-name>/executions`.
```
/job/job1_name/executions
```
```json
[
  {
    "id": 1,
    "start_time":	"2023-05-01 18:07:13"
  },
  {
    "id": 0,
    "start_time":	"2023-05-01 18:02:10"
  }
]
```

To get the details of any execution you can send a request to `/execution/<execution-id>`.
```
/execution/1
```
```json
{
  "execution_id": 1,
  "job_name": "job1_name",
  "start_time": "2023-05-01 18:07:13",
  "status": 0
}
```