PRAGMA main.auto_vacuum = 1;

CREATE TABLE IF NOT EXISTS person (id INTEGER PRIMARY KEY, name TEXT UNIQUE NOT NULL);

CREATE TABLE IF NOT EXISTS pzk (
    snapshot_time INTEGER NOT NULL,
    person_id INTEGER NOT NULL REFERENCES person,
    PRIMARY KEY (snapshot_time, person_id)
) WITHOUT ROWID;
