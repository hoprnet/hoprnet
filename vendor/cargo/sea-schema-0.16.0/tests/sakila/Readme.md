# Import

First import the the sakila dataset

# Install

First install necessary PHP extensions
```sh
sudo apt install php-mysql php-pgsql
```

Then install dependencies
```sh
composer install
```

# Run

```sh
php index.php > schema.json
```

# Dump

I can't seem to be able to reproduce the original .sql
The best I got is

```sh
mysqldump sakila --no-data --skip-opt --skip-quote-names --skip-set-charset
```