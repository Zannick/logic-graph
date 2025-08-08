To install with MySQL support, you will need:

- mysql
- libmysqlclient-dev
- diesel-cli (via cargo)

In MySQL, you will need to create a user `logic_graph` and grant appropriate permissions to it. As root:

```sql
CREATE USER 'logic_graph'@'localhost';
GRANT ALL ON `logic\_graph\_%`.* TO 'logic_graph'@'localhost';
```

You can use roles but they will have to be set as default roles for diesel-cli.

If you wish to store your data in a directory other than the default for your MySQL installation (such as on a different type of storage device), you must grant the `FILE` permission to the user:

```sql
GRANT FILE ON *.* TO 'logic_graph'@'localhost';
```

and add the directory to your MySQL config file. For example, if I'm running in Linux and want to store the data on a mounted device `/mnt/e`, I would add this to my `/etc/mysql/mysql.conf` file:

```conf
[mysql]
innodb_directories=/mnt/e/.mysql
```

Before you run with mysql enabled in a game's subdirectory, first run:

```
$ diesel setup
$ diesel migration run
Running migration 1_create
$
```

This must be done once for each game you wish to run against.

For unittests, in lieu of a migration, simply create the db once:

```sql
CREATE DB logic_graph__unittest;
```
