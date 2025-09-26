-- Add down migration script here
-- This down migration is destructive if you have added skills.
-- It's primarily for development rollback.

DROP TABLE card_skills;
DROP TABLE skills;

ALTER TABLE cards ADD COLUMN info_text TEXT;
