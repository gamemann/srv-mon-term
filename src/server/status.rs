use crate::{
    cli::QueryMonitor,
    context::Context,
    server::{ServerCtx, ServerStatuses, data::ServerStatus},
};

impl ServerCtx {
    pub async fn get_status(&self, statuses: ServerStatuses, ctx: Context) -> ServerStatus {
        let err = {
            let query_monitor = ctx.args.parse_query_monitor();
            let monitor_only = ctx.args.use_query_monitor_only;

            let do_info =
                (monitor_only && query_monitor == Some(QueryMonitor::Info)) || !monitor_only;

            let info_err = if let ServerStatus::Error(code) = statuses.query_info {
                Some(code.clone())
            } else {
                None
            };

            let do_vars =
                (monitor_only && query_monitor == Some(QueryMonitor::Vars)) || !monitor_only;

            let vars_err = if let ServerStatus::Error(code) = statuses.query_vars {
                Some(code.clone())
            } else {
                None
            };

            let do_users =
                (monitor_only && query_monitor == Some(QueryMonitor::Users)) || !monitor_only;

            let users_err = if let ServerStatus::Error(code) = statuses.query_users {
                Some(code.clone())
            } else {
                None
            };

            match query_monitor {
                Some(QueryMonitor::Info) => {
                    if do_info {
                        info_err
                    } else {
                        None
                    }
                }
                Some(QueryMonitor::Vars) => {
                    if do_vars {
                        vars_err
                    } else {
                        None
                    }
                }
                Some(QueryMonitor::Users) => {
                    if do_users {
                        users_err
                    } else {
                        None
                    }
                }

                None => {
                    if do_info || do_vars || do_users {
                        info_err.or(vars_err).or(users_err)
                    } else {
                        None
                    }
                }
            }
        };

        if let Some(e) = err {
            ServerStatus::Error(e)
        } else {
            ServerStatus::Online
        }
    }
}
