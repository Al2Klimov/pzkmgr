PRAGMA main.auto_vacuum = 1;

CREATE TABLE IF NOT EXISTS person (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    birth_year INTEGER,
    birth_month INTEGER,
    birth_day INTEGER,
    CHECK (birth_year BETWEEN 1000 AND 9999),
    CHECK (birth_month BETWEEN 1 AND 12),
    CHECK (birth_day BETWEEN 1 AND 31)
);

CREATE TABLE IF NOT EXISTS pzk (
    snapshot_time INTEGER NOT NULL,
    person_id INTEGER NOT NULL REFERENCES person,
    PRIMARY KEY (snapshot_time, person_id)
) WITHOUT ROWID;
