# Controller

## Configuration

| Environment variable | Default                 | Description                    |
|:---------------------|-------------------------|--------------------------------|
| `DATABASE_LOCATION`  | `/var/lib/rik/data/`    | Database data location         |
| `SCHEDULER_URL`      | `http://localhost:4996` | Host location of the scheduler |
| `PORT`               | `5000`                  | Port to listen on              |


## Database structure

**Workloads**:

* `element_type`: `/workload`

* `element_id`: `/workload/${WORKLOAD_KIND}/${NAMESPACE}/${WORKLOAD_NAME}`
    * *WORKLOAD_KIND*: One of`pods`, `function`
    * *NAMESPACE*: Static `default`
    * *WORKLOAD_NAME*: Dynamically defined


**Instances**:

* `element_type`: `/instance`

* `element_id`: `/instance/${WORKLOAD_KIND}/${NAMESPACE}/${INSTANCE_NAME}`
  * *WORKLOAD_KIND*: One of`pods`, `function`
  * *NAMESPACE*: Static `default`
  * *INSTANCE_NAME*: Dynamically defined
