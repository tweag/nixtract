## Envisioned Usage

nix run tweag/nix-graph-explorer#ui -- --nixtract-json example-data.json

The above CLI will then:

--> Launch DB (Runs init scripts, etc.)
--> Ingest JSON file
--> Launch UI with our local DB as the default connection


## Notes to self

* Currently I've only experimented with running graph-explorer locally using pnpm and have not tried building / running via Nix
* To launch janusgraph use the `janusgraph-server` command. This takes a single optional argument which is the path to the Janusgraph Server YAML config file (default=$PWD/conf/janusgraph-server.yaml)
