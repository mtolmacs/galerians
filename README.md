# Galerians
A galera cluster node auto-discovery and dynamic auto-updater tool. The tool was created to support deployment of multi-master mariadb clusters on Kubernetes running in a sidecar to mariadb in the same Pod.

# Usage
Note: Due to external program dependency ('hostname') this currenly only runs on Linux.

## Running directly in command line
`galerians [OPTIONS] --domain <domain> <--connection <connection>|--file <file>>`

Domain is the domain name which always resolves to the IP address of active nodes. In Kubernetes this could be a headless service backed by a statful set.

The connection string to the mysql or mariadb node is either given in the command line, or preferably read from a file, which is a mounted Kubernetes secret.

# Licence
MIT
