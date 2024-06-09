CREATE TABLE projects (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    color VARCHAR(6) NOT NULL
);

CREATE TABLE time_entries (
    project_id INT REFERENCES projects(id) NOT NULL,
    day DATE NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME,
    PRIMARY KEY(day, start_time)
);

CREATE TABLE time_comments (
    project_id INT REFERENCES projects(id) NOT NULL,
    day DATE NOT NULL,
    content TEXT,
    PRIMARY KEY(project_id, day)
);

CREATE TABLE upkeep_items (
  id SERIAL PRIMARY KEY,
  description VARCHAR(255) NOT NULL,
  cooldown_days INT NOT NULL,
  due DATE NOT NULL
);
