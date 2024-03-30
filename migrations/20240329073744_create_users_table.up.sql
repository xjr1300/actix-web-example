-- ユーザー・テーブルを作成
CREATE TABLE IF NOT EXISTS users (
    id UUID NOT NULL,
    email VARCHAR(254) NOT NULL,
    password VARCHAR(256) NOT NULL,
    active BOOLEAN NOT NULL,
    family_name VARCHAR(40) NOT NULL,
    given_name VARCHAR(40) NOT NULL,
    postal_code VARCHAR(8) NOT NULL,
    address VARCHAR(80) NOT NULL,
    fixed_phone_number VARCHAR(12),
    mobile_phone_number VARCHAR(13),
    remarks VARCHAR(400),
    last_logged_in_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    CONSTRAINT pk_users PRIMARY KEY (id),
    CONSTRAINT ak_users_email UNIQUE (email),
    CONSTRAINT ck_users_either_phone_numbers_must_be_not_null CHECK (
        fixed_phone_number IS NOT NULL
        OR mobile_phone_number IS NOT NULL
    )
);
