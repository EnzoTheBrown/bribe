CREATE TABLE users_new (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	full_name VARCHAR(255) NOT NULL,
	birth_date DATE NOT NULL,
	email VARCHAR(255) NOT NULL UNIQUE,
	hashed_password VARCHAR(255) NOT NULL,
	lang VARCHAR(10) NOT NULL DEFAULT 'fr'
);


INSERT INTO users_new (full_name, birth_date, email, hashed_password, lang)
SELECT full_name, birth_date, email, hashed_password, lang FROM user;

DROP TABLE user;
ALTER TABLE users_new RENAME TO user;
