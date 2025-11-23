-- Add down migration script here

-- This will remove all data from these tables.
-- This is intended for development and testing, be cautious in production.
DELETE FROM names;
DELETE FROM groups;
DELETE FROM sets;
DELETE FROM units;

-- Optional: Reset the autoincrement counter for SQLite.
DELETE FROM sqlite_sequence WHERE name IN ('names', 'groups', 'sets', 'units');