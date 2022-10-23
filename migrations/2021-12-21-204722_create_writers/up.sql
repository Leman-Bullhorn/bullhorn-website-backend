CREATE TABLE writers (
  id SERIAL PRIMARY KEY,
  first_name VARCHAR NOT NULL,
  last_name VARCHAR NOT NULL,
  title VARCHAR NOT NULL,
  bio TEXT,
  image_url TEXT
)
