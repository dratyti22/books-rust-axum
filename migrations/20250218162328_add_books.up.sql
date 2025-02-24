-- Add up migration script here

CREATE TABLE genres (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE books (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(250) NOT NULL,
    description TEXT,
    author_id UUID REFERENCES users(id),
    genre_id UUID REFERENCES genres(id),
    publication_year SMALLINT,
    isbn VARCHAR(13) UNIQUE,
    cover_image VARCHAR(255),
    price NUMERIC(8, 2) CHECK (price >= 0.00), -- 6 цифр до точки, 2 после
    discount NUMERIC(8, 2) CHECK (discount >= 0.00 AND discount <= 100.00), -- скидка в процентах
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);


CREATE INDEX books_title_idx ON books (title);
CREATE INDEX books_isbn_idx ON books (isbn);
CREATE INDEX books_author_id_idx ON books (author_id);
CREATE INDEX books_genre_id_idx ON books (genre_id);
