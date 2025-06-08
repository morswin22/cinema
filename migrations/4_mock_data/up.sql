SET FOREIGN_KEY_CHECKS = 0;

INSERT INTO movies (title, year, director) VALUES
('The Shawshank Redemption', 1994, 'Frank Darabont'),
('The Godfather', 1972, 'Francis Ford Coppola'),
('The Dark Knight', 2008, 'Christopher Nolan'),
('The Godfather: Part II', 1974, 'Francis Ford Coppola'),
('12 Angry Men', 1957, 'Sidney Lumet'),
('The Lord of the Rings: The Return of the King', 2003, 'Peter Jackson'),
('Pulp Fiction', 1994, 'Quentin Tarantino'),
('Schindler''s List', 1993, 'Steven Spielberg'),
('Inception', 2010, 'Christopher Nolan'),
('Fight Club', 1999, 'David Fincher'),
('The Lord of the Rings: The Fellowship of the Ring', 2001, 'Peter Jackson'),
('Forrest Gump', 1994, 'Robert Zemeckis'),
('Il buono, il brutto, il cattivo', 1966, 'Sergio Leone'),
('The Lord of the Rings: The Two Towers', 2002, 'Peter Jackson'),
('The Matrix', 1999, 'Lana Wachowski'),
('Goodfellas', 1990, 'Martin Scorsese'),
('Star Wars: Episode V - The Empire Strikes Back', 1980, 'Irvin Kershner'),
('One Flew Over the Cuckoo''s Nest', 1975, 'Milos Forman'),
('Hamilton', 2020, 'Thomas Kail'),
('Gisaengchung', 2019, 'Bong Joon Ho'),
('Soorarai Pottru', 2020, 'Sudha Kongara'),
('Interstellar', 2014, 'Christopher Nolan'),
('Cidade de Deus', 2002, 'Fernando Meirelles'),
('Sen to Chihiro no kamikakushi', 2001, 'Hayao Miyazaki'),
('Saving Private Ryan', 1998, 'Steven Spielberg'),
('The Green Mile', 1999, 'Frank Darabont'),
('La vita è bella', 1997, 'Roberto Benigni'),
('Se7en', 1995, 'David Fincher'),
('The Silence of the Lambs', 1991, 'Jonathan Demme'),
('Star Wars', 1977, 'George Lucas'),
('Seppuku', 1962, 'Masaki Kobayashi'),
('Shichinin no samurai', 1954, 'Akira Kurosawa'),
('It''s a Wonderful Life', 1946, 'Frank Capra'),
('Joker', 2019, 'Todd Phillips'),
('Whiplash', 2014, 'Damien Chazelle'),
('The Intouchables', 2011, 'Olivier Nakache, Éric Toledano'),
('The Prestige', 2006, 'Christopher Nolan'),
('The Departed', 2006, 'Martin Scorsese'),
('The Pianist', 2002, 'Roman Polanski'),
('Gladiator', 2000, 'Ridley Scott'),
('American History X', 1998, 'Tony Kaye'),
('The Usual Suspects', 1995, 'Bryan Singer'),
('Léon', 1994, 'Luc Besson'),
('The Lion King', 1994, 'Roger Allers, Rob Minkoff'),
('Terminator 2: Judgment Day', 1991, 'James Cameron'),
('Nuovo Cinema Paradiso', 1988, 'Giuseppe Tornatore'),
('Hotaru no haka', 1988, 'Isao Takahata'),
('Back to the Future', 1985, 'Robert Zemeckis'),
('Once Upon a Time in the West', 1968, 'Sergio Leone'),
('Psycho', 1960, 'Alfred Hitchcock');

INSERT INTO rooms (capacity, label) VALUES
(100, 'Screen 1'),
(75, 'Screen 2'),
(120, 'IMAX Theater'),
(50, 'Private Viewing Room'),
(90, 'Classic Cinema Hall'),
(1, 'Test Room');

INSERT INTO schedule (movie_id, room_id, date)
SELECT
    m.id AS movie_id,
    (SELECT id FROM rooms ORDER BY RAND() LIMIT 1) AS room_id,
    DATE_ADD(NOW(), INTERVAL FLOOR(1 + (RAND() * 365)) DAY) + INTERVAL FLOOR(RAND() * 24) HOUR + INTERVAL FLOOR(RAND() * 60) MINUTE AS random_date
FROM
    movies m;

INSERT INTO schedule (movie_id, room_id, date)
SELECT
    m.id AS movie_id,
    (SELECT id FROM rooms ORDER BY RAND() LIMIT 1) AS room_id,
    DATE_ADD(NOW(), INTERVAL FLOOR(1 + (RAND() * 365)) DAY) + INTERVAL FLOOR(RAND() * 24) HOUR + INTERVAL FLOOR(RAND() * 60) MINUTE AS random_date
FROM
    movies m;

SET FOREIGN_KEY_CHECKS = 1;
