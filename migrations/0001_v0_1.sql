CREATE TABLE time_categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    color VARCHAR(6) NOT NULL
);

CREATE TABLE time_entries (
    id SERIAL PRIMARY KEY,
    category_id INT REFERENCES time_categories(id) NOT NULL,
    day DATE NOT NULL,
    start_time VARCHAR(4) NOT NULL,
    end_time VARCHAR(4) NOT NULL
);
