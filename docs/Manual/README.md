
# Table of Contents

1.  [Why pop-test?](#orgb10048b)
2.  [⚙️ A simple test, using the DSL](#orge97a419)
3.  [🖥️ How to run it](#org38a4941)
4.  [🐒 Motivation](#orgf73ff0a)
5.  [🈴  The Scenario DSL (Domain Specific Language)](#org507c1bd)
    1.  [Condition DSL](#org994d41c)
    2.  [Row DSL](#org6988082)
        1.  [Quantifiers](#org6c54f2c)
        2.  [Conditions](#org6191feb)
6.  [pop-test primitives](#org89f226e)
    1.  [pop services](#org1bd3684)
        1.  [service health check](#org333fff2)
        2.  [service postgres](#orgede1754)
        3.  [service kafka](#org6f24a91)
        4.  [service keycloak](#orga7c9cff)
        5.  [service pop<sub>server</sub>](#orgf79afee)
    2.  [pop actions](#org3c75ccb)
        1.  [a kafka produce action](#orga8ca44b)
        2.  [an http assertion](#orgfcc5a7d)
        3.  [an http action](#org285fd70)
        4.  [a kafka check](#orgdd40dfa)
7.  [Orchestrated services, also 3rd party](#orgabe0f64)
    1.  [🌐 pop postgres](#orgd25e951)
    2.  [🌐 pop kafka](#org0d8ea78)
    3.  [🧞 pop-server](#orgcc1deac)
    4.  [🗝️ pop keycloak](#org3d95007)
8.  [Why pop-test can be a Game-Changer](#org09afcfb)
    1.  [💚 Engineering Freedom and Confidence](#org7122b1b)
    2.  [🦸 Faster Development and Easier Maintenance](#org8654e70)
    3.  [💲 Business Value and Risk Reduction](#org91206cb)
9.  [🎶  Conclusion](#orgb70d275)

pop-test - *your test orchestration master of puppets*

pop-test is a tool that helps test computer programs, in integration, e2e, black-box, and other ways, in an easy and future-proof manner that improves reliability and engineering freedom.

In Dutch, *pop* means puppet or doll. 

pop-test simplifies end-to-end testing by providing a powerful orchestration tool and expressive language for your applications and their dependencies.

Define your test scenarios in a DSL (represented in YAML, TOML or JSON) and automate the spinning up (and down) of complex environments involving multiple programs, databases, and services, executing comprehensive tests with ease.

pop-test is built with Rust, enhancing safety, performance and reliability, with a modern and accessible code-base.

pop-tests as we affectionately call them, can range from small scope integration tests to all encompassing end-to-end or black box tests.

![img](../../resources/img/wp10992149-modified.png)

---


<a id="orgb10048b"></a>

# Why pop-test?

-   Frees you and allows you to refactor and test your systems more easily. You can for example completely rewrite a program, even in a different language, and stay certain your implementation works as expected.
-   Catches mistakes early – Helps find problems before real users do since you test with real dependencies.
-   Test coverage with a real meaning - test your systems from head to toe
-   Saves time – No need to test every little part manually, test on higher level.
-   Easy to use – Just write simple rules, in a comfy DSL and `pop-test` does the rest!
-   Fast and reliable – Built with Rust, a programming language known for speed and safety.

---


<a id="orge97a419"></a>

# ⚙️ A simple test, using the DSL

For example tests and learning demos, refer to the scenarios directory: <../../resources/scenarios>

Here follows a trivial example of pop-tests.

In this example we spin up a REST API written in Scala, with SBT, and poll its `/health` endpoint for 120 seconds or until healthy.

If the service is deemed healthy, then we perform some checks on the response of certain HTTP endpoints. These checks are performed using a `jql` inspired query format, and this allows us to check deeply nested attributes of the JSON response.

Reminder: the tool supports YAML, TOML and JSON, but YAML is as of now the preferred format.

More checks are possible (and encouraged), including PostgreSQL rows existence, Kafka offsets and more. For example's sake, we keep it simple:

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
          patience_seconds: 120
    
    pop_actions:
      - name: "Check that the forecast endpoint returns a 200 with a correct data, with load and solar"
        assert_url: "/api/v1/forecast"
        service_id: "green-energy-mix"
        json_body_patterns:
          - pattern: '"meta""forecastDate"'
            condition: '!= ""'
          - pattern: '"data"[0]"load"'
            condition: "> 0"
          - pattern: '"data"[0]"solar"'
            condition: ">= 0"


<a id="org38a4941"></a>

# 🖥️ How to run it

It is recommended to run with `TESTCONTAINERS_COMMAND=keep` and `RUST_LOG=debug` as environment variables.

Also since pop-test will do the resource management for you, we don't need to rely on Ryuk or on testcontainers-rs to do the cleanup.

See the [Makefile](file:///home/joe/Ontwikkeling/Persoonlijk/pop-test/Makefile) for more examples of how to run it.

When installed and available as binary, simply run:

    TESTCONTAINERS_COMMAND=keep pop-test -p ./path/to/file.yaml

`-p` or `--path` (required, string) - path to file or directory that contains scenarios

`-h` or `--help` for help

`-v` or `--version` for version

This is how I usually run in development mode:

    TESTCONTAINERS_COMMAND=keep RUST_LOG=debug nix develop -c cargo run -- --path ./resources/scenarios


<a id="orgf73ff0a"></a>

# 🐒 Motivation

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


<a id="org507c1bd"></a>

# 🈴  The Scenario DSL (Domain Specific Language)

In order to provide an expressive and powerful way to describe test scenarios, a lot of care has gone into creating a good domain-specific language, which remains flexible but concise.

Here follows an explanation of every individual field and parameter possible, including default or optional ones, as well as some recommendations.

Unless otherwise stated, the YAML representation of the Scenario DSL will be used. This is the author's preferred file format, as it can be more readable for complex testing scenarios.

The tool supports YAML, TOML and JSON file formats seamlessly.

---


<a id="org994d41c"></a>

## Condition DSL

This DSL is present in JSON body checks and in SQL database checks, supported conditions include:

`non-empty-str` (ensures the field is not empty)

`empty-str` (ensures the field is empty)

`== x` (ensures the field is x)

`!= x` (ensures the field is not x)

`> 0` (ensures the field is greater than zero)

`>= 0` (ensures the field is non-negative)

`< 0` (ensures the field is smaller than zero)

`<= 0` (ensures the field is non-positive)

---


<a id="org6988082"></a>

## Row DSL

This is the DSL used to perform assertions on SQL databases. Currently PostgreSQL supports this.

    [0] ((non-empty-str)(>= 12)(== en_GB)(non-empty-str))


<a id="org6c54f2c"></a>

### Quantifiers

`[0]` or `[first]` means the check should be applied to the first row in the result set

`[all]` means the check should be applied to all the rows in the result set

`[last]` means the check should be applied to the last row in the result set

`[n]` with n being a positive integer, means the check should be applied to the nth row in the result set


<a id="org6191feb"></a>

### Conditions

The rest of the clause is a tuple-based variant of the condition DSL, in order of selection in a query.

---


<a id="org89f226e"></a>

# pop-test primitives

See <../../src/pop_model.rs> for more. Below follows a detailed, as up-to-date as possible overview of pop-test's own data models.


<a id="org1bd3684"></a>

## pop services

`pop_services` is a list of services to be started for the scenario.

A service is defined by:  

`id` (required, string) – A unique identifier for the service within the scenario. A common practice is to use a kebab-case version of the service name.  

`dir` (required, string) – The working directory (pwd) from which the service process should be started. This can be an absolute path or a path starting with `$HOME`, which will be expanded to an absolute path.  

`base_url` (required, string) – The URL where the service’s HTTP endpoint is reachable.  

`start_cmd_args` (required, list of strings) – The command(s) needed to start the service. These will be executed via a shell interpreter (default: `bash -c`). For example, if `make run` is specified, the system will actually execute `bash -c make run`.

`shell_path` (optional, string) - the path to the shell interpreter to use to run the service. This defaults to `bash`.  

`env_vars` (optional, map of string to string) – A set of environment variables to be defined for the service.  


<a id="org333fff2"></a>

### service health check

Defines how pop-test determines whether the service is healthy. Field is `health_check`, a required object.

Health checks can take different forms:

Dummy/Placebo health check:

`always_healthy` (boolean) – If set, forces the service to always be considered healthy (or never healthy).

HTTP health check:

`path` (required, string) – The HTTP route used to check the service’s health (e.g., `/health`).  

`patience_seconds` (required, integer) – The maximum time allowed for the service to become healthy. If the service is healthy before this deadline, the scenario setup continues without waiting.


<a id="orgede1754"></a>

### service postgres

Defines how PostgreSQL is used in the service. Field is `postgres`, an optional object.

`enabled` (optional, boolean) - Whether to inject environment variables with PostgreSQL details

`host_var` (optional, string) - Container hostname will be injected on services, via environment variable, by default `POSTGRESQL_HOST`

`port_var` (optional, string) - Container DB port will be injected on services, via environment variable, by default ~POSTGRESQL<sub>PORT</sub>

`db_var` (optional, string) - Container DB schema name will be injected on services, via environment variable, by default `POSTGRESQL_DB`

`user_var` (optional, string) - Container DB username will be injected on services, via environment variable, by default `POSTGRESQL_USER`

`pass_var` (optional, string) - Container DB password will be injected on services, via environment variable, by default `POSTGRESQL_PASSWORD`


<a id="org6f24a91"></a>

### service kafka

Defines how Kafka is used in the service. Field is `kafka`, an optional object.

`enabled` (optional, boolean) - Whether to inject environment variables with Kafka details

`bootstrap_server_var` (optional, string) - Container HTTP connection string for Kafka will be injected on services, via environment variable, by default `KAFKA_BOOTSTRAP_SERVER`


<a id="orga7c9cff"></a>

### service keycloak

`enabled` (optional, boolean) - Whether to inject environment variables with Kafka details

`url_var` (optional, string) - Container HTTP connection string for Keycloak will be injected on services, via environment variable, by default `KEYCLOAK_AUTH_SERVER_URL`

`realm_var` (optional, string) - Container realm name string for Keycloak will be injected on services, via environment variable, by default `KEYCLOAK_REALM`


<a id="orgf79afee"></a>

### service pop<sub>server</sub>

`enabled` (optional, boolean) - Whether to inject environment variables with Kafka details

`url_vars` (optional, list of strings) - List of environment variable names that will be injected to the service, with pop<sub>server</sub>'s URL

---


<a id="org3c75ccb"></a>

## pop actions

`pop_actions` is a list of actions that setup state for a test and validate expected responses and behaviours from the services in the scenario. These actions help verify that the services return valid and expected data before proceeding with the scenario execution.

there are many types of pop action, see the following sections.

---


<a id="orga8ca44b"></a>

### a kafka produce action

A kafka produce action is defined by:

`message_file_path` (optional, string)

`kafka_topic` (required, string)

---


<a id="orgfcc5a7d"></a>

### an http assertion

`expected_code` (optional, u16 number) - expected HTTP status code, defaults to 200

`assert_url` (required, string) – The endpoint path to be tested within the service. This should be a relative path, such as `/api/v1/forecast`, which will be appended to the base<sub>url</sub> of the service.

`service_id` (required, string) – A reference to the id of the service being checked. This links the check to a specific service defined in `pop_services`.

`json_body_patterns` (optional, list of objects) – A list of conditions that must be met within the JSON response body. Each pattern defines a JSON field to inspect and a condition that must be satisfied.

Each JSON body pattern entry includes:

`pattern` (required, string) – A JSON path-like expression that identifies the field in the response body. This uses the Rust `jql` library.

`condition` (required, string) – A logical condition to be checked.


<a id="org285fd70"></a>

### an http action


<a id="orgdd40dfa"></a>

### a kafka check

---


<a id="orgabe0f64"></a>

# Orchestrated services, also 3rd party


<a id="orgd25e951"></a>

## 🌐 pop postgres

You can use PostgreSQL in pop-tests easily.

`enabled` (optional, boolean) - Whether to enable PostgreSQL in a container or not

`image_name` (optional, string) - Container image name for PostgreSQL, by default `postgres`

`image_tag` (optional, string) - Container image tag for PostgreSQL, by default `17.2` or newer

---


<a id="org0d8ea78"></a>

## 🌐 pop kafka

You can use Kafka in pop-tests easily.

`enabled` (optional, boolean) - Whether to enable Kafka in a container or not

`image_name` (optional, string) - Container image for Kafka, by default `confluentinc/cp-kafka`

`image_tag` (optional, string) - Container image tag for Kafka, by default `6.1.1`

---


<a id="orgcc1deac"></a>

## 🧞 pop-server

pop-server is a web-server that helps you simulate behaviours and test integrations with external systems.

pop-server is a lightweight HTTP server that allows dynamic behavior registration and retrieval. It acts as a simple puppet server where you can define request-response behaviors dynamically via HTTP requests. This can be useful for testing, mocking APIs, or creating programmable server responses.

Find the project here: <https://codeberg.org/jjba23/pop-server>

You can use `pop_server` in pop-tests easily:

`enabled` (optional, boolean) - Whether to enable pop-server in a container or not

`image_name` (optional, string) - Container image for pop-server, by default `jjba23/pop-server`

`image_tag` (optional, string) - Container image tag for pop-server, by default `0.3.0`

You can then  have it prepared with certain behaviors (under `behaviors` ):

`path` (required, string) - what URL path should be called to trigger behavior

`method` (required, string) - what HTTP method should be used to trigger behavior

`status` (required, string) - what HTTP status should be returned

`content_type` (required, string) - what content type should be returned as HTTP header

`body_file` (required, string) - path to the file to load data from, that will be returned as HTTP body


<a id="org3d95007"></a>

## 🗝️ pop keycloak

You can use Keycloak in pop-tests easily and have it auto-setup.

`enabled` (optional, boolean) - Whether to enable Keycloak in a container or not

`image_name` (optional, string) - Container image for Keycloak, by default `keycloak/keycloak`

`image_tag` (optional, string) - Container image tag for Keycloak, by default `26.1`

`realm` (required, string) – Name of the Keycloak realm to be created.

`client_id` (required, string) – OAuth ID of the Keycloak client that will be created for service authentication.

`client_secret` (optional, string) – OAuth secret  of the Keycloak client that will be created for service authentication.

`email`  (optional, string) – Email of the Keycloak user that will be created for service authentication.

`username` (required, string) - Username of the Keycloak client that will be created for service authentication.

`password` (required, string) - Password  of the Keycloak client that will be created for service authentication.

`roles` (optional, list of strings) - Names of roles to be created realm-wide and associated to the user

`scopes` (optional, list of strings) - Names of scopes to be created realm-wide and associated to the OAuth client

---

---


<a id="org09afcfb"></a>

# Why pop-test can be a Game-Changer

Software testing is an essential part of modern engineering, and pop-test takes it to the next level by making end-to-end testing simple, powerful, and reliable.

Whether you’re refactoring legacy code, maintaining a complex system, or ensuring business continuity, pop-test provides the confidence and efficiency your team needs.


<a id="org7122b1b"></a>

## 💚 Engineering Freedom and Confidence

pop-test enables fearless refactoring.

Engineers can completely rewrite a system—even in a different way, or language, without second-guessing whether it still functions correctly.

With pop-test validating the expected behaviors, teams can iterate quickly and improve systems without introducing regressions.

In my opinion mocks sometimes deliver few or even fake guarantees, are often more of a pain to maintain, and some implementations are outright broken and illogical&#x2026;

pop-tests verify the entire system as a whole ✅.

By testing with real dependencies—databases, APIs, message queues, pop-test ensures that your application works as expected in real-world conditions.

This approach leads to fewer surprises in production and more stable software releases.


<a id="org8654e70"></a>

## 🦸 Faster Development and Easier Maintenance

Maintaining a large codebase is much easier with pop-test in place. As systems evolve, tests remain valid, preventing accidental breakages and reducing the need for tedious manual verification.

Automating these processes saves valuable engineering time, allowing teams to focus on building new features instead of debugging regressions.


<a id="org91206cb"></a>

## 💲 Business Value and Risk Reduction

For businesses, software stability is critical. Bugs in production can lead to costly downtime, reputational damage, and loss of user trust.

pop-test mitigates these risks by catching issues early, before they impact customers.

With predictable and repeatable test automation, teams can ship faster with greater assurance, leading to a more agile and resilient business.

---


<a id="orgb70d275"></a>

# 🎶  Conclusion

pop-test is designed for modern development teams who need robust testing without complexity. Its expressive DSL, seamless service orchestration, and extensibility make it a long-term solution for ensuring software quality across any technology stack.

With pop-test, you gain more than just a testing tool—you gain a strategic advantage in software development. Engineering teams become more productive, businesses gain confidence in their products, and maintaining large systems becomes easier than ever.

pop-test is not just about testing—it’s about enabling better engineering.

