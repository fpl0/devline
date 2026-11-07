# SeaweedFS CLI · weed backup

````text
Usage: weed backup -dir=. -volumeId=234 -master=localhost:9333

  Incrementally backup volume data.

	It is expected that you use this inside a script, to loop through
	all possible volume ids that needs to be backup to local folder.

	The volume id does not need to exist locally or even remotely.
	This will help to backup future new volumes.

	Usually backing up is just copying the .dat (and .idx) files.
	But it's tricky to incrementally copy the differences.

	The complexity comes when there are multiple addition, deletion and compaction.
	This tool will handle them correctly and efficiently, avoiding unnecessary data transportation.
  
Default Parameters:
  -collection string
    	collection name
  -dir string
    	directory to store volume data files (default ".")
  -master string
    	SeaweedFS master location (default "localhost:9333")
  -replication string
    	backup volume's replication, default is the same with origin
  -server string
    	SeaweedFS master location (deprecated, use -master instead)
  -ttl string
    	backup volume's time to live, format: 
				3m: 3 minutes
				4h: 4 hours
				5d: 5 days
				6w: 6 weeks
				7M: 7 months
				8y: 8 years
				default is the same with origin
  -volumeId int
    	a volume id. The volume .dat and .idx files should already exist in the dir. (default -1)
````
