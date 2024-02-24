//ts-node -r tsconfig-paths/register --files test.ts
import Database from 'better-sqlite3';
const db: any = new Database('users.db');

let createUserTableQuery = `
CREATE TABLE IF NOT EXISTS user (
  id  INTEGER PRIMARY KEY ASC,  -- integer affinity by rule 1
  node_id  TEXT,     -- text affinity by rule 2
  device_crt TEXT,
  device_key TEXT
);`;

db.prepare(createUserTableQuery).run();
export default db;
