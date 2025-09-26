-- Add up migration script here
-- Create a table to map rarity codes to their type (Regular/Parallel)
CREATE TABLE rarities (
    rarity_code TEXT PRIMARY KEY,
    rarity_type TEXT NOT NULL CHECK(rarity_type IN ('Regular', 'Parallel'))
);

-- Populate with known parallel rarities.
-- Any rarity code NOT in this table will be considered 'Regular' by the application.
INSERT INTO rarities (rarity_code, rarity_type) VALUES
    ('P',   'Parallel'),
    ('LLE', 'Parallel');
