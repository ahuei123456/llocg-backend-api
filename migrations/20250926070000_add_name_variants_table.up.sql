-- Table to map variant names to a canonical name.
CREATE TABLE IF NOT EXISTS name_variants (
    variant_name TEXT PRIMARY KEY NOT NULL,
    canonical_name TEXT NOT NULL
);

-- Example entries for "Shibuya Kanon"
INSERT INTO name_variants (variant_name, canonical_name) VALUES
    ('Kanon Shibuya', 'Shibuya Kanon'),
    ('澁谷かのん', 'Shibuya Kanon');