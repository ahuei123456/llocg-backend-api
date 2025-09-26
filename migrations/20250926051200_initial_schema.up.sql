-- Table for card sets
CREATE TABLE IF NOT EXISTS sets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    set_code TEXT NOT NULL UNIQUE, -- e.g., 'bp2', 'PR'
    name TEXT NOT NULL -- e.g., 'Booster Pack Vol. 2', 'Promo'
);

-- Table for groups (e.g., μ's, Aqours)
CREATE TABLE IF NOT EXISTS groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

-- Table for units (e.g., Printemps, CYaRon!)
CREATE TABLE IF NOT EXISTS units (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

-- Represents the core, unique game card, independent of rarity or printing.
-- Uniqueness is defined by series, set code, and number within the set.
CREATE TABLE IF NOT EXISTS cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    series_code TEXT NOT NULL, -- e.g., 'PL!S'
    set_code TEXT NOT NULL, -- e.g., 'bp2'
    number_in_set TEXT NOT NULL, -- e.g., '001'
    name TEXT NOT NULL,
    card_type TEXT NOT NULL CHECK(card_type IN ('Character', 'Live', 'Energy')),
    info_text TEXT, -- Skill text for Character and Live cards

    -- A unique card is identified by its series, set, and number
    UNIQUE(series_code, set_code, number_in_set)
);

-- Represents a specific printing of a card, including its rarity and image.
CREATE TABLE IF NOT EXISTS printings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    card_id INTEGER NOT NULL,
    rarity_code TEXT NOT NULL, -- e.g., 'R', 'P', 'LLE'
    rarity_type TEXT NOT NULL CHECK(rarity_type IN ('Regular', 'Parallel')),
    image_url TEXT,

    -- A printing is unique by its card and rarity code
    UNIQUE(card_id, rarity_code),
    FOREIGN KEY(card_id) REFERENCES cards(id)
);

-- Junction table for the many-to-many relationship between cards and groups.
CREATE TABLE IF NOT EXISTS card_groups (
    card_id INTEGER NOT NULL,
    group_id INTEGER NOT NULL,
    PRIMARY KEY (card_id, group_id),
    FOREIGN KEY(card_id) REFERENCES cards(id),
    FOREIGN KEY(group_id) REFERENCES groups(id)
);

-- Junction table for the many-to-many relationship between cards and units.
CREATE TABLE IF NOT EXISTS card_units (
    card_id INTEGER NOT NULL,
    unit_id INTEGER NOT NULL,
    PRIMARY KEY (card_id, unit_id),
    FOREIGN KEY(card_id) REFERENCES cards(id),
    FOREIGN KEY(unit_id) REFERENCES units(id)
);

-- Junction table to store the count of each heart color for a card.
CREATE TABLE IF NOT EXISTS card_hearts (
    card_id INTEGER NOT NULL,
    -- The seven possible heart colors
    color TEXT NOT NULL CHECK(color IN ('Pink', 'Red', 'Yellow', 'Green', 'Blue', 'Purple', 'Gray')),
    count INTEGER NOT NULL DEFAULT 1,

    -- Character cards should not have 'Gray' hearts. This must be enforced
    -- in application logic, as a CHECK constraint here cannot reference other tables.

    PRIMARY KEY (card_id, color),
    FOREIGN KEY(card_id) REFERENCES cards(id)
);

-- Table for properties specific to 'Character' cards.
CREATE TABLE IF NOT EXISTS character_cards (
    card_id INTEGER PRIMARY KEY,
    cost INTEGER NOT NULL,
    blades INTEGER NOT NULL,
    -- The six possible blade heart colors for a Character card, plus 'All'
    blade_heart TEXT CHECK(blade_heart IS NULL OR blade_heart IN ('Pink', 'Red', 'Yellow', 'Green', 'Blue', 'Purple', 'All')),

    FOREIGN KEY(card_id) REFERENCES cards(id)
);

-- Table for properties specific to 'Live' cards.
CREATE TABLE IF NOT EXISTS live_cards (
    card_id INTEGER PRIMARY KEY,
    score INTEGER NOT NULL,
    -- The six possible blade heart colors for a Live card, plus 'All'
    blade_heart TEXT CHECK(blade_heart IS NULL OR blade_heart IN ('Pink', 'Red', 'Yellow', 'Green', 'Blue', 'Purple', 'All')),
    -- The possible special heart types
    special_heart TEXT CHECK(special_heart IS NULL OR special_heart IN ('Draw', 'Score')),

    FOREIGN KEY(card_id) REFERENCES cards(id)
);

-- 'Energy' cards currently have no unique properties beyond the base card,
-- but a table is created for future-proofing and consistency.
CREATE TABLE IF NOT EXISTS energy_cards (
    card_id INTEGER PRIMARY KEY,

    FOREIGN KEY(card_id) REFERENCES cards(id)
);
-- Add up migration script here
