-- Wyrd foundation external Postgres role bootstrap template.
--
-- Run this as a cluster administrator before starting wyrd-server in external
-- Postgres mode. The psql caller supplies three variables from the deploy secret
-- store (`--set migrator_password=...`, etc.). Migrations must run as
-- wyrd_migrator after these roles exist.

\set ON_ERROR_STOP on

SELECT format('CREATE ROLE wyrd_migrator LOGIN NOCREATEDB BYPASSRLS NOSUPERUSER PASSWORD %L', :'migrator_password')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'wyrd_migrator')\gexec
SELECT format('CREATE ROLE wyrd_app LOGIN NOCREATEDB NOBYPASSRLS NOSUPERUSER PASSWORD %L', :'app_password')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'wyrd_app')\gexec
SELECT format('CREATE ROLE wyrd_platform_admin LOGIN NOCREATEDB BYPASSRLS NOSUPERUSER PASSWORD %L', :'platform_admin_password')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'wyrd_platform_admin')\gexec

ALTER ROLE wyrd_migrator WITH LOGIN NOCREATEDB BYPASSRLS NOSUPERUSER PASSWORD :'migrator_password';
ALTER ROLE wyrd_app WITH LOGIN NOCREATEDB NOBYPASSRLS NOSUPERUSER PASSWORD :'app_password';
ALTER ROLE wyrd_platform_admin WITH LOGIN NOCREATEDB BYPASSRLS NOSUPERUSER PASSWORD :'platform_admin_password';

REVOKE ALL PRIVILEGES ON DATABASE wyrd FROM wyrd_migrator, wyrd_app, wyrd_platform_admin;
REVOKE wyrd_migrator, wyrd_app, wyrd_platform_admin FROM wyrd_migrator, wyrd_app, wyrd_platform_admin;

GRANT CONNECT ON DATABASE wyrd TO wyrd_migrator, wyrd_app, wyrd_platform_admin;
GRANT CREATE ON DATABASE wyrd TO wyrd_migrator;
