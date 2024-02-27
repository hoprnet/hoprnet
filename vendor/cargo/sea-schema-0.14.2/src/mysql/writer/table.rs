use crate::mysql::def::TableDef;
use sea_query::{Alias, Iden, Table, TableCreateStatement};

impl TableDef {
    pub fn write(&self) -> TableCreateStatement {
        let mut table = Table::create();
        table.table(Alias::new(&self.info.name));
        for col in self.columns.iter() {
            table.col(&mut col.write());
        }
        table.engine(self.info.engine.to_string().as_str());
        table.character_set(self.info.char_set.to_string().as_str());
        table.collate(self.info.collation.to_string().as_str());
        for idx in self.indexes.iter() {
            table.index(&mut idx.write());
        }
        for key in self.foreign_keys.iter() {
            table.foreign_key(&mut key.write());
        }
        table
    }
}

#[cfg(test)]
mod tests {
    use crate::mysql::def::*;
    use sea_query::MysqlQueryBuilder;

    #[test]
    fn test_1() {
        assert_eq!(
            TableDef {
                info: TableInfo {
                    name: "actor".to_owned(),
                    engine: StorageEngine::InnoDb,
                    auto_increment: None,
                    char_set: CharSet::Utf8Mb4,
                    collation: Collation::Utf8Mb40900AiCi,
                    comment: "".to_owned(),
                },
                columns: vec![
                    ColumnInfo {
                        name: "actor_id".to_owned(),
                        col_type: ColumnType::SmallInt(
                            NumericAttr {
                                maximum: None,
                                decimal: None,
                                unsigned: Some(
                                    true,
                                ),
                                zero_fill: None,
                            },
                        ),
                        null: false,
                        key: ColumnKey::Primary,
                        default: None,
                        extra: ColumnExtra {
                            auto_increment: true,
                            on_update_current_timestamp: false,
                            generated: false,
                            default_generated: false,
                        },
                        expression: None,
                        comment: "Actor ID".to_owned(),
                    },
                    ColumnInfo {
                        name: "first_name".to_owned(),
                        col_type: ColumnType::Varchar(
                            StringAttr {
                                length: Some(
                                    45,
                                ),
                                charset: None,
                                collation: None,
                            },
                        ),
                        null: false,
                        key: ColumnKey::NotKey,
                        default: None,
                        extra: ColumnExtra {
                            auto_increment: false,
                            on_update_current_timestamp: false,
                            generated: false,
                            default_generated: false,
                        },
                        expression: None,
                        comment: "".to_owned(),
                    },
                    ColumnInfo {
                        name: "last_name".to_owned(),
                        col_type: ColumnType::Varchar(
                            StringAttr {
                                length: Some(
                                    45,
                                ),
                                charset: None,
                                collation: None,
                            },
                        ),
                        null: false,
                        key: ColumnKey::Multiple,
                        default: None,
                        extra: ColumnExtra {
                            auto_increment: false,
                            on_update_current_timestamp: false,
                            generated: false,
                            default_generated: false,
                        },
                        expression: None,
                        comment: "".to_owned(),
                    },
                    ColumnInfo {
                        name: "last_update".to_owned(),
                        col_type: ColumnType::Timestamp(
                            TimeAttr {
                                fractional: None,
                            },
                        ),
                        null: false,
                        key: ColumnKey::NotKey,
                        default: Some(
                            ColumnDefault::CurrentTimestamp,
                        ),
                        extra: ColumnExtra {
                            auto_increment: false,
                            on_update_current_timestamp: true,
                            generated: false,
                            default_generated: true,
                        },
                        expression: None,
                        comment: "".to_owned(),
                    },
                ],
                indexes: vec![],
                foreign_keys: vec![],
            }.write().to_string(MysqlQueryBuilder),
            [
                "CREATE TABLE `actor` (",
                    "`actor_id` smallint UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'Actor ID',",
                    "`first_name` varchar(45) NOT NULL,",
                    "`last_name` varchar(45) NOT NULL,",
                    "`last_update` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP",
                ")",
                "ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
            ].join(" ")
        );
    }

