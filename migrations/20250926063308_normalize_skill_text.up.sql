-- Add up migration script here
-- Create a table for unique skill texts
CREATE TABLE skills (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text TEXT NOT NULL UNIQUE
);

-- Create a junction table for the many-to-many relationship between cards and skills
CREATE TABLE card_skills (
    card_id INTEGER NOT NULL,
    skill_id INTEGER NOT NULL,
    PRIMARY KEY (card_id, skill_id),
    FOREIGN KEY(card_id) REFERENCES cards(id),
    FOREIGN KEY(skill_id) REFERENCES skills(id)
);

-- Recreate the cards table without the info_text column.
-- SQLite requires this multi-step process to drop a column.
CREATE TABLE cards_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    series_code TEXT NOT NULL,
    set_code TEXT NOT NULL,
    number_in_set TEXT NOT NULL,
    name TEXT NOT NULL,
    card_type TEXT NOT NULL CHECK(card_type IN ('Character', 'Live', 'Energy')),
    UNIQUE(series_code, set_code, number_in_set)
);

INSERT INTO cards_new (id, series_code, set_code, number_in_set, name, card_type)
SELECT id, series_code, set_code, number_in_set, name, card_type FROM cards;

DROP TABLE cards;

ALTER TABLE cards_new RENAME TO cards;
