# SeaweedFS CLI · weed scaffold

````text
Usage: weed scaffold -config=[filer|notification|replication|security|master|volume|shell|credential|admin]

  Generate configuration files with all possible configurations for you to customize.

	The options can also be overwritten by environment variables.
	For example, the filer.toml mysql password can be overwritten by environment variable
		export WEED_MYSQL_PASSWORD=some_password
	Environment variable rules:
		* Prefix the variable name with "WEED_".
		* Uppercase the rest of the variable name.
		* Replace '.' with '_'.

  
Default Parameters:
  -config string
    	[filer|notification|replication|security|master|volume|shell|credential|admin] the configuration file to generate (default "filer")
  -output string
    	if not empty, save the configuration file to this directory
````
