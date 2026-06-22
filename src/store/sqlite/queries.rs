pub const SQL_SETTINGS_FETCH: &str = "SELECT key, value FROM settings";

pub const SQL_SETTINGS_SAVE: &str = "INSERT INTO settings (key, value) VALUES (?1, ?2)
                        ON CONFLICT(key) DO UPDATE SET value = excluded.value";

pub const SQL_SRV_FETCH_BY_ID: &str = "SELECT name, ip, port, port_query, query_interval, query_timeout, query_type, latency_interval, latency_timeout, latency_type, latency_history_size FROM servers WHERE id = ?1";

pub const SQL_SRV_FETCH_BY_ADDR: &str = "SELECT id, name, port_query, query_interval, query_timeout, query_type, latency_interval, latency_timeout, latency_type, latency_history_size FROM servers WHERE ip = ?1 AND port = ?2";

pub const SQL_SRV_FETCH_ALL: &str = "SELECT id, name, ip, port, port_query, query_interval, query_timeout, query_type, latency_interval, latency_timeout, latency_type, latency_history_size FROM servers";

pub const SQL_SRV_INSERT: &str = "INSERT INTO servers (id, name, ip, port, port_query, query_interval, query_timeout, query_type, latency_interval, latency_timeout, latency_type, latency_history_size) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12) ON CONFLICT(id) DO UPDATE SET name = excluded.name, ip = excluded.ip, port = excluded.port, port_query = excluded.port_query, query_interval = excluded.query_interval, query_timeout = excluded.query_timeout, query_type = excluded.query_type, latency_interval = excluded.latency_interval, latency_timeout = excluded.latency_timeout, latency_type = excluded.latency_type, latency_history_size = excluded.latency_history_size";

pub const SQL_SRV_UPDATE: &str = "UPDATE servers SET name = ?1, ip = ?2, port = ?3, port_query = ?4, query_interval = ?5, query_timeout = ?6, query_type = ?7, latency_interval = ?8, latency_timeout = ?9, latency_type = ?10, latency_history_size = ?11 WHERE id = ?12 OR (ip = ?2 AND port = ?3)";

pub const SQL_SRV_DELETE: &str = "DELETE FROM servers WHERE id = ?1 OR (ip = ?2 AND port = ?3)";
