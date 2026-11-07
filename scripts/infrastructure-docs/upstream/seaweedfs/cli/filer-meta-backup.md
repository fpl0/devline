# SeaweedFS CLI · weed filer.meta.backup

````text
Usage: weed filer.meta.backup [-filer=localhost:8888] [-filerDir=/] [-includePrefixes=...] [-excludePrefixes=...] [-restart] -config=/path/to/backup_filer.toml

  continuously backup filer meta data changes.
The backup writes to another filer store specified in a backup_filer.toml.

	weed filer.meta.backup -config=/path/to/backup_filer.toml -filer="localhost:8888"
	weed filer.meta.backup -config=/path/to/backup_filer.toml -filer="localhost:8888" -restart

The -includePrefixes and -excludePrefixes flags accept comma-separated path prefixes.
Paths must be absolute (start with '/'). Matching is at directory boundaries.
When both match, the deeper prefix wins.
  
Default Parameters:
  -config string
    	path to filer.toml specifying backup filer store
  -excludePrefixes string
    	comma-separated path prefixes to exclude from backup
  -filer string
    	filer hostname:port (default "localhost:8888")
  -filerDir string
    	a folder on the filer (default "/")
  -includePrefixes string
    	comma-separated path prefixes to include in backup (if set, only these paths are backed up)
  -restart
    	copy the full metadata before async incremental backup
````
