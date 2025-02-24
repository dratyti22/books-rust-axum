-- Add up migration script here
CREATE TYPE user_role AS ENUM (
    'пользователь',
    'автор',
    'работник',
    'админ',
    'продавец'
);

CREATE TABLE
    "users" (
        id UUID NOT NULL PRIMARY KEY DEFAULT (gen_random_uuid()),
        first_name VARCHAR(100) NOT NULL,
        last_name VARCHAR(100) NOT NULL,
        middle_name VARCHAR(100),
        age INT NOT NULL CHECK (age >= 0 AND age <= 150),
        email VARCHAR(255) NOT NULL UNIQUE,
        biography TEXT,
        file VARCHAR NOT NULL DEFAULT 'uploads/photo-user/default.png',
        verified BOOLEAN NOT NULL DEFAULT FALSE,
        password VARCHAR(100) NOT NULL,
        role user_role NOT NULL DEFAULT 'пользователь',
        balance NUMERIC(12, 2) NOT NULL DEFAULT 0.00
            CHECK (balance >= 0.00 AND balance <= 100000.00),
        rating NUMERIC(4, 2) NOT NULL DEFAULT 0.00
            CHECK (rating >= 0.00 AND rating <= 100.00),
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );

CREATE INDEX users_email_idx ON users (email);