DROP DATABASE IF EXISTS s_regexp;

CREATE DATABASE s_regexp;

USE s_regexp;

CREATE TABLE IF NOT EXISTS `regexps` (
  `key` CHAR(36) NOT NULL PRIMARY KEY,
  `regexp` TEXT NOT NULL,
  `user_id` CHAR(36) NOT NULL,
  `user_name` VARCHAR(32) NOT NULL,
  `created_at` DATETIME NOT NULL
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci;