
# Table of Contents

1.  [Why pop-test?](#orga2510d4)
2.  [Motivation](#org7d552d7)
3.  [The Scenario DSL (Domain Specific Language)](#org83061fb)
    1.  [pop services](#orgb32ae2c)
        1.  [service health check](#orgb861efc)
        2.  [service postgres](#orgdf0cc1b)
        3.  [service kafka](#org61cff88)
    2.  [pop actions](#org56a20ba)
    3.  [a kafka produce action](#org6183146)
    4.  [an http assertion](#orgdd5e0c1)
    5.  [Condition DSL](#orga20d1ce)
    6.  [pop postgres](#org1adf59d)
    7.  [pop kafka](#org3008e43)
4.  [Example tests, using the DSL](#org0d48a4a)

In Dutch 🇳🇱, *pop* means puppet or doll. But pop-test is not about dolls! It’s a tool that helps test computer programs in an easy and future-proof manner that improves reliability and engineering freedom.

pop-test simplifies end-to-end testing by providing a powerful orchestration tool and expressive language for your applications and their dependencies.

Define your test scenarios in a DSL (YAML, in the future also TOML or JSON) and automate the spinning up (and down) of complex environments involving multiple programs, databases, and services, executing comprehensive tests with ease.

pop-test is built with Rust, enhancing safety, performance and reliability, with a modern and accessible code-base.

pop-tests as we affectionately call them, can range from small scope integration tests to all encompassing end-to-end or black box tests.

![img](../../resources/img/wp10992149.jpg)

---


<a id="orga2510d4"></a>

# Why pop-test?

-   Catches mistakes early – Helps find problems before real users do.
-   Test coverage with a real meaning - test your systems from head to toe
-   Frees you and allows you to refactor and test your systems more easily
-   Saves time – No need to test every little part manually, test on higher level.
-   Easy to use – Just write simple rules, in a comfy DSL and `pop-test` does the rest!
-   Fast and reliable – Built with Rust, a programming language known for speed and safety.


<a id="org7d552d7"></a>

# Motivation

Imagine you're building a toy robot, but you don't need to understand all the small parts inside the robot to know if it works. You just need to press a button and see if it performs the task correctly.

A big advantage of this is that you test based on the output, not the code itself or implementation.

Of course this doesn't mean you shouldn't add some unit/property-based tests when useful. But these shouldn't be the bulk of your tests.

It's great when you change or improve your code (refactor), that you don’t need to worry about breaking anything as long as the system gives the same results as before.

It's kind of like saying: "As long as pressing the button still works the same, everything’s fine."

These tests are a safety net for changes to systems (specially if you aren't too familiar with the system yet). It gives you confidence when making changes to the code.

How Does This "Free You"? You can change the internal code without fear because you already have pop-tests that check if everything works as expected. So, you can keep improving the system without worrying about accidentally breaking things.

This also allows for faster development, since you don’t have to check every single little part of the code manually.

As your system grows, you don’t need to rewrite tests every time. You just keep running your pop-tests, and you know that new features won’t break old ones.

In short, pop-tests make engineers happier, business people more certain of what they sell and overall, it makes refactoring and maintaining things easier.

---


<a id="org83061fb"></a>

# The Scenario DSL (Domain Specific Language)

In order to provide an expressive and powerful way to describe test scenarios, a lot of care has gone into creating a good domain-specific language, which remains flexible but concise.

Here follows an explanation of every individual field and parameter possible, including default or optional ones, as well as some recommendations. Unless otherwise stated, the YAML representation of the Scenario DSL will be used.

`pop_test` represents the top node under which all other nodes live.

---


<a id="orgb32ae2c"></a>

## pop services

`pop_services` is a list of services to be started for the scenario.

A service is defined by:  

`id` (required, string) – A unique identifier for the service within the scenario. A common practice is to use a kebab-case version of the service name.  

`dir` (required, string) – The working directory (pwd) from which the service process should be started. This can be an absolute path or a path starting with `$HOME`, which will be expanded to an absolute path.  

`base_url` (required, string) – The URL where the service’s HTTP endpoint is reachable.  

`start_cmd_args` (required, list of strings) – The command(s) needed to start the service. These will be executed via a shell interpreter (default: `bash -c`). For example, if `make run` is specified, the system will actually execute `bash -c make run`.

`shell_path` (optional, string) - the path to the shell interpreter to use to run the service. This defaults to `bash`.  

`env_vars` (optional, map of string to string) – A set of environment variables to be defined for the service.  


<a id="orgb861efc"></a>

### service health check

Defines how pop-test determines whether the service is healthy. Field is `health_check`, a required object.

Health checks can take different forms:

Dummy/Placebo health check:

`always_healthy` (boolean) – If set, forces the service to always be considered healthy (or never healthy).

HTTP health check:

`path` (required, string) – The HTTP route used to check the service’s health (e.g., `/health`).  

`patience_seconds` (required, integer) – The maximum time allowed for the service to become healthy. If the service is healthy before this deadline, the scenario setup continues without waiting.


<a id="orgdf0cc1b"></a>

### service postgres

Defines how PostgreSQL is used in the service. Field is `postgres`, an optional object.

`enabled` (optional, boolean) - Whether to inject environment variables with PostgreSQL details

`host_var` (optional, string) - Container hostname will be injected on services, via environment variable, by default `POSTGRESQL_HOST`

`port_var` (optional, string) - Container DB port will be injected on services, via environment variable, by default ~POSTGRESQL<sub>PORT</sub>

`db_var` (optional, string) - Container DB schema name will be injected on services, via environment variable, by default `POSTGRESQL_DB`

`user_var` (optional, string) - Container DB username will be injected on services, via environment variable, by default `POSTGRESQL_USER`

`pass_var` (optional, string) - Container DB password will be injected on services, via environment variable, by default `POSTGRESQL_PASSWORD`


<a id="org61cff88"></a>

### service kafka

Defines how Kafka is used in the service. Field is `kafka`, an optional object.

`enabled` (optional, boolean) - Whether to inject environment variables with Kafka details

`bootstrap_server_var` (optional, string) - Container HTTP connection string for Kafka will be injected on services, via environment variable, by default `KAFKA_BOOTSTRAP_SERVER`

---


<a id="org56a20ba"></a>

## pop actions

`pop_actions` is a list of actions that setup state for a test and validate expected responses and behaviours from the services in the scenario. These actions help verify that the services return valid and expected data before proceeding with the scenario execution.

there are many types of pop action, see the following sections.

---


<a id="org6183146"></a>

## a kafka produce action

A kafka produce action is defined by:

`json_from_file` (optional, string)

---


<a id="orgdd5e0c1"></a>

## an http assertion

`expected_code` (optional, u16 number) - expected HTTP status code

`check_url` (required, string) – The endpoint path to be tested within the service. This should be a relative path, such as `/api/v1/forecast`, which will be appended to the base<sub>url</sub> of the service.

`service_id` (required, string) – A reference to the id of the service being checked. This links the check to a specific service defined in `pop_services`.

`json_body_patterns` (optional, list of objects) – A list of conditions that must be met within the JSON response body. Each pattern defines a JSON field to inspect and a condition that must be satisfied.

Each JSON body pattern entry includes:

`pattern` (required, string) – A JSON path-like expression that identifies the field in the response body. This uses the Rust `jql` library.

`condition` (required, string) – A logical condition to be checked.

---


<a id="orga20d1ce"></a>

## Condition DSL

Supported conditions include:

`!= ""` (ensures the field is not empty)

`== ""` (ensures the field is empty)

`> 0` (ensures the field is greater than zero)

`>= 0` (ensures the field is non-negative)

`< 0` (ensures the field is smaller than zero)

`<= 0` (ensures the field is non-positive)

---


<a id="org1adf59d"></a>

## pop postgres

You can use PostgreSQL in pop-tests easily.

`enabled` (optional, boolean) - Whether to enable PostgreSQL in a container or not

`image_name` (optional, string) - Container image name for PostgreSQL, by default `postgres`

`image_tag` (optional, string) - Container image tag for PostgreSQL, by default `17.2` or newer

---


<a id="org3008e43"></a>

## pop kafka

You can use Kafka in pop-tests easily.

`enabled` (optional, boolean) - Whether to enable Kafka in a container or not

`image_name` (optional, string) - Container image for Kafka, by default `confluentinc/cp-kafka`

`image_tag` (optional, string) - Container image tag for Kafka, by default `6.1.1`

---


<a id="org0d48a4a"></a>

# Example tests, using the DSL

For example tests and learning demos, refer to the scenarios directory: <../../resources/scenarios>

Here follows also a trivial example of pop-tests.

In this example we spin up a REST API written in Scala, with SBT, and poll its `/health` endpoint for 90 seconds or until healthy. Then we perform some checks on the response of certain HTTP endpoints. These checks are performed using `jql` and this allows us to check deeply nested attributes of the JSON response.

    pop_test:
      pop_services:
        - id: "green-energy-mix"
          dir: "$HOME/Ontwikkeling/Werk"
          base_url: "http://localhost:50000"
          start_cmd_args:
            - 'sbt "greenEnergyMix/run"'
          env_vars:
            SOME_TOKEN: "0027809ce9"
          health_check:
            path: "/health"
            patience_seconds: 90
    
      pop_actions:
        - check_url: "/api/v1/forecast"
          service_id: "green-energy-mix"
          json_body_patterns:
            - pattern: '"meta""forecastDate"'
              condition: '!= ""'
            - pattern: '"data"[0]"load"'
              condition: "> 0"
            - pattern: '"data"[0]"solar"'
              condition: ">= 0"

