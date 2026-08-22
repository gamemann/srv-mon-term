pub const SQL_SETTINGS_FETCH: &str = "SELECT key, value FROM settings";

pub const SQL_SETTINGS_SAVE: &str = "INSERT INTO settings (key, value) VALUES (?1, ?2)
                        ON CONFLICT(key) DO UPDATE SET value = excluded.value";

pub const SQL_SRV_FETCH_BY_ID: &str = "SELECT ip, port, display_name, port_query, query_interval, query_timeout, query_type, latency_interval, latency_timeout, latency_type, latency_history_size FROM servers WHERE id = ?1";

pub const SQL_SRV_FETCH_BY_ADDR: &str = "SELECT id, display_name, port_query, query_interval, query_timeout, query_type, latency_interval, latency_timeout, latency_type, latency_history_size FROM servers WHERE ip = ?1 AND port = ?2";

pub const SQL_SRV_FETCH_ALL: &str = "SELECT id, ip, port, display_name, port_query, query_interval, query_timeout, query_type, latency_interval, latency_timeout, latency_type, latency_history_size FROM servers";

pub const SQL_SRV_INSERT: &str = "INSERT INTO servers (id, ip, port, display_name, port_query, query_interval, query_timeout, query_type, latency_interval, latency_timeout, latency_type, latency_history_size) VALUES (:id, :ip, :port, :display_name, :port_query, :query_interval, :query_timeout, :query_type, :latency_interval, :latency_timeout, :latency_type, :latency_history_size) ON CONFLICT(id) DO UPDATE SET display_name = excluded.display_name, ip = excluded.ip, port = excluded.port, port_query = excluded.port_query, query_interval = excluded.query_interval, query_timeout = excluded.query_timeout, query_type = excluded.query_type, latency_interval = excluded.latency_interval, latency_timeout = excluded.latency_timeout, latency_type = excluded.latency_type, latency_history_size = excluded.latency_history_size";

pub const SQL_SRV_UPDATE: &str = "UPDATE servers SET ip = :ip, port = :port, display_name = :display_name, port_query = :port_query, query_interval = :query_interval, query_timeout = :query_timeout, query_type = :query_type, latency_interval = :latency_interval, latency_timeout = :latency_timeout, latency_type = :latency_type, latency_history_size = :latency_history_size WHERE id = :id OR (ip = :ip AND port = :port)";

pub const SQL_SRV_DELETE: &str =
    "DELETE FROM servers WHERE id = :id OR (ip = :ip AND port = :port)";
