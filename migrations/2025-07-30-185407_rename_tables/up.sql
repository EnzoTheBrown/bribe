-- Your SQL goes here
CREATE TABLE USERS (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	full_name VARCHAR(255) NOT NULL,
	birth_date DATE NOT NULL,
	email VARCHAR(255) NOT NULL UNIQUE,
	hashed_password VARCHAR(255) NOT NULL,
	lang VARCHAR(10) NOT NULL DEFAULT 'fr'
);


CREATE TABLE EVENTS (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	date DATE NOT NULL,
	description TEXT NOT NULL,
	person_id INT NOT NULL,
	FOREIGN KEY (person_id) REFERENCES USER(id) ON DELETE CASCADE ON UPDATE CASCADE
);


CREATE TABLE MESSAGES (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	event_id INT NOT NULL,
	source VARCHAR(255) NOT NULL,
	content TEXT NOT NULL,
	FOREIGN KEY (event_id) REFERENCES EVENT(id) ON DELETE CASCADE ON UPDATE CASCADE
);

INSERT INTO users (full_name, birth_date, email, hashed_password, lang)
SELECT full_name, birth_date, email, hashed_password, lang FROM user;

DROP TABLE user;
INSERT into events (date, description, person_id)
SELECT date, description, person_id FROM event;

DROP TABLE event;

INSERT into messages (event_id, source, content)
SELECT event_id, source, content FROM message;
DROP TABLE message;
