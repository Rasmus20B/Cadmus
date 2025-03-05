-- Add migration script here
ALTER TABLE projects 
RENAME COLUMN metadta TO metadata;
