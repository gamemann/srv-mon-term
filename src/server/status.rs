use crate::{
    cli::QueryMonitor,
    context::Context,
    server::{ServerCtx, ServerStatuses, data::ServerStatus},
};

impl ServerCtx {
    /// Resolves the overall status we show for a server.
    ///
    /// The status of the query we monitor is authoritative. Failures of the other queries are
    /// surfaced in their own panels instead, since plenty of servers answer info queries while
    /// blocking the player or rule queries.
    pub async fn get_status(&self, statuses: ServerStatuses, ctx: Context) -> ServerStatus {
        let query_monitor = ctx.args.parse_query_monitor();
        let monitor_only = ctx.args.use_query_monitor_only;

        let primary = if monitor_only {
            match query_monitor {
                Some(QueryMonitor::Users) => statuses.query_users.clone(),
                Some(QueryMonitor::Vars) => statuses.query_vars.clone(),
                _ => statuses.query_info.clone(),
            }
        } else {
            statuses.query_info.clone()
        };

        if primary != ServerStatus::Unknown {
            return primary;
        }

        // Fall back to whichever query has actually run so far.
        [
            statuses.query_info,
            statuses.query_users,
            statuses.query_vars,
        ]
        .into_iter()
        .find(|s| *s != ServerStatus::Unknown)
        .unwrap_or(ServerStatus::Unknown)
    }
}
