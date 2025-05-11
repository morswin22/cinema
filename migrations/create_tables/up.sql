CREATE TABLE users (
                       id INT AUTO_INCREMENT PRIMARY KEY,
                       email VARCHAR(255) NOT NULL,
                       password VARCHAR(255) NOT NULL
);

CREATE TABLE movies (
                        id INT AUTO_INCREMENT PRIMARY KEY,
                        title VARCHAR(255) NOT NULL,
                        year INT NOT NULL,
                        director VARCHAR(255) NOT NULL
);

CREATE TABLE rooms (
                       id INT AUTO_INCREMENT PRIMARY KEY,
                       capacity INT NOT NULL,
                       label VARCHAR(50) NOT NULL
);

CREATE TABLE schedule (
                          id INT AUTO_INCREMENT PRIMARY KEY,
                          movie_id INT NOT NULL,
                          room_id INT NOT NULL,
                          date DATETIME NOT NULL,
                          FOREIGN KEY (movie_id) REFERENCES movies(id),
                          FOREIGN KEY (room_id) REFERENCES rooms(id)
);

CREATE TABLE reservation (
                             id INT AUTO_INCREMENT PRIMARY KEY,
                             user_id INT NOT NULL,
                             schedule_id INT NOT NULL,
                             FOREIGN KEY (user_id) REFERENCES users(id),
                             FOREIGN KEY (schedule_id) REFERENCES schedule(id)
);
