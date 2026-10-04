# http-zero
An HTTP server with zero dependencies. Built using Rust's standard library only. This is an educational project, so some comments explain Rust mechanics or general computing concepts beyond what might be considered appropriate for a production codebase. 

# Diagram
```mermaid
flowchart TD
    main["main.rs<br/>register routes, start server"] --> server

    subgraph server ["server/mod.rs"]
        Start["<b>Server::start</b><br/>localhost:port"] --> Listener["start_tcp_listener"]
        Listener -.->|"thread"| Dispatcher["<b>Dispatcher</b><br/>Dispatch requests to route handler<br/>Write response"]
    end
        Dispatcher --> request

    subgraph request ["request/mod.rs"]
        Request["<b>Request::from_stream</b><br/>Parse request"]
    end
    Request  --> router
    subgraph router ["router/mod.rs"]
        Router["<b>Router::dispatch</b><br/>Look up (method, path)"] -->|"route found"| Handler["RouteHandler"]
        Router -->|"no route"| NF["404 Not Found"]
    end

        Handler --> response
        NF --> response
    subgraph response ["response/mod.rs"]
        Response["<b>Response</b><br/>Handle response data transmission"]
    end
    Response -->|"to_bytes()"| Dispatcher
    ```
