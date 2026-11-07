# SeaweedFS CLI · weed shell

````text
Usage: weed shell

  run interactive administrative commands.

	Generate shell.toml via "weed scaffold -config=shell"


Default Parameters:
  -cluster string
    	cluster defined in shell.toml
  -connectTimeout duration
    	timeout for the initial master connection and filer discovery (default 30s)
  -debug
    	print informational logs to stderr
  -filer string
    	filer host and port for initial connection, e.g. localhost:8888
  -filerGroup string
    	filerGroup for the filers
  -master string
    	comma-separated master servers, e.g. localhost:9333
````