    #[test]
    fn test_2() {
        assert_eq!(
            TableDef {
                info: TableInfo {
                    name: "film_actor".to_owned(),
                    engine: StorageEngine::InnoDb,
                    auto_increment: None,
                    char_set: CharSet::Utf8Mb4,
                    collation: Collation::Utf8Mb40900AiCi,
                    comment: "".to_owned(),
                },
                columns: vec![
                    ColumnInfo {
                        name: "actor_id".to_owned(),
                        col_type: ColumnType::SmallInt(
                            NumericAttr {
                                maximum: None,
                                decimal: None,
                                unsigned: Some(
                                    true,
                                ),
                                zero_fill: None,
                            },
                        ),
                        null: false,
                        key: ColumnKey::Primary,
                        default: None,
                        extra: ColumnExtra {
                            auto_increment: false,
                            on_update_current_timestamp: false,
                            generated: false,
                            default_generated: false,
                        },
                        expression: None,
                        comment: "".to_owned(),
                    },
                    ColumnInfo {
                        name: "film_id".to_owned(),
                        col_type: ColumnType::SmallInt(
                            NumericAttr {
                                maximum: None,
                                decimal: None,
                                unsigned: Some(
                                    true,
                                ),
                                zero_fill: None,
                            },
                        ),
                        null: false,
                        key: ColumnKey::Primary,
                        default: None,
                        extra: ColumnExtra {
                            auto_increment: false,
                            on_update_current_timestamp: false,
                            generated: false,
                            default_generated: false,
                        },
                        expression: None,
                        comment: "".to_owned(),
                    },
                    ColumnInfo {
                        name: "last_update".to_owned(),
                        col_type: ColumnType::Timestamp(
                            TimeAttr {
                                fractional: None,
                            },
                        ),
                        null: false,
                        key: ColumnKey::NotKey,
                        default: Some(
                            ColumnDefault::CurrentTimestamp,
                        ),
                        extra: ColumnExtra {
                            auto_increment: false,
                            on_update_current_timestamp: true,
                            generated: false,
                            default_generated: true,
                        },
                        expression: None,
                        comment: "".to_owned(),
                    },
                ],
                indexes: vec![
                    IndexInfo {
                        unique: true,
                        name: "PRIMARY".to_owned(),
                        parts: vec![
                            IndexPart {
                                column: "actor_id".to_owned(),
                                order: IndexOrder::Ascending,
                                sub_part: None,
                            },
                            IndexPart {
                                column: "film_id".to_owned(),
                                order: IndexOrder::Ascending,
                                sub_part: None,
                            },
                        ],
                        nullable: false,
                        idx_type: IndexType::BTree,
                        comment: "".to_owned(),
                        functional: false,
                    },
                    IndexInfo {
                        unique: false,
                        name: "idx_fk_film_id".to_owned(),
                        parts: vec![
                            IndexPart {
                                column: "film_id".to_owned(),
                                order: IndexOrder::Ascending,
                                sub_part: None,
                            },
                        ],
                        nullable: false,
                        idx_type: IndexType::BTree,
                        comment: "".to_owned(),
                        functional: false,
                    },
                ],
                foreign_keys: vec![
                    ForeignKeyInfo {
                        name: "fk_film_actor_actor".to_owned(),
                        columns: vec![
                            "actor_id".to_owned(),
                        ],
                        referenced_table: "actor".to_owned(),
                        referenced_columns: vec![
                            "actor_id".to_owned(),
                        ],
                        on_delete: ForeignKeyAction::Restrict,
                        on_update: ForeignKeyAction::Cascade,
                    },
                    ForeignKeyInfo {
                        name: "fk_film_actor_film".to_owned(),
                        columns: vec![
                            "film_id".to_owned(),
                        ],
                        referenced_table: "film".to_owned(),
                        referenced_columns: vec![
                            "film_id".to_owned(),
                        ],
                        on_delete: ForeignKeyAction::Restrict,
                        on_update: ForeignKeyAction::Cascade,
                    },
                ],
            }.write().to_string(MysqlQueryBuilder),
            vec![
                "CREATE TABLE `film_actor` (",
                    "`actor_id` smallint UNSIGNED NOT NULL,",
                    "`film_id` smallint UNSIGNED NOT NULL,",
                    "`last_update` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,",
                    "PRIMARY KEY (`actor_id`, `film_id`),",
                    "KEY `idx_fk_film_id` (`film_id`),",
                    "CONSTRAINT `fk_film_actor_actor`",
                        "FOREIGN KEY (`actor_id`) REFERENCES `actor` (`actor_id`)",
                        "ON DELETE RESTRICT ON UPDATE CASCADE,",
                    "CONSTRAINT `fk_film_actor_film`",
                        "FOREIGN KEY (`film_id`) REFERENCES `film` (`film_id`)",
                        "ON DELETE RESTRICT ON UPDATE CASCADE",
                ")",
                "ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
            ].join(" ")
        );
    }

    #[test]
    fn test_3() {
        assert_eq!(
            TableDef {
                info: TableInfo {
                    name: "film_actor".to_owned(),
                    engine: StorageEngine::InnoDb,
                    auto_increment: None,
                    char_set: CharSet::Utf8Mb4,
                    collation: Collation::Utf8Mb40900AiCi,
                    comment: "".to_owned(),
                },
                columns: vec![],
                indexes: vec![IndexInfo {
                    unique: false,
                    name: "idx_fk_film_id".to_owned(),
                    parts: vec![IndexPart {
                        column: "film_id".to_owned(),
                        order: IndexOrder::Ascending,
                        sub_part: Some(32),
                    },],
                    nullable: false,
                    idx_type: IndexType::BTree,
                    comment: "".to_owned(),
                    functional: false,
                },],
                foreign_keys: vec![],
            }
            .write()
            .to_string(MysqlQueryBuilder),
            [
                "CREATE TABLE `film_actor` (",
                "KEY `idx_fk_film_id` (`film_id` (32))",
                ")",
                "ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
            ]
            .join(" ")
        );
    }

    #[test]
    fn test_4() {
        assert_eq!(
            TableDef {
                info: TableInfo {
                    name: "film_actor".to_owned(),
                    engine: StorageEngine::InnoDb,
                    auto_increment: None,
                    char_set: CharSet::Utf8Mb4,
                    collation: Collation::Utf8Mb40900AiCi,
                    comment: "".to_owned(),
                },
                columns: vec![],
                indexes: vec![IndexInfo {
                    unique: false,
                    name: "idx_fk_film_id".to_owned(),
                    parts: vec![IndexPart {
                        column: "film_id".to_owned(),
                        order: IndexOrder::Descending,
                        sub_part: None,
                    },],
                    nullable: false,
                    idx_type: IndexType::BTree,
                    comment: "".to_owned(),
                    functional: false,
                },],
                foreign_keys: vec![],
            }
            .write()
            .to_string(MysqlQueryBuilder),
            [
                "CREATE TABLE `film_actor` (",
                "KEY `idx_fk_film_id` (`film_id` DESC)",
                ")",
                "ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
            ]
            .join(" ")
        );
    }
}
