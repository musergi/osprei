# Osprei

## Description

Osprei attempts to solve all your CI needs in a single solution.

## Current roadmap

- [ ] Clean up code
- [ ] Make executable a server
- [ ] Add periodic test execution
- [ ] Add rest trigger for test

# Rest API

```
/job
```
  
```json
[
  "job1_name",
  "job2_name"
]
```

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


```
/job/job1_name/executions
```
```json
[]
```
