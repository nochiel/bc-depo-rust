## Installation

```bash
$ brew install mariadb
$ brew services start mariadb
$ sudo $(brew --prefix mariadb)/bin/mariadb-admin -u root password NEWPASSWORD
```

For no password (development only!):

```bash
sudo $(brew --prefix mariadb)/bin/mariadb-admin -u root password ''
```
