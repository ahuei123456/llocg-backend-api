-- Add up migration script here

-- 1. Create a new table for unique canonical names.
CREATE TABLE names (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

-- 2. Recreate the cards table to use a name_id foreign key.
-- This is a destructive change and will drop all existing card data.
CREATE TABLE cards_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    series_code TEXT NOT NULL,
    set_code TEXT NOT NULL,
    number_in_set TEXT NOT NULL,
    name_id INTEGER NOT NULL,
    card_type TEXT NOT NULL CHECK(card_type IN ('Character', 'Live', 'Energy')),
    UNIQUE(series_code, set_code, number_in_set),
    FOREIGN KEY(name_id) REFERENCES names(id)
);

DROP TABLE cards;

ALTER TABLE cards_new RENAME TO cards;