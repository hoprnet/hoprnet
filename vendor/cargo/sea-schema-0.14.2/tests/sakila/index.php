<?php

require_once 'vendor/autoload.php';

$connectionParams = array(
    'url' => 'mysql://sea:sea@localhost/sakila',
    // 'url' => 'postgres://sea:sea@localhost/sakila',
);
$conn = \Doctrine\DBAL\DriverManager::getConnection($connectionParams);

$sm = $conn->getSchemaManager();

$sm->getDatabasePlatform()->registerDoctrineTypeMapping('geometry', 'string');
$sm->getDatabasePlatform()->registerDoctrineTypeMapping('enum', 'string');
$sm->getDatabasePlatform()->registerDoctrineTypeMapping('mpaa_rating', 'string');
$sm->getDatabasePlatform()->registerDoctrineTypeMapping('_text', 'string');

echo json_encode(getSchema($sm), JSON_PRETTY_PRINT)."\n";

function getSchema($sm)
{
    return [
        'tables' => array_map('getTable', $sm->listTables()),
    ];
}

function getTable($table)
{
    return [
        'name' => $table->getName(),
        'columns' => array_map('getColumn', array_values($table->getColumns())),
        'indexes' => array_map('getIndex', array_values($table->getIndexes())),
        'foreignKeys' => array_map('getForeignKey', array_values($table->getForeignKeys())),
    ];
}

function getColumn($column)
{
    return [
        'name' => $column->getName(),
        'type' => $column->getType()->getName(),
        'notNull' => $column->getNotNull(),
        'default' => $column->getDefault(),
        'length' => $column->getLength(),
        'fixed' => $column->getFixed(),
        'precision' => $column->getPrecision(),
        'scale' => $column->getScale(),
        'unsigned' => $column->getUnsigned(),
        'platformOptions' => $column->getPlatformOptions(),
        'autoincrement' => $column->getAutoincrement(),
        'definition' => $column->getColumnDefinition(),
        'comment' => $column->getComment(),
    ];
}

function getIndex($index)
{
    return [
        'name' => $index->getName(),
        'columns' => $index->getColumns(),
        'isUnique' => $index->isUnique(),
        'isPrimary' => $index->isPrimary(),
        'flags' => $index->getFlags(),
        'options' => $index->getOptions(),
    ];
}

function getForeignKey($key)
{
    return [
        'name' => $key->getName(),
        'localTable' => $key->getLocalTableName(),
        'foreignTable' => $key->getForeignTableName(),
        'localColumns' => $key->getLocalColumns(),
        'foreignColumns' => $key->getForeignColumns(),
        'onUpdate' => $key->onUpdate(),
        'onDelete' => $key->onDelete(),
        'options' => removeKeys($key->getOptions(), ['onUpdate', 'onDelete']),
    ];
}

function removeKeys($arr, $keys)
{
    foreach ($keys as $key) {
        unset($arr[$key]);
    }
    return $arr;
}