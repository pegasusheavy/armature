-- TechEmpower Framework Benchmark Database Schema
-- PostgreSQL version
-- Run: psql -h localhost -U benchmarkdbuser -d hello_world -f create.sql

-- World table (10,000 rows)
DROP TABLE IF EXISTS World;
CREATE TABLE World (
    id INTEGER NOT NULL PRIMARY KEY,
    randomNumber INTEGER NOT NULL
);

-- Fortune table (12 rows)
DROP TABLE IF EXISTS Fortune;
CREATE TABLE Fortune (
    id INTEGER NOT NULL PRIMARY KEY,
    message TEXT NOT NULL
);

-- Populate World table with random numbers
INSERT INTO World (id, randomNumber)
SELECT s.id, floor(random() * 10000 + 1)::int
FROM generate_series(1, 10000) AS s(id);

-- Populate Fortune table
INSERT INTO Fortune (id, message) VALUES
(1, 'fortune: No such file or directory'),
(2, 'A computer scientist is someone who fixes things that aren''t broken.'),
(3, 'After enough decimal places, nobody gives a damn.'),
(4, 'A bad random number generator: 1, 1, 1, 1, 1, 4.33e+67, 1, 1, 1'),
(5, 'A computer program does what you tell it to do, not what you want it to do.'),
(6, 'Emstrongs''s Law: If you make something idiot-proof, someone will make a better idiot.'),
(7, 'Feature: A bug with seniority.'),
(8, 'Computers make very fast, very accurate mistakes.'),
(9, '<script>alert("This should not be displayed in a alarm box.");</script>'),
(10, 'フレームワークのベンチマーク'),
(11, 'Any program that runs right is obsolete.'),
(12, 'Waste not, compute not.');

-- Create index for random lookups
CREATE INDEX idx_world_id ON World(id);
CREATE INDEX idx_fortune_id ON Fortune(id);

-- Verify
SELECT 'World rows: ' || COUNT(*) FROM World;
SELECT 'Fortune rows: ' || COUNT(*) FROM Fortune;

