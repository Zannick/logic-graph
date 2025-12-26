# MySQL support

To install with MySQL support, you will need:

- mysql
- libmysqlclient-dev
- diesel-cli (via cargo)

Update the default parameters in your config file (e.g. `/etc/mysql/my.cnf` for Linux):

```conf
[mysqld]
cte_max_recursion_depth=100000
```

In MySQL, you will need to create a user `logic_graph` and grant appropriate permissions to it. As root:

```sql
CREATE USER 'logic_graph'@'localhost';
GRANT ALL ON `logic\_graph\_%`.* TO 'logic_graph'@'localhost';
GRANT SELECT ON `sys`.* TO 'logic_graph'@'localhost';
GRANT SELECT ON `performance_schema`.* TO 'logic_graph'@'localhost';
```

You can use roles but they will have to be set as default roles for diesel-cli.

## Changing data directory

If you wish to store your data in a directory other than the default for your MySQL installation (such as on a different type of storage device), you must grant the `FILE` permission to the user:

```sql
GRANT FILE ON *.* TO 'logic_graph'@'localhost';
```

and add the directory to your MySQL config file. For example, if I'm running in Linux and want to store the data on a mounted device `/mnt/e`, I would add this to my `/etc/mysql/my.cnf` file, under the same `[mysqld]` section as above:

```conf
[mysqld]
innodb_directories=/mnt/e/.mysql
```

Before you run with mysql enabled in a game's subdirectory, first run:

```
$ diesel setup
Creating database: logic_graph_axiom_verge2
Running migration 1_create
$
```

This must be done once for each game you wish to run against.

For unittests, in lieu of running a migration, simply create the db once:

```sql
CREATE DB logic_graph__unittest;
```

## Danger zone

As admin, you can run some commands to improve the mysql performance that affect the entire instance. Do not use if you have other uses of MySQL on your system.

```sql
ALTER INSTANCE DISABLE INNODB REDO_LOG;
```

In the case of a crash, the instance will not be recoverable, and you will have to clear your data directories and rerun initialize:

```
sudo -u mysql mysqld --defaults-file=/etc/mysql/my.cnf --initialize-insecure --init-file=init.sql --user=mysql --console
```
