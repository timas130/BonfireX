create schema profile;

create table profile.profiles (
    user_id bigint not null primary key,
    display_name text null,
    username text not null,
    avatar_id bigint null,
    bio text not null default '',
    cover_id bigint null,
    created_at timestamptz not null default now()
);

create unique index on profile.profiles (lower(username));

create table profile.usernames (
    id bigint primary key generated always as identity,
    user_id bigint not null references profile.profiles (user_id) on delete cascade,
    username text not null,
    created_at timestamptz not null default now()
);

create index on profile.usernames (user_id);
create unique index on profile.usernames (lower(username));

create table profile.notes (
    id bigint not null generated always as identity primary key,
    user_id bigint not null references profile.profiles (user_id) on delete cascade,
    profile_id bigint not null references profile.profiles (user_id) on delete cascade,
    note text not null,
    created_at timestamptz not null default now()
);

create unique index on profile.notes (user_id, profile_id);

create type profile.profile_request as (
    user_id bigint,
    username text,
    for_user_id bigint
);
