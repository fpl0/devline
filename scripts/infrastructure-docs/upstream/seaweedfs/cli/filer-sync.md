# SeaweedFS CLI · weed filer.sync

````text
Usage: weed filer.sync -a=<oneFilerHost>:<oneFilerPort> -b=<otherFilerHost>:<otherFilerPort>

  resumable continuous synchronization for file changes between two active-active or active-passive filers

	filer.sync listens on filer notifications. If any file is updated, it will fetch the updated content,
	and write to the other destination. Different from filer.replicate:

	* filer.sync only works between two filers.
	* filer.sync does not need any special message queue setup.
	* filer.sync supports both active-active and active-passive modes.
	
	If restarted, the synchronization will resume from the previous checkpoints, persisted every minute.
	A fresh sync will start from the earliest metadata logs.


Default Parameters:
  -a string
    	filer A in one SeaweedFS cluster
  -a.collection string
    	collection on filer A
  -a.debug
    	debug mode to print out filer A received files
  -a.disk string
    	[hdd|ssd|<tag>] hard drive or solid state drive or any tag on filer A
  -a.doDeleteFiles
    	delete and update files when synchronizing on filer A (default true)
  -a.excludePaths string
    	exclude directories to sync on filer A
  -a.filerProxy
    	read and write file chunks by filer A instead of volume servers
  -a.fromTsMs int
    	synchronization from timestamp on filer A. The unit is millisecond
  -a.path string
    	directory to sync on filer A (default "/")
  -a.replication string
    	replication on filer A
  -a.security string
    	security.toml file for filer A when clusters use different certificates
  -a.ttlSec int
    	ttl in seconds on filer A
  -b string
    	filer B in the other SeaweedFS cluster
  -b.collection string
    	collection on filer B
  -b.debug
    	debug mode to print out filer B received files
  -b.disk string
    	[hdd|ssd|<tag>] hard drive or solid state drive or any tag on filer B
  -b.doDeleteFiles
    	delete and update files when synchronizing on filer B (default true)
  -b.excludePaths string
    	exclude directories to sync on filer B
  -b.filerProxy
    	read and write file chunks by filer B instead of volume servers
  -b.fromTsMs int
    	synchronization from timestamp on filer B. The unit is millisecond
  -b.path string
    	directory to sync on filer B (default "/")
  -b.replication string
    	replication on filer B
  -b.security string
    	security.toml file for filer B when clusters use different certificates
  -b.ttlSec int
    	ttl in seconds on filer B
  -chunkConcurrency int
    	The maximum number of chunks that will be replicated concurrently per file. (default 32)
  -concurrency int
    	The maximum number of files that will be synced concurrently. (default 32)
  -cpuprofile string
    	cpu profile output file
  -debug
    	serves runtime profiling data via pprof on the port specified by -debug.port
  -debug.port int
    	http port for debugging (default 6060)
  -isActivePassive
    	one directional follow from A to B if true
  -memprofile string
    	memory profile output file
  -metricsIp string
    	metrics listen ip
  -metricsPort int
    	metrics listen port
````
