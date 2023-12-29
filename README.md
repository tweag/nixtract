## Envisioned Usage

The basic idea of this project is to allow you to spin up a front-end for exploring any nixtract output.
To do so, it leverages an in-memory Gremlin database (powered by JanusGraph) as well as AWS's graph-explorer UI.

The idea is that you can invoke this tool on a given nixtract .jsonl file and it will automatically spin up a local graph db
with the data from the jsonl file and launch an explorer instance which connects to it.

nix run tweag/nix-graph-explorer#ui -- --nixtract-json example-data.json

The above CLI will then:

--> Launch DB (Runs init scripts, etc.)
--> Ingest JSON file
--> Launch UI with our local DB as the default connection

## Notes to self

- Currently I've only experimented with running graph-explorer locally using pnpm and have not tried building / running via Nix
- To launch janusgraph use the `janusgraph-server` command. This takes a single optional argument which is the path to the Janusgraph Server YAML config file (default=$PWD/conf/janusgraph-server.yaml)
