# mongod --help

```text
Options:
  --networkMessageCompressors arg (=snappy,zstd,zlib)
                                        Comma-separated list of compressors to 
                                        use for network messages

General options:
  -h [ --help ]                         Show this usage information
  --version                             Show version information
  -f [ --config ] arg                   Configuration file specifying 
                                        additional options
  --configExpand arg                    Process expansion directives in config 
                                        file (none, exec, rest)
  --port arg                            Specify port number - 27017 by default
  --ipv6                                Enable IPv6 support (disabled by 
                                        default)
  --listenBacklog arg                   Set socket listen backlog size
  --maxConns arg (=1000000)             Max number of simultaneous connections
  --pidfilepath arg                     Full path to pidfile (if not set, no 
                                        pidfile is created)
  --timeZoneInfo arg                    Full path to time zone info directory, 
                                        e.g. /usr/share/zoneinfo
  --nounixsocket                        Disable listening on unix sockets
  --proxyUnixSocketPrefix arg           The directory for proxy UNIX domain 
                                        sockets. If not configured or set to 
                                        empty, the proxy sockets are disabled.
  --proxyUnixSocketFileGroupId arg      The group id that will be assigned to 
                                        the proxy UNIX domain socket. If unset,
                                        the socket group defaults to the 
                                        server's effective group id."
  --unixSocketPrefix arg                Alternative directory for UNIX domain 
                                        sockets (defaults to /tmp)
  --filePermissions arg                 Permissions to set on UNIX domain 
                                        socket file - 0700 by default
  --fork                                Fork server process
  --secondaryPort arg                   Specify port number - 27017 by default
  -v [ --verbose ] [=arg(=v)]           Be more verbose (include multiple times
                                        for more verbosity e.g. -vvvvv)
  --quiet                               Quieter output
  --logpath arg                         Log file to send write to instead of 
                                        stdout - has to be a file, not 
                                        directory
  --syslog                              Log to system's syslog facility instead
                                        of file or stdout
  --syslogFacility arg                  syslog facility used for mongodb syslog
                                        message
  --logappend                           Append to logpath instead of 
                                        over-writing
  --logRotate arg                       Set the log rotation behavior 
                                        (rename|reopen)
  --timeStampFormat arg                 Desired format for timestamps in log 
                                        messages. One of iso8601-utc or 
                                        iso8601-local
  --setParameter arg                    Set a configurable parameter
  --bind_ip arg                         Comma separated list of ip addresses to
                                        listen on - localhost by default
  --bind_ip_all                         Bind to all ip addresses
  --noauth                              Run without security
  --transitionToAuth                    For rolling access control upgrade. 
                                        Attempt to authenticate over outgoing 
                                        connections and proceed regardless of 
                                        success. Accept incoming connections 
                                        with or without authentication.
  --slowms arg (=100)                   Value of slow for profile and console 
                                        log
  --slowOpSampleRate arg (=1)           Fraction of slow ops to include in the 
                                        profile and console log
  --profileFilter arg                   Query predicate to control which 
                                        operations are logged and profiled
  --auth                                Run with security
  --clusterIpSourceAllowlist arg        Network CIDR specification of permitted
                                        origin for `__system` access
  --profile arg                         0=off 1=slow, 2=all
  --cpu                                 Periodically show cpu and iowait 
                                        utilization
  --sysinfo                             Print some diagnostic system 
                                        information
  --noscripting                         Disable scripting engine
  --notablescan                         Do not allow table scans
  --shutdown                            Kill a running server (for init 
                                        scripts)
  --proxyPort arg                       The port that accepts connections with 
                                        a proxy protocol header. Access to this
                                        port MUST be restricted to trusted 
                                        hosts (the load balancer or reverse 
                                        proxy).
  --keyFile arg                         Private key for cluster authentication
  --clusterAuthMode arg                 Authentication mode used for cluster 
                                        authentication. Alternatives are 
                                        (keyFile|sendKeyFile|sendX509|x509)
  --upgradeBackCompat                   Indicates to the process that it is 
                                        being run as part of an upgrade 
                                        process. No features or commands which 
                                        would not be recognized by a previous 
                                        binary version are used.
  --downgradeBackCompat                 Indicates to the process that it is 
                                        being run as part of an downgrade 
                                        process. No features or commands which 
                                        would not be recognized by a previous 
                                        binary version are used.

Replication options:
  --oplogSize arg                       Size to use (in MB) for replication op 
                                        log. default is 5% of disk space (i.e. 
                                        large is good)

Replica set options:
  --replSet arg                         arg is <setname>[/<optionalseedhostlist
                                        >]
  --replSetName arg                     arg is <setname>
  --enableMajorityReadConcern [=arg(=1)] (=1)
                                        Enables majority readConcern. 
                                        enableMajorityReadConcern=false is no 
                                        longer supported

Serverless mode:
  --serverless arg                      Serverless mode implies replication is 
                                        enabled, cannot be used with replSet or
                                        replSetName.

Sharding options:
  --configsvr                           Assigns the config-server role to this 
                                        node in a sharded cluster. The default 
                                        listening port is 27019 and the default
                                        database directory is /data/configdb.
  --shardsvr                            Assigns the shard-server role to this 
                                        node in a sharded cluster. The default 
                                        listening port is 27018.
  --routerPort [=arg(=27016)]           Assigns the router role to this node in
                                        a sharded cluster, and results in 
                                        listening to a dedicated port (27016 by
                                        default) to serve routing requests.
  --maintenanceMode arg                 Allows this node to skip starting up 
                                        sharding components or both replication
                                        and  sharding components. This allows 
                                        for performing maintenance on this 
                                        node. To skip  setting up just sharding
                                        components use --maintenanceMode=replic
                                        aSet. To skip setting  up both sharding
                                        and replication components use 
                                        --maintenanceMode=standalone.

Storage options:
  --storageEngine arg                   What storage engine to use - defaults 
                                        to wiredTiger if no data files present
  --dbpath arg                          Directory for datafiles - defaults to 
                                        /data/db
  --directoryperdb                      Each database will be stored in a 
                                        separate directory
  --syncdelay arg                       Seconds between disk syncs
  --journalCommitInterval arg           how often to group/batch commit (ms)
  --upgrade                             Upgrade db if needed
  --repair                              Run repair on all dbs
  --restore                             This should only be used when restoring
                                        from a backup. Mongod will behave 
                                        differently by handling collections 
                                        with missing data files, allowing 
                                        database renames, skipping oplog 
                                        entries for collections not restored 
                                        and more.
  --oplogMinRetentionHours arg (=0)     Minimum number of hours to preserve in 
                                        the oplog. Default is 0 (turned off). 
                                        Fractions are allowed (e.g. 1.5 hours)

AWS IAM Options:
  --awsIamSessionToken arg              AWS Session Token for temporary 
                                        credentials

TLS Options:
  --tlsOnNormalPorts                    Use TLS on configured ports
  --tlsMode arg                         Set the TLS operation mode 
                                        (disabled|allowTLS|preferTLS|requireTLS
                                        )
  --tlsCertificateKeyFile arg           Certificate and key file for TLS. 
                                        Certificate is presented in response to
                                        inbound connections always. Certificate
                                        is also presented for outbound 
                                        connections if tlsClusterFile is not 
                                        specified.
  --tlsCertificateKeyFilePassword arg   Password to unlock key in the TLS 
                                        certificate key file
  --tlsClusterFile arg                  Certificate and key file for internal 
                                        TLS authentication. Certificate is 
                                        presented on outbound connections if 
                                        specified.
  --tlsClusterPassword arg              Internal authentication key file 
                                        password
  --tlsCAFile arg                       Certificate Authority file for TLS. 
                                        Used to verify remote certificates 
                                        presented in response to outbound 
                                        connections. Also used to verify remote
                                        certificates from inbound connections 
                                        if tlsClusterCAFile is not specified.
  --tlsClusterCAFile arg                CA used for verifying remotes during 
                                        inbound connections
  --tlsCRLFile arg                      Certificate Revocation List file for 
                                        TLS
  --tlsDisabledProtocols arg            Comma separated list of TLS protocols 
                                        to disable [TLS1_0,TLS1_1,TLS1_2,TLS1_3
                                        ]
  --tlsAllowConnectionsWithoutCertificates 
                                        Allow client to connect without 
                                        presenting a certificate
  --tlsAllowInvalidHostnames            Allow server certificates to provide 
                                        non-matching hostnames
  --tlsAllowInvalidCertificates         Allow connections to servers with 
                                        invalid certificates
  --tlsLogVersions arg                  Comma separated list of TLS protocols 
                                        to log on connect [TLS1_0,TLS1_1,TLS1_2
                                        ,TLS1_3]
  --tlsClusterAuthX509ExtensionValue arg
                                        If specified, clients who expect to be 
                                        regarded as cluster members must 
                                        present a valid X.509 certificate 
                                        containing an X.509 extension for OID 
                                        1.3.6.1.4.1.34601.2.1.2 which contains 
                                        the specified value.
  --tlsClusterAuthX509Attributes arg    If specified, clients performing X.509 
                                        authentication must present a 
                                        certificate with a  subject name with 
                                        the exact attributes and values 
                                        provided in this config option to be  
                                        treated as peer cluster nodes.

WiredTiger options:
  --wiredTigerCacheSizeGB arg           Maximum amount of memory to allocate 
                                        for cache in GB; Defaults to 1/2 of 
                                        physical RAM. Only one of either 
                                        wiredTigerCacheSizePct or 
                                        wiredTigerCacheSizeGB can be provided
  --wiredTigerCacheSizePct arg          Maximum amount of memory to allocate 
                                        for cache as a percentage of physical 
                                        RAM; Defaults to 1/2 of physical RAM 
                                        and a minimum of 256MB. Only one of 
                                        either wiredTigerCacheSizePct or 
                                        wiredTigerCacheSizeGB can be provided
  --zstdDefaultCompressionLevel arg (=6)
                                        Default compression level for zstandard
                                        compressor
  --wiredTigerJournalCompressor arg (=snappy)
                                        Use a compressor for log records 
                                        [none|snappy|zlib|zstd]
  --wiredTigerDirectoryForIndexes       Put indexes and data in different 
                                        directories
  --wiredTigerStatisticsSetting arg (=fast)
                                        Statistics collection mode 
                                        [all|cache_walk|fast|none|clear|tree_wa
                                        lk]
  --wiredTigerCollectionBlockCompressor arg (=snappy)
                                        Block compression algorithm for 
                                        collection data [none|snappy|zlib|zstd]
  --wiredTigerIndexPrefixCompression arg (=1)
                                        Use prefix compression on row-store 
                                        leaf pages
```
