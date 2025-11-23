-- Add up migration script here
-- This script populates the database with initial data for names, groups, sets, and units.

-- Populate the 'names' table with initial canonical card names.
-- The 'id' will be auto-incremented.
INSERT INTO names (name) VALUES
('Kousaka Honoka'),
('Ayase Eli'),
('Minami Kotori'),
('Sonoda Umi'),
('Hoshizora Rin'),
('Nishikino Maki'),
('Tojo Nozomi'),
('Koizumi Hanayo'),
('Yazawa Nico'),
('Takami Chika'),
('Sakurauchi Riko'),
('Matsuura Kanan'),
('Kurosawa Dia'),
('Watanabe You'),
('Tsushima Yoshiko'),
('Kunikida Hanamaru'),
('Ohara Mari'),
('Kurosawa Ruby'),
('Uehara Ayumu'),
('Nakasu Kasumi'),
('Osaka Shizuku'),
('Asaka Karin'),
('Miyashita Ai'),
('Konoe Kanata'),
('Yuki Setsuna'),
('Emma Verde'),
('Tennoji Rina'),
('Mifune Shioriko'),
('Mia Taylor'),
('Zhong Lanzhu'),
('Shibuya Kanon'),
('Tang Keke'),
('Arashi Chisato'),
('Heanna Sumire'),
('Hazuki Ren'),
('Sakurakoji Kinako'),
('Yoneme Mei'),
('Wakana Shiki'),
('Onitsuka Natsumi'),
('Wien Margarete'),
('Onitsuka Tomari'),
('Hinoshita Kaho'),
('Murano Sayaka'),
('Otomune Kozue'),
('Yugiri Tsuzuri'),
('Osawa Rurino'),
('Fujishima Megumi'),
('Momose Ginko'),
('Kachimachi Kosuzu'),
('Anyoji Hime'),
('Ceras Yanagida Lilienfeld'),
('Katsuragi Izumi');
-- TODO: Add more canonical names here.

-- Populate the 'groups' table with initial groups.
INSERT INTO groups (name) VALUES
('Love Live!'),
('Love Live! Sunshine!!'),
('Love Live! Nijigasaki High School Idol Club'),
('Love Live! Superstar!!'),
('Hasu no Sora Jogakuin School Idol Club');

-- Populate the 'sets' table with initial card sets.
INSERT INTO sets (set_code, name) VALUES
('PR', 'Promo Cards'),
('NSD01', 'Start Deck Love Live! Nijigasaki High School Idol Club'),
('PLSD01', 'Start Deck Love Live! Superstar!!'),
('BP01', 'Booster Pack vol.1'),
('PBSP', 'Premium Booster Love Live! Superstar!!'),
('BP02', 'Booster Pack NEXT STEP'),
('PBLS', 'Premium Booster Love Live! Sunshine!!'),
('BP03', 'Booster Pack The Beginning of Summer'),
('PBLL', 'Premium Booster Love Live!'),
('BP04', 'Booster Pack SAPPHIRE MOON');

-- Populate the 'units' table with initial units.
INSERT INTO units (name) VALUES
('Printemps'),
('BiBi'),
('lily white'),
('A-RISE'),
('CYaRon!'),
('Guilty Kiss'),
('AZALEA'),
('Saint Snow'),
('A・ZU・NA'),
('QU4RTZ'),
('DiverDiva'),
('R3BIRTH'),
('CatChu!'),
('KALEIDOSCORE'),
('5yncri5e!'),
('Sunny Passion'),
('Cerise Bouquet'),
('DOLLCHESTRA'),
('Mira-Cra Park!'),
('Edel Note');