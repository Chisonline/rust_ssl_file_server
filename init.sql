CREATE TABLE `file_block` (
  `id` int NOT NULL AUTO_INCREMENT COMMENT 'id',
  `file_id` int NOT NULL COMMENT 'file_info id',
  `block_id` bigint NOT NULL COMMENT '块id',
  `block_checksum` int unsigned NOT NULL COMMENT '块checksum',
  `block_size` int unsigned NOT NULL COMMENT '块体积Bytes',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'CreatedAt',
  `block_name` varchar(255) NOT NULL COMMENT '块文件名',
  PRIMARY KEY (`id`),
  KEY `idx_file_id_block_id` (`file_id`,`block_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='块元数据';

CREATE TABLE `file_info` (
  `id` int NOT NULL AUTO_INCREMENT COMMENT 'id',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'CreatedAt',
  `file_name` varchar(255) NOT NULL COMMENT '文件名',
  `file_checksum` int unsigned NOT NULL COMMENT '文件描述',
  `file_size` bigint NOT NULL COMMENT '文件体积Bytes',
  `file_status` int NOT NULL COMMENT '0:未完成,1:已完成,2:已删除',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='文件元数据';

CREATE TABLE `user` (
  `id` int NOT NULL AUTO_INCREMENT COMMENT 'id',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'CreatedAt',
  `user_name` varchar(255) NOT NULL COMMENT '用户名',
  `user_password` varchar(255) NOT NULL COMMENT '用户密码',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='用户表';