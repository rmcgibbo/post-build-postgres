create table if not exists nix_build (
  id serial primary key,
  name text not null,
  drv_path text not null,
  out_paths text[],
  ctime timestamp with time zone not null,
  build_elapsed interval not null,
  instance_type text,
  instance_id text,
  pull_request_number bigint
);

create index nix_build__name on nix_build (name);
