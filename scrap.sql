CREATE SCHEMA IF NOT EXISTS scrap;

CREATE TABLE IF NOT EXISTS scrap.challenge (
	id SERIAL PRIMARY KEY CHECK (id <= 64),
	slug TEXT NOT NULL UNIQUE,
	title TEXT NOT NULL,
	author TEXT NOT NULL,
	value INTEGER NOT NULL,
	description TEXT NOT NULL,
	tags TEXT[],
	flag TEXT NOT NULL,
	enabled BOOLEAN,
	solves INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS scrap.team (
	id SERIAL PRIMARY KEY,
	name TEXT NOT NULL UNIQUE,
	discord TEXT NOT NULL UNIQUE,
	hash TEXT NOT NULL,
	solves BIGINT DEFAULT 0,
	score INTEGER DEFAULT 0,
	submit TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS scrap.session (
	cookie TEXT PRIMARY KEY,
	team INTEGER NOT NULL REFERENCES scrap.team ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS scrap.ctf (
	id INTEGER NOT NULL UNIQUE CHECK (id = 1) DEFAULT 1,
	title TEXT NOT NULL,
	home TEXT NOT NULL,
	start TIMESTAMP WITH TIME ZONE,
	stop TIMESTAMP WITH TIME ZONE
);

CREATE TABLE IF NOT EXISTS scrap.event (
	id INTEGER NOT NULL UNIQUE,
	title TEXT NOT NULL,
	short TEXT NOT NULL,
	date TEXT NOT NULL,
	description TEXT NOT NULL,
	link TEXT,
	slides TEXT
);

CREATE INDEX IF NOT EXISTS team_name_hash_index ON scrap.team (name, hash);
CREATE INDEX IF NOT EXISTS team_score_submit_index ON scrap.team (score DESC, submit ASC) INCLUDE (name, solves);
CREATE INDEX IF NOT EXISTS session_cookie_index ON scrap.session (cookie);

CREATE OR REPLACE FUNCTION lookup(TEXT) RETURNS INTEGER AS $$
SELECT team FROM scrap.session WHERE cookie=$1 LIMIT 1;
$$ LANGUAGE sql STABLE;

CREATE OR REPLACE FUNCTION solved(solves BIGINT, id INTEGER) RETURNS BOOLEAN AS $$
SELECT ((COALESCE(solves, 0) >> (id - 1)) & 1)=1;
$$ LANGUAGE sql IMMUTABLE;

CREATE OR REPLACE FUNCTION update(solves BIGINT, id INTEGER) RETURNS BIGINT AS $$
SELECT solves | (1 << (id - 1));
$$ LANGUAGE sql IMMUTABLE;
