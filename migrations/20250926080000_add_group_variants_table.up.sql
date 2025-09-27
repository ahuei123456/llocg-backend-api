-- Table to map variant group names to a canonical group name.
CREATE TABLE IF NOT EXISTS group_variants (
    variant_name TEXT PRIMARY KEY NOT NULL,
    canonical_name TEXT NOT NULL
);

-- Example entries for "μ's"
INSERT INTO group_variants (variant_name, canonical_name) VALUES
    ('ラブライブ！', 'Love Live!'),
    ('ラブライブ！サンシャイン!!', 'Love Live! Sunshine!!'),
    ('ラブライブ！虹ヶ咲学園スクールアイドル同好会', 'Love Live! Nijigasaki High School Idol Club'),
    ('ラブライブ！スーパースター!!', 'Love Live! Superstar!!'),
    ('蓮ノ空女学院スクールアイドルクラブ', 'Hasu no Sora Jogakuin School Idol Club');