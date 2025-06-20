create schema image;

create table image.images (
    id bigint not null generated always as identity primary key,
    full_width int not null,
    full_height int not null,
    full_size int not null,
    thumbnail_width int not null,
    thumbnail_height int not null,
    thumbnail_size int not null,
    blur_data bytea not null,
    created_at timestamptz not null default now()
);

create table image.image_tickets (
    id bigint not null generated always as identity primary key,
    ticket text not null unique,
    user_id bigint not null,
    image_id bigint null references image.images (id) on delete cascade,
    created_at timestamptz not null default now()
);

create table image.image_refs (
    image_id bigint not null references image.images (id) on delete cascade,
    ref_id text not null,
    created_at timestamptz not null default now()
);

create unique index on image.image_refs (image_id, ref_id);
