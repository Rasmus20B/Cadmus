
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE
  user = 'user' OR datname = 'cadmus_db';
DROP DATABASE IF EXISTS cadmus_db;

CREATE DATABASE cadmus_db OWNER 'user' ENCODING = 'UTF-8';
