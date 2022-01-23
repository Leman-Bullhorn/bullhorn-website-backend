CREATE TABLE articles (
  id SERIAL PRIMARY KEY,
  headline VARCHAR NOT NULL,
  slug TEXT NOT NULL,
  body TEXT NOT NULL,
  writer_id int NOT NULL,
  section_id int NOT NULL,
  publication_date TIMESTAMP WITH TIME ZONE NOT NULL,
  preview TEXT,
  image_url TEXT,
  CONSTRAINT fk_writer
    FOREIGN KEY(writer_id)
      REFERENCES writers(id)
      ON DELETE CASCADE,
  CONSTRAINT fk_section
    FOREIGN KEY(section_id)
      REFERENCES sections(id)
      ON DELETE CASCADE
)