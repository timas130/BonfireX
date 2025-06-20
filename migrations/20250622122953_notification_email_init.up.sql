create schema notification_email;

create table notification_email.blocked_emails (
    id bigint generated always as identity primary key,
    email text not null unique
);

create table notification_email.blocked_email_domains (
    id bigint generated always as identity primary key,
    domain text not null unique
);

create unique index on notification_email.blocked_email_domains ((domain || '.'));

create table notification_email.email_log (
    id bigint generated always as identity primary key,
    message_id text not null unique,
    destination text not null,
    subject text not null,
    created_at timestamptz not null default now()
);
