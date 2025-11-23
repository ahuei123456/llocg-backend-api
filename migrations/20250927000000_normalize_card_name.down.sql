-- Add down migration script here

-- This is a destructive change.

DROP TABLE cards;
DROP TABLE names;

CREATE TABLE cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    series_code TEXT NOT NULL,
    set_code TEXT NOT NULL,
    number_in_set TEXT NOT NULL,
    name TEXT NOT NULL,
    card_type TEXT NOT NULL CHECK(card_type IN ('Character', 'Live', 'Energy')),
    UNIQUE(series_code, set_code, number_in_set)
);